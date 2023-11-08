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
    applications::transfer::{
        msgs::transfer::MsgTransfer, packet::PacketData, Amount, BaseDenom, Memo, PrefixedCoin,
        PrefixedDenom, TracePath,
    },
    core::{
        ics04_channel::timeout::TimeoutHeight,
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
        timestamp::Timestamp,
        MsgEnvelope,
    },
    Height, Signer,
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
    AccountId, BorshStorageKey, PanicOnDefault,
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

mod collections;
mod context;
mod events;
mod ext_interfaces;
mod ibc_impl;
pub mod migration;
mod module_holder;
mod prelude;
mod testnet_functions;
pub mod types;
pub mod viewer;

pub const VERSION: &str = "v1.2.0-pre.0";
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
    ClientProcessedTimesIndex {
        client_id: ClientId,
    },
    ClientProcessedTimesKey {
        client_id: ClientId,
    },
    ClientProcessedHeights,
    ClientProcessedHeightsIndex {
        client_id: ClientId,
    },
    ClientProcessedHeightsKey {
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
    #[payable]
    pub fn deliver(&mut self, messages: Vec<Any>) {
        assert!(
            env::attached_deposit()
                >= utils::MINIMUM_DEPOSIT_FOR_DELEVER_MSG * messages.len() as u128,
            "Need to attach at least {} yocto NEAR to cover the possible storage cost.",
            utils::MINIMUM_DEPOSIT_FOR_DELEVER_MSG * messages.len() as u128
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        // Deliver messages to `ibc-rs`
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();

        messages.into_iter().fold(vec![], |mut errors, msg| {
            match MsgEnvelope::try_from(msg.clone()) {
                Ok(msg) => match ibc::core::dispatch(&mut near_ibc_store, self, msg.clone()) {
                    Ok(()) => (),
                    Err(e) => {
                        log!("Error occurred in processing message: {:?}, {:?}", msg, e);
                        errors.push(e)
                    }
                },
                Err(e) => {
                    log!("Error occurred in routing message: {:?}, {:?}", msg, e);
                    errors.push(e)
                }
            }
            errors
        });
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
    /// Setup the token contract for the given asset denom with the given metadata.
    ///
    /// Only the governance account can call this function.
    #[payable]
    pub fn setup_wrapped_token(
        &mut self,
        trace_path: String,
        base_denom: String,
        metadata: FungibleTokenMetadata,
    ) {
        self.assert_governance();
        assert!(
            env::prepaid_gas() >= utils::GAS_FOR_COMPLEX_FUNCTION_CALL,
            "ERR_NOT_ENOUGH_GAS"
        );
        let asset_denom = AssetDenom {
            trace_path: trace_path.clone(),
            base_denom: base_denom.clone(),
        };
        let cross_chain_asset = CrossChainAsset {
            asset_id: "00000000000000000000000000000000".to_string(),
            asset_denom: asset_denom.clone(),
            metadata: metadata.clone(),
        };
        let minimum_deposit = utils::INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT
            + env::storage_byte_cost()
                * (32 + borsh::to_vec(&cross_chain_asset).unwrap().len()) as u128;
        assert!(
            env::attached_deposit() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        ext_token_factory::ext(utils::get_token_factory_contract_id())
            .with_attached_deposit(minimum_deposit)
            .with_static_gas(
                utils::GAS_FOR_COMPLEX_FUNCTION_CALL
                    .checked_sub(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                    .unwrap(),
            )
            .with_unused_gas_weight(0)
            .setup_asset(asset_denom.trace_path, asset_denom.base_denom, metadata);
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
    }
    /// Set the max length of the IBC events history queue.
    ///
    /// Only the governance account can call this function.
    pub fn set_max_length_of_ibc_events_history(&mut self, max_length: u64) -> ProcessingResult {
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let result = near_ibc_store.ibc_events_history.set_max_length(max_length);
        self.near_ibc_store.set(&near_ibc_store);
        result
    }
    /// Setup the escrow contract for the given channel.
    ///
    /// Only the governance account can call this function.
    #[payable]
    pub fn setup_channel_escrow(&mut self, channel_id: String) {
        self.assert_governance();
        assert!(
            env::prepaid_gas() >= utils::GAS_FOR_COMPLEX_FUNCTION_CALL,
            "ERR_NOT_ENOUGH_GAS"
        );
        let minimum_deposit = utils::INIT_BALANCE_FOR_CHANNEL_ESCROW_CONTRACT
            + env::storage_byte_cost() * (borsh::to_vec(&channel_id).unwrap().len() + 16) as u128;
        assert!(
            env::attached_deposit() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        ext_escrow_factory::ext(utils::get_escrow_factory_contract_id())
            .with_attached_deposit(minimum_deposit)
            .with_static_gas(
                utils::GAS_FOR_COMPLEX_FUNCTION_CALL
                    .checked_sub(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                    .unwrap(),
            )
            .with_unused_gas_weight(0)
            .create_escrow(ChannelId::from_str(channel_id.as_str()).unwrap());
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
    }
    /// Register the given token contract for the given channel.
    ///
    /// Only the governance account can call this function.
    #[payable]
    pub fn register_asset_for_channel(
        &mut self,
        channel_id: String,
        base_denom: String,
        token_contract: AccountId,
    ) {
        self.assert_governance();
        let prefixed_base_account = format!(".{}", env::current_account_id());
        assert!(
            !token_contract
                .to_string()
                .ends_with(prefixed_base_account.as_str()),
            "ERR_INVALID_TOKEN_CONTRACT_ACCOUNT, \
            must not be the cross chain assets received by near-ibc."
        );
        let asset_denom = AssetDenom {
            trace_path: String::new(),
            base_denom,
        };
        let minimum_deposit = env::storage_byte_cost()
            * (borsh::to_vec(&asset_denom).unwrap().len() + token_contract.to_string().len())
                as u128;
        assert!(
            env::attached_deposit() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        let escrow_account_id =
            format!("{}.{}", channel_id, utils::get_escrow_factory_contract_id());
        ext_channel_escrow::ext(AccountId::from_str(escrow_account_id.as_str()).unwrap())
            .with_attached_deposit(minimum_deposit)
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
            .with_unused_gas_weight(0)
            .register_asset(asset_denom.base_denom, token_contract);
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
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
