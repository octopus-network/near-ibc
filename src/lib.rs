#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use core::str::FromStr;

use crate::{
    context::{NearIbcStore, NearRouterContext},
    events::EventEmit,
    link_map::KeySortLinkMap,
};
use ibc::{
    applications::transfer,
    core::{
        ics24_host::identifier::{ClientId, PortId},
        ics26_routing::context::RouterBuilder,
        ics26_routing::handler::MsgReceipt,
    },
    events::IbcEvent,
};
use ibc_impl::{applications::transfer::TransferModule, core::routing::NearRouterBuilder};
use ibc_proto::google::protobuf::{Any, Duration};
use itertools::Itertools;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{LazyOption, LookupMap, UnorderedMap},
    env,
    json_types::Base64VecU8,
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json, AccountId, BorshStorageKey, PanicOnDefault,
};

pub mod context;
pub mod events;
pub mod ibc_impl;
pub mod interfaces;
pub mod link_map;
pub mod types;
pub mod viewer;

pub const DEFAULT_COMMITMENT_PREFIX: &str = "ibc";

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
                    client_state: UnorderedMap::new(StorageKey::ClientStates),
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
                    packet_receipt: LookupMap::new(StorageKey::PacketReceipt),
                    packet_acknowledgement: LookupMap::new(StorageKey::PacketAcknowledgement),
                    port_to_module: LookupMap::new(StorageKey::PortToModule),
                    packet_commitment: LookupMap::new(StorageKey::PacketCommitment),
                }),
            ),
        }
    }

    pub fn deliver(&mut self, messages: Vec<Any>) {
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
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    ClientTypes,
    ClientStates,
    ConsensusStates,
    ConsensusStatesKey { client_id: ClientId },
    ConsensusStatesLink { client_id: ClientId },
    ClientProcessedTimes,
    ClientProcessedHeights,
    ClientConnections,
    Connections,
    ConnectionChannels,
    Channels,
    NextSequenceSend,
    NextSequenceRecv,
    NextSequenceAck,
    PacketReceipt,
    PacketAcknowledgement,
    PortToModule,
    PacketCommitment,
    NearIbcStore,
}

#[near_bindgen]
impl Contract {
    pub fn clear_near_ibc_store(&mut self) {
        near_sdk::assert_self();
        assert!(
            !env::current_account_id().to_string().ends_with(".near"),
            "This function can not be called on mainnet."
        );
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();

        for client_id in near_ibc_store.client_state.keys() {
            near_ibc_store.client_types.remove(&client_id);
            near_ibc_store.client_connections.remove(&client_id);
            near_ibc_store.consensus_states.remove(&client_id);
            near_ibc_store.client_connections.remove(&client_id);
        }
        near_ibc_store.client_state.clear();
        for connection_id in near_ibc_store.connections.keys() {
            near_ibc_store.connection_channels.remove(&connection_id);
        }
        near_ibc_store.connections.clear();
        near_ibc_store.channel_ids_counter = 0;
        for channel_id in near_ibc_store.channels.keys() {
            near_ibc_store.next_sequence_send.remove(&channel_id);
            near_ibc_store.next_sequence_recv.remove(&channel_id);
            near_ibc_store.next_sequence_ack.remove(&channel_id);
        }
        near_ibc_store.channels.clear();
        let port_id = PortId::from_str("transfer").unwrap();
        near_ibc_store.port_to_module.remove(&port_id);

        self.near_ibc_store.set(&near_ibc_store);
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
