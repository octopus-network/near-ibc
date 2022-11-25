#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::context::{IbcContext, NearIbcStore};
use crate::link_map::KeySortLinkMap;
use ibc_proto::google::protobuf::Duration;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, AccountId};
use near_sdk::{near_bindgen, BorshStorageKey, PanicOnDefault};

pub mod context;
pub mod ibc_impl;
pub mod link_map;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
struct Contract {
    // todo do we need LazyOption?
    ibc_store: NearIbcStore,
}

#[near_bindgen]
impl Contract {
    #[private]
    #[init]
    pub fn init() -> Self {
        Self {
            ibc_store: NearIbcStore {
                client_types: LookupMap::new(StorageKey::ClientTypes),
                client_state: LookupMap::new(StorageKey::ClientStates),
                consensus_states: KeySortLinkMap::new(StorageKey::ConsensusStates),
                client_processed_times: LookupMap::new(StorageKey::ClientProcessedTimes),
                client_processed_heights: LookupMap::new(StorageKey::ClientProcessedHeights),
                client_ids_counter: 0,
                client_connections: LookupMap::new(StorageKey::ClientConnections),
                connections: LookupMap::new(StorageKey::Connections),
                connection_ids_counter: 0,
                connection_channels: LookupMap::new(StorageKey::ConnectionChannels),
                channel_ids_counter: 0,
                channels: LookupMap::new(StorageKey::Channels),
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

    // todo confirm how to impl Any
    pub fn deliver(messages: Vec<ibc_support::Any>) {




    }
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    ClientTypes,
    ClientStates,
    ConsensusStates,
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
