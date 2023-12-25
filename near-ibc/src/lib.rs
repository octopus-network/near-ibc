#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

extern crate alloc;
#[cfg(any(test, feature = "std"))]
extern crate std;

use crate::{context::NearIbcStore, ibc_impl::applications::transfer::TransferModule, prelude::*};
use core::str::FromStr;
use ibc::{
    apps::transfer::types::{
        msgs::transfer::MsgTransfer, packet::PacketData, Amount, BaseDenom, Memo, PrefixedCoin,
        PrefixedDenom, TracePath,
    },
    core::{
        channel::types::timeout::TimeoutHeight,
        client::types::Height,
        handler::types::msgs::MsgEnvelope,
        host::types::identifiers::{ChannelId, ClientId, ConnectionId, PortId},
        primitives::{Signer, Timestamp},
    },
};
use ibc_proto::google::protobuf::Any;
use module_holder::ModuleHolder;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::{Base64VecU8, U128},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json,
    store::LookupMap,
    AccountId, BorshStorageKey, NearToken, PanicOnDefault,
};
use octopus_lpos::msgs::MsgValidatorSetChange;
use types::*;
use utils::{
    interfaces::{
        ext_channel_escrow, ext_escrow_factory, ext_process_transfer_request_callback,
        ext_token_factory, TransferRequestHandler,
    },
    types::{AssetDenom, CrossChainAsset, Ics20TransferRequest},
    ExtraDepositCost,
};

pub mod collections;
mod context;
mod events;
mod ext_interfaces;
mod ibc_impl;
pub mod migration;
mod module_holder;
mod prelude;
mod sudo_functions;
mod testnet_functions;
pub mod types;
pub mod viewer;

const VERSION: &str = env!("CARGO_PKG_VERSION");
/// The default timeout seconds for the `MsgTransfer` message.
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 1000;

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey, Clone, Debug)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    NearIbcStore,
    PortToModule,
    ClientIdSet,
    ClientConsensusStateHeightSets,
    ClientConsensusStateHeightSet {
        client_id: ClientId,
    },
    ClientProcessedTimes,
    ClientProcessedTimesMap {
        client_id: ClientId,
    },
    ClientProcessedHeights,
    ClientProcessedHeightsMap {
        client_id: ClientId,
    },
    ConnectionIdSet,
    PortChannelIdSet,
    PacketCommitmentSequenceSets,
    PacketCommitmentSequenceSet {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketReceiptSequenceSets,
    PacketReceiptSequenceSet {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketAcknowledgementSequenceSets,
    PacketAcknowledgementSequenceSet {
        port_id: PortId,
        channel_id: ChannelId,
    },
    IbcEventsHistoryIndexMap,
    IbcEventsHistoryValueMap,
    ChainIdChannelMap,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct NearIbcContract {
    near_ibc_store: LazyOption<NearIbcStore>,
    /// To support the mutable borrow in `Router::get_route_mut`.
    module_holder: ModuleHolder,
    governance_account: AccountId,
}

#[near_bindgen]
impl NearIbcContract {
    #[private]
    #[init]
    pub fn init(appchain_registry_account: AccountId) -> Self {
        env::storage_write("version".as_bytes(), VERSION.as_bytes());
        Self {
            near_ibc_store: LazyOption::new(StorageKey::NearIbcStore, Some(&NearIbcStore::new())),
            governance_account: env::current_account_id(),
            module_holder: ModuleHolder::new(appchain_registry_account),
        }
    }
    ///
    pub fn version(&self) -> String {
        VERSION.to_string()
    }
    ///
    #[payable]
    pub fn deliver(&mut self, messages: Vec<Any>) {
        assert!(
            env::attached_deposit().as_yoctonear()
                >= utils::MINIMUM_DEPOSIT_FOR_DELEVER_MSG * messages.len() as u128,
            "Need to attach at least {} yocto NEAR to cover the possible storage cost.",
            utils::MINIMUM_DEPOSIT_FOR_DELEVER_MSG * messages.len() as u128
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        // Deliver messages to `ibc-rs`
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();

        let mut errors_count = 0;
        messages
            .into_iter()
            .for_each(|msg| match MsgEnvelope::try_from(msg.clone()) {
                Ok(msg) => match ibc::core::handler::entrypoint::dispatch(
                    &mut near_ibc_store,
                    self,
                    msg.clone(),
                ) {
                    Ok(()) => (),
                    Err(e) => {
                        log!("Error occurred in processing message: {:?}, {:?}", msg, e);
                        errors_count += 1;
                    }
                },
                Err(e) => {
                    log!("Error occurred in routing message: {:?}, {:?}", msg, e);
                    errors_count += 1;
                }
            });
        if errors_count > 0 {
            log!(
                r#"EVENT_JSON:{{"standard":"nep297","version":"1.0.0","event":"ERR_DELIVER_MESSAGE"}}"#,
            );
        }
        near_ibc_store.flush();
        self.near_ibc_store.set(&near_ibc_store);
        // Refund unused deposit.
        utils::refund_deposit(used_bytes);
    }
    // Assert that the caller is the preset governance account.
    fn assert_governance(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.governance_account,
            "ERR_NOT_GOVERNANCE_ACCOUNT"
        );
    }
}

pub struct TransferringCoins {
    pub trace_path: String,
    pub base_denom: String,
    pub amount: String,
}

impl TryInto<PrefixedCoin> for TransferringCoins {
    type Error = String;

    fn try_into(self) -> Result<PrefixedCoin, Self::Error> {
        Ok(PrefixedCoin {
            denom: PrefixedDenom {
                trace_path: TracePath::from_str(self.trace_path.as_str())
                    .map_err(|_| "ERR_INVALID_TRACE_PATH".to_string())?,
                base_denom: BaseDenom::from_str(self.base_denom.as_str())
                    .map_err(|_| "ERR_INVALID_BASE_DENOM".to_string())?,
            },
            amount: Amount::from_str(&self.amount.as_str())
                .map_err(|_| "ERR_INVALID_AMOUNT".to_string())?,
        })
    }
}

#[no_mangle]
pub extern "C" fn remove_storage_keys() {
    env::setup_panic_hook();
    near_sdk::assert_self();
    assert!(
        !env::current_account_id().to_string().ends_with(".near"),
        "This function can not be called on mainnet."
    );

    let input = env::input().unwrap();
    //
    #[derive(Serialize, Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    struct Args {
        pub keys: Vec<String>,
    }
    //
    let args: Args = serde_json::from_slice(&input).unwrap();
    for key in args.keys {
        let json_str = format!("\"{}\"", key);
        log!(
            "Remove key '{}': {}",
            key,
            env::storage_remove(&serde_json::from_str::<Base64VecU8>(&json_str).unwrap().0)
        );
    }
}
