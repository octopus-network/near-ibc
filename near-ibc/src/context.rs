use crate::{
    ibc_impl::{
        applications::transfer::TransferModule,
        core::{
            host::type_define::{IbcHostHeight, NearTimeStamp, RawClientState, RawConsensusState},
            routing::{NearRouter, NearRouterBuilder},
        },
    },
    indexed_lookup_queue::IndexedLookupQueue,
    StorageKey,
};
use core::fmt::{Debug, Formatter};
use ibc::{
    applications::transfer,
    core::{
        ics02_client::client_type::ClientType,
        ics03_connection::connection::ConnectionEnd,
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            packet::{Receipt, Sequence},
        },
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
        ics26_routing::context::{ModuleId, RouterBuilder},
    },
    Height,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    store::{LookupMap, UnorderedMap, Vector},
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearIbcStore {
    pub client_types: LookupMap<ClientId, ClientType>,
    pub client_states: UnorderedMap<ClientId, RawClientState>,
    pub consensus_states: LookupMap<ClientId, IndexedLookupQueue<Height, RawConsensusState>>,
    pub client_processed_times: LookupMap<ClientId, IndexedLookupQueue<Height, NearTimeStamp>>,
    pub client_processed_heights: LookupMap<ClientId, IndexedLookupQueue<Height, IbcHostHeight>>,
    pub client_ids_counter: u64,
    pub client_connections: LookupMap<ClientId, Vector<ConnectionId>>,
    pub connections: UnorderedMap<ConnectionId, ConnectionEnd>,
    pub connection_ids_counter: u64,
    pub port_to_module: LookupMap<PortId, ModuleId>,
    pub connection_channels: LookupMap<ConnectionId, Vector<(PortId, ChannelId)>>,
    pub channel_ids_counter: u64,
    pub channels: UnorderedMap<(PortId, ChannelId), ChannelEnd>,
    pub next_sequence_send: LookupMap<(PortId, ChannelId), Sequence>,
    pub next_sequence_recv: LookupMap<(PortId, ChannelId), Sequence>,
    pub next_sequence_ack: LookupMap<(PortId, ChannelId), Sequence>,
    pub packet_receipts: LookupMap<(PortId, ChannelId), IndexedLookupQueue<Sequence, Receipt>>,
    pub packet_acknowledgements:
        LookupMap<(PortId, ChannelId), IndexedLookupQueue<Sequence, AcknowledgementCommitment>>,
    pub packet_commitments:
        LookupMap<(PortId, ChannelId), IndexedLookupQueue<Sequence, PacketCommitment>>,
}

pub trait NearIbcStoreHost {
    ///
    fn get_near_ibc_store() -> NearIbcStore {
        let store =
            near_sdk::env::storage_read(&StorageKey::NearIbcStore.try_to_vec().unwrap()).unwrap();
        let store = NearIbcStore::try_from_slice(&store).unwrap();
        store
    }
    ///
    fn set_near_ibc_store(store: &NearIbcStore) {
        let store = store.try_to_vec().unwrap();
        near_sdk::env::storage_write(b"ibc_store", &store);
    }
}

pub struct NearRouterContext {
    pub near_ibc_store: NearIbcStore,
    pub router: NearRouter,
}

impl NearRouterContext {
    pub fn new(store: NearIbcStore) -> Self {
        let router = NearRouterBuilder::default()
            .add_route(transfer::MODULE_ID_STR.parse().unwrap(), TransferModule()) // register transfer Module
            .unwrap()
            .build();

        Self {
            near_ibc_store: store,
            router,
        }
    }
}

impl Debug for NearIbcStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "NearIbcStore {{ ... }}")
    }
}

impl NearIbcStore {
    ///
    pub fn remove_client(&mut self, client_id: &ClientId) {
        if let Some(vector) = self.client_connections.get_mut(client_id) {
            vector.clear();
        }
        self.client_connections.remove(client_id);
        if let Some(queue) = self.client_processed_heights.get_mut(client_id) {
            queue.clear();
        }
        self.client_processed_heights.remove(client_id);
        if let Some(queue) = self.client_processed_times.get_mut(client_id) {
            queue.clear();
        }
        self.client_processed_times.remove(client_id);
        if let Some(queue) = self.consensus_states.get_mut(client_id) {
            queue.clear();
        }
        self.consensus_states.remove(client_id);
        self.client_states.remove(client_id);
        self.client_types.remove(client_id);
    }
    ///
    pub fn remove_connection(&mut self, connection_id: &ConnectionId) {
        if let Some(vector) = self.connection_channels.get_mut(connection_id) {
            vector.clear();
        }
        self.connection_channels.remove(connection_id);
        self.connections.remove(connection_id);
    }
    ///
    pub fn remove_channel(&mut self, channel_end: &(PortId, ChannelId)) {
        if let Some(queue) = self.packet_receipts.get_mut(channel_end) {
            queue.clear();
        }
        self.packet_receipts.remove(channel_end);
        if let Some(queue) = self.packet_commitments.get_mut(channel_end) {
            queue.clear();
        }
        self.packet_commitments.remove(channel_end);
        if let Some(queue) = self.packet_acknowledgements.get_mut(channel_end) {
            queue.clear();
        }
        self.packet_acknowledgements.remove(channel_end);
        self.next_sequence_send.remove(channel_end);
        self.next_sequence_recv.remove(channel_end);
        self.next_sequence_ack.remove(channel_end);
        self.channels.remove(channel_end);
    }
    ///
    pub fn clear_counters(&mut self) {
        self.client_ids_counter = 0;
        self.connection_ids_counter = 0;
        self.channel_ids_counter = 0;
    }
}
