use crate::{
    context::{NearIbcStore, NearRouterContext},
    events::EventEmit,
    ibc_impl::applications::transfer::TransferModule,
};
use ibc::{
    applications::transfer::{
        msgs::transfer::MsgTransfer, relay::send_transfer::send_transfer, Amount, BaseDenom,
        PrefixedCoin, PrefixedDenom, TracePath,
    },
    core::{
        ics04_channel::timeout::TimeoutHeight,
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
        ics26_routing::handler::MsgReceipt,
    },
    events::IbcEvent,
    handler::HandlerOutput,
    signer::Signer,
    timestamp::Timestamp,
    Height,
};
use ibc_proto::google::protobuf::Any;
use indexed_lookup_queue::IndexedLookupQueue;
use itertools::Itertools;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::{Base64VecU8, U128},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json,
    store::{LookupMap, UnorderedMap},
    AccountId, BorshStorageKey, PanicOnDefault, Promise,
};
use std::str::FromStr;
use utils::{
    interfaces::{
        ext_channel_escrow, ext_escrow_factory, ext_process_transfer_request_callback,
        ext_token_factory, TransferRequestHandler,
    },
    types::{AssetDenom, Ics20TransferRequest},
};

pub mod context;
pub mod events;
pub mod ibc_impl;
pub mod indexed_lookup_queue;
pub mod migration;
pub mod viewer;

pub const DEFAULT_COMMITMENT_PREFIX: &str = "ibc";

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    ClientTypes,
    ClientStates,
    ConsensusStates,
    ConsensusStatesIndex {
        client_id: ClientId,
    },
    ConsensusStatesKey {
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
    ClientConnections,
    ClientConnectionsVector {
        client_id: ClientId,
    },
    Connections,
    PortToModule,
    ConnectionChannels,
    ConnectionChannelsVector {
        connection_id: ConnectionId,
    },
    Channels,
    NextSequenceSend,
    NextSequenceRecv,
    NextSequenceAck,
    PacketReceipt,
    PacketReceiptIndex {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketReceiptKey {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketAcknowledgement,
    PacketAcknowledgementIndex {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketAcknowledgementKey {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketCommitment,
    PacketCommitmentIndex {
        port_id: PortId,
        channel_id: ChannelId,
    },
    PacketCommitmentKey {
        port_id: PortId,
        channel_id: ChannelId,
    },
    NearIbcStore,
    IbcEventsHistoryIndex,
    IbcEventsHistoryKey,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    near_ibc_store: LazyOption<NearIbcStore>,
    ibc_events_history: IndexedLookupQueue<u64, Vec<u8>>,
    governance_account: AccountId,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn init() -> Self {
        Self {
            near_ibc_store: LazyOption::new(
                StorageKey::NearIbcStore,
                Some(&NearIbcStore {
                    client_types: LookupMap::new(StorageKey::ClientTypes),
                    client_states: UnorderedMap::new(StorageKey::ClientStates),
                    consensus_states: LookupMap::new(StorageKey::ConsensusStates),
                    client_processed_times: LookupMap::new(StorageKey::ClientProcessedTimes),
                    client_processed_heights: LookupMap::new(StorageKey::ClientProcessedHeights),
                    client_ids_counter: 0,
                    client_connections: LookupMap::new(StorageKey::ClientConnections),
                    connections: UnorderedMap::new(StorageKey::Connections),
                    connection_ids_counter: 0,
                    connection_channels: LookupMap::new(StorageKey::ConnectionChannels),
                    channel_ids_counter: 0,
                    channels: UnorderedMap::new(StorageKey::Channels),
                    next_sequence_send: LookupMap::new(StorageKey::NextSequenceSend),
                    next_sequence_recv: LookupMap::new(StorageKey::NextSequenceRecv),
                    next_sequence_ack: LookupMap::new(StorageKey::NextSequenceAck),
                    packet_receipts: LookupMap::new(StorageKey::PacketReceipt),
                    packet_acknowledgements: LookupMap::new(StorageKey::PacketAcknowledgement),
                    port_to_module: LookupMap::new(StorageKey::PortToModule),
                    packet_commitments: LookupMap::new(StorageKey::PacketCommitment),
                }),
            ),
            ibc_events_history: IndexedLookupQueue::new(
                StorageKey::IbcEventsHistoryIndex,
                StorageKey::IbcEventsHistoryKey,
                u64::MAX,
            ),
            governance_account: env::current_account_id(),
        }
    }
    ///
    #[payable]
    pub fn deliver(&mut self, messages: Vec<Any>) {
        assert!(
            env::attached_deposit() >= utils::MINIMUM_DEPOSIT_FOR_DELEVER_MSG,
            "Need to attach at least 0.1 NEAR to cover the possible storage cost."
        );
        let used_bytes = env::storage_usage();
        // Deliver messages to `ibc-rs`
        let near_ibc_store = self.near_ibc_store.get().unwrap();

        let mut router_context = NearRouterContext::new(near_ibc_store);

        let (events, logs, errors) = messages.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut events, mut logs, mut errors), msg| {
                match ibc::core::ics26_routing::handler::deliver(&mut router_context, msg) {
                    Ok(MsgReceipt {
                        events: temp_events,
                        log: temp_logs,
                    }) => {
                        events.extend(temp_events);
                        logs.extend(temp_logs);
                    }
                    Err(e) => errors.push(e),
                }
                (events, logs, errors)
            },
        );
        self.near_ibc_store.set(&router_context.near_ibc_store);

        log!("near ibc deliver logs: {:?}", logs);
        log!("near ibc deliver errors: {:?}", errors);
        for event in &events {
            event.emit();
        }
        // Save the IBC events history.
        let raw_ibc_events = events.try_to_vec().unwrap();
        self.ibc_events_history
            .push_back((env::block_height(), raw_ibc_events));
        // Refund unused deposit.
        utils::refund_deposit(used_bytes, env::attached_deposit());
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
        ext_token_factory::ext(utils::get_token_factory_contract_id())
            .with_attached_deposit(env::attached_deposit())
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
        utils::refund_deposit(used_bytes, env::attached_deposit() - minimum_deposit)
    }
    /// Set the max length of the IBC events history queue.
    ///
    /// Only the governance account can call this function.
    pub fn set_max_length_of_ibc_events_history(&mut self, max_length: u64) {
        self.assert_governance();
        self.ibc_events_history.set_max_length(max_length);
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
        ext_escrow_factory::ext(utils::get_escrow_factory_contract_id())
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(
                utils::GAS_FOR_COMPLEX_FUNCTION_CALL - utils::GAS_FOR_SIMPLE_FUNCTION_CALL,
            )
            .with_unused_gas_weight(0)
            .create_escrow(ChannelId::from_str(channel_id.as_str()).unwrap());
        utils::refund_deposit(used_bytes, env::attached_deposit() - minimum_deposit);
    }
    /// Register the given token contract for the given channel.
    ///
    /// Only the governance account can call this function.
    pub fn register_asset_for_channel_escrow(
        &mut self,
        channel_id: String,
        denom: String,
        token_contract: AccountId,
    ) {
        self.assert_governance();
        let escrow_account_id =
            format!("{}.{}", channel_id, utils::get_escrow_factory_contract_id());
        ext_channel_escrow::ext(AccountId::from_str(escrow_account_id.as_str()).unwrap())
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
            .with_unused_gas_weight(0)
            .register_asset(denom, token_contract);
    }
}

utils::impl_storage_check_and_refund!(Contract);

#[near_bindgen]
impl TransferRequestHandler for Contract {
    fn process_transfer_request(&mut self, transfer_request: Ics20TransferRequest) {
        utils::assert_sub_account();
        let mut output = HandlerOutput::<()>::builder();
        if let Err(e) = send_transfer(
            &mut TransferModule(),
            &mut output,
            MsgTransfer {
                port_on_a: PortId::from_str(transfer_request.port_on_a.as_str()).unwrap(),
                chan_on_a: ChannelId::from_str(transfer_request.chan_on_a.as_str()).unwrap(),
                token: TransferringCoins {
                    trace_path: transfer_request.token_trace_path.clone(),
                    base_denom: transfer_request.token_denom.clone(),
                    amount: transfer_request.amount.0.to_string(),
                },
                sender: Signer::from_str(transfer_request.sender.as_str()).unwrap(),
                receiver: Signer::from_str(transfer_request.receiver.as_str()).unwrap(),
                timeout_height_on_b: TimeoutHeight::At(
                    Height::new(0, env::block_height() + 1000).unwrap(),
                ),
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
        let events = output.with_result(()).events;
        for event in &events {
            event.emit();
        }
        // Save the IBC events history.
        let raw_ibc_events = events.try_to_vec().unwrap();
        self.ibc_events_history
            .push_back((env::block_height(), raw_ibc_events));
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
