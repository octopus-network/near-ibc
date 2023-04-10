#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::context::{IbcContext, NearIbcStore};
use crate::events::EventEmit;
use crate::link_map::KeySortLinkMap;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::core::ics26_routing::handler::MsgReceipt;
use ibc::events::IbcEvent;
use ibc_proto::google::protobuf::{Any, Duration};
use itertools::Itertools;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, log, AccountId};
use near_sdk::{near_bindgen, BorshStorageKey, PanicOnDefault};

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
    // todo if need LazyOption?
    near_ibc_store: NearIbcStore,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn init() -> Self {
        Self {
            near_ibc_store: NearIbcStore {
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
            },
        }
    }

    pub(crate) fn build_ibc_context(&mut self) -> IbcContext {
        IbcContext {
            near_ibc_store: &mut self.near_ibc_store,
            router: Default::default(),
        }
    }

    pub fn deliver(&mut self, messages: Vec<Any>) {
        let mut ibc_context = IbcContext {
            near_ibc_store: &mut self.near_ibc_store,
            router: Default::default(),
        };

        let (events, logs, errors) = messages.into_iter().fold(
            (vec![], vec![], vec![]),
            |(mut events, mut logs, mut errors), msg| {
                match ibc::core::ics26_routing::handler::deliver(&mut ibc_context, msg) {
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

        log!("near ibc deliver logs: {:?}", logs);
        log!("near ibc deliver errors: {:?}", errors);
        for event in events {
            event.emit();
        }
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
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
}
