#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use core::str::FromStr;

use crate::{
    context::{NearIbcStore, NearRouterContext},
    events::EventEmit,
    indexed_lookup_queue::IndexedLookupQueue,
};
use ibc::{
    applications::transfer,
    core::{
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
        ics26_routing::context::RouterBuilder,
        ics26_routing::handler::MsgReceipt,
    },
    events::IbcEvent,
};
use ibc_impl::{applications::transfer::TransferModule, core::routing::NearRouterBuilder};
use ibc_proto::google::protobuf::{Any, Duration};
use itertools::Itertools;
use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::{Base64VecU8, U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json,
    store::{LookupMap, UnorderedMap},
    AccountId, BorshStorageKey, Gas, PanicOnDefault, Promise,
};

pub mod context;
pub mod events;
pub mod ibc_impl;
pub mod indexed_lookup_queue;
pub mod viewer;

pub const DEFAULT_COMMITMENT_PREFIX: &str = "ibc";
/// As the `deliver` function may cause storage changes, the caller needs to attach some NEAR
/// to cover the storage cost. The minimum valid amount is 0.01 NEAR (for 1 kb storage).
const MINIMUM_ATTACHED_NEAR_FOR_DELEVER_MSG: u128 = 10_000_000_000_000_000_000_000;
/// Gas for calling `check_refund` function.
const GAS_FOR_CHECK_REFUND: Gas = Gas(15_000_000_000_000);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    near_ibc_store: LazyOption<NearIbcStore>,
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
        }
    }
    ///
    #[payable]
    pub fn deliver(&mut self, messages: Vec<Any>) {
        assert!(
            env::attached_deposit() >= MINIMUM_ATTACHED_NEAR_FOR_DELEVER_MSG,
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
        for event in events {
            event.emit();
        }
        // Check and fefund the unused attached deposit by a promise function call.
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub caller: AccountId,
            pub attached_deposit: U128,
            pub used_bytes: U64,
        }
        let args = Input {
            caller: env::predecessor_account_id(),
            attached_deposit: U128(env::attached_deposit()),
            used_bytes: U64(used_bytes),
        };
        let args =
            near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_MINT_FUNCTION");
        Promise::new(env::current_account_id()).function_call(
            "check_refund".to_string(),
            args,
            0,
            GAS_FOR_CHECK_REFUND,
        );
    }
    /// Check the storage usage and refund the unused attached deposit.
    pub fn check_refund(&mut self, caller: AccountId, attached_deposit: U128, used_bytes: U64) {
        assert_self();
        let mut refund_amount = attached_deposit.0;
        if env::storage_usage() > used_bytes.0 {
            log!(
                "near ibc deliver storage usage: {}",
                env::storage_usage() - used_bytes.0
            );
            let cost = env::storage_byte_cost() * (env::storage_usage() - used_bytes.0) as u128;
            if cost >= refund_amount {
                return;
            } else {
                refund_amount -= cost;
            }
        }
        Promise::new(caller).transfer(refund_amount);
    }
}

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
