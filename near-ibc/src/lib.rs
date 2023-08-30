#![no_std]
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
        msgs::transfer::MsgTransfer, packet::PacketData, send_transfer, Amount, BaseDenom, Memo,
        PrefixedCoin, PrefixedDenom, TracePath,
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
use utils::{
    interfaces::{
        ext_channel_escrow, ext_escrow_factory, ext_process_transfer_request_callback,
        ext_token_factory, TransferRequestHandler,
    },
    types::{AssetDenom, Ics20TransferRequest},
    ExtraDepositCost,
};

mod collections;
mod context;
mod events;
mod ibc_impl;
pub mod migration;
mod module_holder;
mod prelude;
mod testnet_functions;
pub mod types;
pub mod viewer;

pub const VERSION: &str = "v1.0.0-pre.3";

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey, Clone)]
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
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    near_ibc_store: LazyOption<NearIbcStore>,
    governance_account: AccountId,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn init() -> Self {
        env::storage_write("version".as_bytes(), VERSION.as_bytes());
        Self {
            near_ibc_store: LazyOption::new(StorageKey::NearIbcStore, Some(&NearIbcStore::new())),
            governance_account: env::current_account_id(),
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

        let errors = messages.into_iter().fold(vec![], |mut errors, msg| {
            match MsgEnvelope::try_from(msg) {
                Ok(msg) => match ibc::core::dispatch(&mut near_ibc_store, msg) {
                    Ok(()) => (),
                    Err(e) => errors.push(e),
                },
                Err(e) => errors.push(e),
            }
            errors
        });
        if errors.len() > 0 {
            log!("Error(s) occurred: {:?}", errors);
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
    /// Setup the token contract for the given asset denom with the given metadata.
    ///
    /// Only the governance account can call this function.
    #[payable]
    pub fn setup_wrapped_token(
        &mut self,
        port_id: String,
        channel_id: String,
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
            trace_path,
            base_denom,
        };
        let minimum_deposit = utils::INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT
            + env::storage_byte_cost() * (asset_denom.try_to_vec().unwrap().len() + 32) as u128 * 2;
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
                utils::GAS_FOR_COMPLEX_FUNCTION_CALL - utils::GAS_FOR_SIMPLE_FUNCTION_CALL,
            )
            .with_unused_gas_weight(0)
            .setup_asset(
                port_id,
                channel_id,
                asset_denom.trace_path,
                asset_denom.base_denom,
                metadata,
            );
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
    }
    /// Set the max length of the IBC events history queue.
    ///
    /// Only the governance account can call this function.
    pub fn set_max_length_of_ibc_events_history(&mut self, max_length: u64) {
        self.assert_governance();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.ibc_events_history.set_max_length(max_length);
        self.near_ibc_store.set(&near_ibc_store);
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
            + env::storage_byte_cost() * (channel_id.try_to_vec().unwrap().len() + 16) as u128;
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
                utils::GAS_FOR_COMPLEX_FUNCTION_CALL - utils::GAS_FOR_SIMPLE_FUNCTION_CALL,
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
        denom: String,
        token_contract: AccountId,
    ) {
        self.assert_governance();
        let minimum_deposit =
            env::storage_byte_cost() * (denom.len() + token_contract.to_string().len()) as u128;
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
            .register_asset(denom, token_contract);
        ExtraDepositCost::add(minimum_deposit);
        utils::refund_deposit(used_bytes);
    }
}

#[near_bindgen]
impl TransferRequestHandler for Contract {
    fn process_transfer_request(&mut self, transfer_request: Ics20TransferRequest) {
        utils::assert_sub_account();
        if let Err(e) = send_transfer(
            &mut TransferModule(),
            MsgTransfer {
                port_id_on_a: PortId::from_str(transfer_request.port_on_a.as_str()).unwrap(),
                chan_id_on_a: ChannelId::from_str(transfer_request.chan_on_a.as_str()).unwrap(),
                packet_data: PacketData {
                    token: PrefixedCoin {
                        denom: PrefixedDenom {
                            trace_path: TracePath::from_str(
                                transfer_request.token_trace_path.as_str(),
                            )
                            .unwrap(),
                            base_denom: BaseDenom::from_str(transfer_request.token_denom.as_str())
                                .unwrap(),
                        },
                        amount: Amount::from_str(transfer_request.amount.0.to_string().as_str())
                            .unwrap(),
                    },
                    sender: Signer::from(transfer_request.sender.clone()),
                    receiver: Signer::from(transfer_request.receiver.clone()),
                    memo: Memo::from_str("").unwrap(),
                },
                timeout_height_on_b: TimeoutHeight::Never {},
                timeout_timestamp_on_b: Timestamp::from_nanoseconds(
                    env::block_timestamp() + 1000 * 1000000000,
                )
                .unwrap(),
            },
        ) {
            log!("ERR_SEND_TRANSFER: {:?}", e);
            log!(
                "Cancelling transfer request for account {}, trace path {}, base denom {} with amount {}",
                transfer_request.sender,
                transfer_request.token_trace_path,
                transfer_request.token_denom,
                transfer_request.amount.0
            );
            ext_process_transfer_request_callback::ext(env::predecessor_account_id())
                .with_attached_deposit(0)
                .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 4)
                .with_unused_gas_weight(0)
                .cancel_transfer_request(
                    transfer_request.token_denom,
                    AccountId::from_str(transfer_request.sender.as_str()).unwrap(),
                    transfer_request.amount,
                );
        }
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

/// Some sudo functions for testing.
#[near_bindgen]
impl Contract {
    pub fn cancel_transfer_request_in_channel_escrow(
        &mut self,
        channel_id: String,
        token_denom: String,
        sender_id: AccountId,
        amount: U128,
    ) {
        self.assert_governance();
        let channel_escrow_id =
            format!("{}.{}", channel_id, utils::get_escrow_factory_contract_id());
        ext_process_transfer_request_callback::ext(
            AccountId::from_str(channel_escrow_id.as_str()).unwrap(),
        )
        .with_attached_deposit(0)
        .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 4)
        .with_unused_gas_weight(0)
        .cancel_transfer_request(token_denom, sender_id, amount);
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
