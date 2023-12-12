use crate::{
    collections::IndexedAscendingLookupQueue, prelude::*, types::ProcessingResult, StorageKey,
};
use core::fmt::{Debug, Formatter};
use ibc::core::{
    client::types::Height,
    handler::types::events::IbcEvent,
    host::types::{
        identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence},
        path::{
            AckPath, ClientConnectionPath, ClientConsensusStatePath, ClientStatePath,
            CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
        },
    },
};
use itertools::Itertools;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, log,
    store::{LookupMap, UnorderedMap, UnorderedSet},
};
use serde::{Deserialize, Serialize};

pub type NearTimeStamp = u64;
pub type HostHeight = Height;

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct NearIbcStore {
    /// The client ids of the clients.
    pub client_id_set: UnorderedSet<ClientId>,
    pub client_counter: u64,
    pub client_processed_times: LookupMap<ClientId, UnorderedMap<Height, NearTimeStamp>>,
    pub client_processed_heights: LookupMap<ClientId, UnorderedMap<Height, HostHeight>>,
    /// This collection contains the heights corresponding to all consensus states of
    /// all clients stored in the contract.
    pub client_consensus_state_height_sets: LookupMap<ClientId, UnorderedSet<Height>>,
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
        let store = near_sdk::env::storage_read(&borsh::to_vec(&StorageKey::NearIbcStore).unwrap())
            .unwrap();
        let store = NearIbcStore::try_from_slice(&store).unwrap();
        store
    }
    ///
    fn set_near_ibc_store(store: &NearIbcStore) {
        let store = borsh::to_vec(&store).unwrap();
        near_sdk::env::storage_write(&borsh::to_vec(&StorageKey::NearIbcStore).unwrap(), &store);
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
            queue.flush();
        }
        self.client_processed_heights.remove(client_id);
        self.client_processed_heights.flush();
        if let Some(queue) = self.client_processed_times.get_mut(client_id) {
            queue.clear();
            queue.flush();
        }
        self.client_processed_times.remove(client_id);
        self.client_processed_times.flush();
        env::storage_remove(
            &ClientConnectionPath::new(client_id)
                .to_string()
                .into_bytes(),
        );
        self.client_consensus_state_height_sets
            .get(client_id)
            .map(|heights| {
                heights.iter().for_each(|height| {
                    env::storage_remove(
                        &ClientConsensusStatePath::new(
                            client_id.clone(),
                            height.revision_number(),
                            height.revision_height(),
                        )
                        .to_string()
                        .into_bytes(),
                    );
                });
            });
        self.client_consensus_state_height_sets.remove(client_id);
        self.client_consensus_state_height_sets.flush();
        env::storage_remove(&ClientStatePath::new(client_id).to_string().into_bytes());
        self.client_id_set.remove(client_id);
        self.client_id_set.flush();
        log!("Client '{}' has been removed.", client_id);
    }
    ///
    pub fn remove_connection(&mut self, connection_id: &ConnectionId) {
        env::storage_remove(&ConnectionPath::new(&connection_id).to_string().into_bytes());
        self.connection_id_set.remove(connection_id);
        self.connection_id_set.flush();
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
        self.port_channel_id_set.flush();
        log!(
            "Channel '{}/{}' has been removed.",
            port_channel_id.0,
            port_channel_id.1
        );
    }
    ///
    pub fn clear_consensus_state_by(
        &mut self,
        client_id: &ClientId,
        lt_height: Option<&Height>,
    ) -> ProcessingResult {
        let max_gas = env::prepaid_gas().saturating_mul(2).saturating_div(5);
        let height_set = self
            .client_consensus_state_height_sets
            .get_mut(client_id)
            .unwrap();
        let height_iter: Box<dyn Iterator<Item = &Height>> = if lt_height.is_some() {
            Box::new(height_set.iter().sorted())
        } else {
            Box::new(height_set.iter())
        };

        let mut result = ProcessingResult::Ok;
        let mut need_remove_heights = vec![];
        for height in height_iter {
            if lt_height.is_some() && height.ge(lt_height.unwrap()) {
                break;
            }
            env::storage_remove(
                &ClientConsensusStatePath::new(
                    client_id.clone(),
                    height.revision_number(),
                    height.revision_height(),
                )
                .to_string()
                .into_bytes(),
            );
            need_remove_heights.push(height.clone());

            if env::used_gas() >= max_gas {
                result = ProcessingResult::NeedMoreGas;
                break;
            }
        }

        // It is why max_gas is 2/5 prepaid_gas
        for height in need_remove_heights {
            height_set.remove(&height);
        }
        self.client_consensus_state_height_sets.flush();
        result
    }
    ///
    pub fn clear_counters(&mut self) {
        self.client_counter = 0;
        self.connection_counter = 0;
        self.channel_counter = 0;
    }
    ///
    pub fn clear_ibc_events_history(&mut self) -> ProcessingResult {
        self.ibc_events_history.clear()
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

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct NearEd25519Verifier;

impl tendermint::crypto::signature::Verifier for NearEd25519Verifier {
    fn verify(
        pubkey: tendermint::PublicKey,
        msg: &[u8],
        signature: &tendermint::Signature,
    ) -> Result<(), tendermint::crypto::signature::Error> {
        if env::ed25519_verify(
            signature
                .as_bytes()
                .try_into()
                .map_err(|_| tendermint::crypto::signature::Error::MalformedSignature)?,
            msg,
            &pubkey
                .to_bytes()
                .try_into()
                .map_err(|_| tendermint::crypto::signature::Error::MalformedPublicKey)?,
        ) {
            Ok(())
        } else {
            Err(tendermint::crypto::signature::Error::VerificationFailed)
        }
    }
}
