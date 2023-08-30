use crate::{
    collections::{
        IndexedAscendingLookupQueue, IndexedAscendingQueueViewer, IndexedAscendingSimpleQueue,
    },
    module_holder::ModuleHolder,
    prelude::*,
    StorageKey,
};
use core::fmt::{Debug, Formatter};
use ibc::{
    core::{
        events::IbcEvent,
        ics04_channel::packet::Sequence,
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::{
                AckPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
                CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
            },
        },
    },
    Height,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, log,
    store::{LookupMap, UnorderedSet},
};

pub type NearTimeStamp = u64;
pub type HostHeight = Height;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearIbcStore {
    /// To support the mutable borrow in `Router::get_route_mut`.
    pub module_holder: ModuleHolder,
    /// The client ids of the clients.
    pub client_id_set: UnorderedSet<ClientId>,
    pub client_counter: u64,
    pub client_processed_times:
        LookupMap<ClientId, IndexedAscendingLookupQueue<Height, NearTimeStamp>>,
    pub client_processed_heights:
        LookupMap<ClientId, IndexedAscendingLookupQueue<Height, HostHeight>>,
    /// This collection contains the heights corresponding to all consensus states of
    /// all clients stored in the contract.
    pub client_consensus_state_height_sets:
        LookupMap<ClientId, IndexedAscendingSimpleQueue<Height>>,
    /// The connection ids of the connections.
    pub connection_id_set: UnorderedSet<ConnectionId>,
    pub connection_counter: u64,
    /// The port and channel id tuples of the channels.
    pub port_channel_id_set: UnorderedSet<(PortId, ChannelId)>,
    pub channel_counter: u64,
    /// The sequence numbers of the packet commitments.
    pub packet_commitment_sequence_sets: LookupMap<(PortId, ChannelId), UnorderedSet<Sequence>>,
    /// The sequence numbers of the packet receipts.
    pub packet_receipt_sequence_sets: LookupMap<(PortId, ChannelId), UnorderedSet<Sequence>>,
    /// The sequence numbers of the packet acknowledgements.
    pub packet_acknowledgement_sequence_sets:
        LookupMap<(PortId, ChannelId), UnorderedSet<Sequence>>,
    /// The history of IBC events.
    pub ibc_events_history: IndexedAscendingLookupQueue<Height, Vec<IbcEvent>>,
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
        near_sdk::env::storage_write(&StorageKey::NearIbcStore.try_to_vec().unwrap(), &store);
    }
}

impl Debug for NearIbcStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "NearIbcStore {{ ... }}")
    }
}

impl NearIbcStore {
    ///
    pub fn new() -> Self {
        Self {
            module_holder: ModuleHolder::new(),
            client_id_set: UnorderedSet::new(StorageKey::ClientIdSet),
            client_counter: 0,
            client_processed_times: LookupMap::new(StorageKey::ClientProcessedTimes),
            client_processed_heights: LookupMap::new(StorageKey::ClientProcessedHeights),
            client_consensus_state_height_sets: LookupMap::new(
                StorageKey::ClientConsensusStateHeightSets,
            ),
            connection_id_set: UnorderedSet::new(StorageKey::ConnectionIdSet),
            connection_counter: 0,
            port_channel_id_set: UnorderedSet::new(StorageKey::PortChannelIdSet),
            channel_counter: 0,
            packet_commitment_sequence_sets: LookupMap::new(
                StorageKey::PacketCommitmentSequenceSets,
            ),
            packet_receipt_sequence_sets: LookupMap::new(StorageKey::PacketReceiptSequenceSets),
            packet_acknowledgement_sequence_sets: LookupMap::new(
                StorageKey::PacketAcknowledgementSequenceSets,
            ),
            ibc_events_history: IndexedAscendingLookupQueue::new(
                StorageKey::IbcEventsHistoryIndexMap,
                StorageKey::IbcEventsHistoryValueMap,
                u64::MAX,
            ),
        }
    }
    ///
    pub fn remove_client(&mut self, client_id: &ClientId) {
        if let Some(queue) = self.client_processed_heights.get_mut(client_id) {
            queue.clear();
        }
        self.client_processed_heights.remove(client_id);
        if let Some(queue) = self.client_processed_times.get_mut(client_id) {
            queue.clear();
        }
        self.client_processed_times.remove(client_id);
        env::storage_remove(
            &ClientConnectionPath::new(client_id)
                .to_string()
                .into_bytes(),
        );
        self.client_consensus_state_height_sets
            .get(client_id)
            .map(|heights| {
                heights.keys().iter().for_each(|height| {
                    height.map(|height| {
                        env::storage_remove(
                            &ClientConsensusStatePath::new(client_id, height)
                                .to_string()
                                .into_bytes(),
                        )
                    });
                })
            });
        self.client_consensus_state_height_sets.remove(client_id);
        env::storage_remove(&ClientStatePath::new(client_id).to_string().into_bytes());
        self.client_id_set.remove(client_id);
        log!("Client '{}' has been removed.", client_id);
    }
    ///
    pub fn remove_connection(&mut self, connection_id: &ConnectionId) {
        env::storage_remove(&ConnectionPath::new(&connection_id).to_string().into_bytes());
        self.connection_id_set.remove(connection_id);
        log!("Connection '{}' has been removed.", connection_id);
    }
    ///
    pub fn remove_channel(&mut self, port_channel_id: &(PortId, ChannelId)) {
        self.packet_commitment_sequence_sets
            .get(port_channel_id)
            .map(|set| {
                set.iter().for_each(|sequence| {
                    env::storage_remove(
                        &CommitmentPath::new(&port_channel_id.0, &port_channel_id.1, *sequence)
                            .to_string()
                            .into_bytes(),
                    );
                })
            });
        self.packet_receipt_sequence_sets
            .get(port_channel_id)
            .map(|set| {
                set.iter().for_each(|sequence| {
                    env::storage_remove(
                        &ReceiptPath::new(&port_channel_id.0, &port_channel_id.1, *sequence)
                            .to_string()
                            .into_bytes(),
                    );
                });
            });
        self.packet_acknowledgement_sequence_sets
            .get(port_channel_id)
            .map(|set| {
                set.iter().for_each(|sequence| {
                    env::storage_remove(
                        &AckPath::new(&port_channel_id.0, &port_channel_id.1, *sequence)
                            .to_string()
                            .into_bytes(),
                    );
                });
            });
        env::storage_remove(
            &SeqSendPath(port_channel_id.0.clone(), port_channel_id.1.clone())
                .to_string()
                .into_bytes(),
        );
        env::storage_remove(
            &SeqRecvPath(port_channel_id.0.clone(), port_channel_id.1.clone())
                .to_string()
                .into_bytes(),
        );
        env::storage_remove(
            &SeqAckPath(port_channel_id.0.clone(), port_channel_id.1.clone())
                .to_string()
                .into_bytes(),
        );
        self.port_channel_id_set.remove(port_channel_id);
        log!(
            "Channel '{}/{}' has been removed.",
            port_channel_id.0,
            port_channel_id.1
        );
    }
    ///
    pub fn clear_counters(&mut self) {
        self.client_counter = 0;
        self.connection_counter = 0;
        self.channel_counter = 0;
    }
    ///
    pub fn clear_ibc_events_history(&mut self) {
        self.ibc_events_history.clear();
    }
    ///
    pub fn flush(&mut self) {
        self.client_id_set.flush();
        self.client_processed_heights.flush();
        self.client_processed_times.flush();
        self.client_consensus_state_height_sets.flush();
        self.connection_id_set.flush();
        self.port_channel_id_set.flush();
        self.packet_commitment_sequence_sets.flush();
        self.packet_receipt_sequence_sets.flush();
        self.packet_acknowledgement_sequence_sets.flush();
        self.ibc_events_history.flush();
    }
}
