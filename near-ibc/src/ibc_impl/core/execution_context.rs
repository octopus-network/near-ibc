use super::{client_state::AnyClientState, consensus_state::AnyConsensusState};
use crate::{
    collections::{
        IndexedAscendingLookupQueue, IndexedAscendingQueueViewer, IndexedAscendingSimpleQueue,
    },
    context::NearIbcStore,
    events::EventEmitter,
    prelude::*,
    StorageKey,
};
use core::fmt::Debug;
use ibc::{
    core::{
        events::IbcEvent,
        ics02_client::ClientExecutionContext,
        ics03_connection::{connection::ConnectionEnd, error::ConnectionError},
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            packet::{Receipt, Sequence},
        },
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::{
                AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath,
                ClientStatePath, CommitmentPath, ConnectionPath, ReceiptPath, SeqAckPath,
                SeqRecvPath, SeqSendPath,
            },
        },
        timestamp::Timestamp,
        ContextError, ExecutionContext, ValidationContext,
    },
    Height,
};
use ibc_proto::protobuf::Protobuf;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, log,
    store::{LookupMap, UnorderedSet},
};

impl ClientExecutionContext for NearIbcStore {
    type ClientValidationContext = Self;

    type AnyClientState = AnyClientState;

    type AnyConsensusState = AnyConsensusState;

    fn store_client_state(
        &mut self,
        client_state_path: ClientStatePath,
        client_state: Self::AnyClientState,
    ) -> Result<(), ContextError> {
        log!(
            "store_client_state - path: {}, client_state: {:?}",
            client_state_path,
            client_state
        );
        let data = client_state.encode_vec();
        let key = client_state_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        //
        self.client_id_set.insert(client_state_path.0.clone());
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ClientConsensusStatePath,
        consensus_state: Self::AnyConsensusState,
    ) -> Result<(), ContextError> {
        log!(
            "store_consensus_state - path: {}, consensus_state: {:?}",
            consensus_state_path,
            consensus_state
        );
        let data = Protobuf::encode_vec(&consensus_state);
        let key = consensus_state_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        //
        if !self
            .client_consensus_state_height_sets
            .contains_key(&consensus_state_path.client_id)
        {
            self.client_consensus_state_height_sets.insert(
                consensus_state_path.client_id.clone(),
                IndexedAscendingSimpleQueue::new(
                    StorageKey::ClientConsensusStateHeightSet {
                        client_id: consensus_state_path.client_id.clone(),
                    },
                    u64::MAX,
                ),
            );
        }
        self.client_consensus_state_height_sets
            .get_mut(&consensus_state_path.client_id)
            .map(|heights| {
                heights.push_back(
                    Height::new(consensus_state_path.epoch, consensus_state_path.height).unwrap(),
                )
            });
        Ok(())
    }
}

impl ExecutionContext for NearIbcStore {
    fn increase_client_counter(&mut self) -> Result<(), ContextError> {
        self.client_counter += 1;
        log!("client_counter has increased to: {}", self.client_counter);
        Ok(())
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        log!(
            "store_update_time - client_id: {}, height: {}, timestamp: {}",
            client_id,
            height,
            timestamp
        );
        if !self.client_processed_times.contains_key(&client_id) {
            self.client_processed_times.insert(
                client_id.clone(),
                IndexedAscendingLookupQueue::new(
                    StorageKey::ClientProcessedTimesIndex {
                        client_id: client_id.clone(),
                    },
                    StorageKey::ClientProcessedTimesKey {
                        client_id: client_id.clone(),
                    },
                    u64::MAX,
                ),
            );
        }
        self.client_processed_times
            .get_mut(&client_id)
            .and_then(|processed_times| {
                Some(processed_times.push_back((height, timestamp.nanoseconds())))
            });
        Ok(())
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError> {
        log!(
            "store_update_height - client_id: {}, height: {}, host_height: {}",
            client_id,
            height,
            host_height
        );
        if !self.client_processed_heights.contains_key(&client_id) {
            self.client_processed_heights.insert(
                client_id.clone(),
                IndexedAscendingLookupQueue::new(
                    StorageKey::ClientProcessedHeightsIndex {
                        client_id: client_id.clone(),
                    },
                    StorageKey::ClientProcessedHeightsKey {
                        client_id: client_id.clone(),
                    },
                    u64::MAX,
                ),
            );
        }
        self.client_processed_heights
            .get_mut(&client_id)
            .and_then(|processed_heights| Some(processed_heights.push_back((height, host_height))));
        Ok(())
    }

    fn store_connection(
        &mut self,
        connection_path: &ConnectionPath,
        connection_end: ConnectionEnd,
    ) -> Result<(), ContextError> {
        log!(
            "store_connection: path: {}, connection_end: {:?}",
            connection_path,
            connection_end
        );
        let data = connection_end.encode_vec();
        let key = connection_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        //
        self.connection_id_set.insert(connection_path.0.clone());
        Ok(())
    }

    fn store_connection_to_client(
        &mut self,
        client_connection_path: &ClientConnectionPath,
        conn_id: ConnectionId,
    ) -> Result<(), ContextError> {
        log!(
            "store_connection_to_client: path: {}, connection_id: {:?}",
            client_connection_path,
            conn_id
        );
        #[derive(BorshDeserialize, BorshSerialize, Debug)]
        struct ConnectionIds(pub Vec<ConnectionId>);
        let key = client_connection_path.to_string().into_bytes();
        let data = if env::storage_has_key(&key) {
            let mut connection_ids =
                ConnectionIds::try_from_slice(&env::storage_read(&key).unwrap()).map_err(|e| {
                    ContextError::ConnectionError(ConnectionError::Other {
                        description: format!("ConnectionIds decoding error: {:?}", e),
                    })
                })?;
            connection_ids.0.push(conn_id);
            connection_ids.try_to_vec().map_err(|e| {
                ContextError::ConnectionError(ConnectionError::Other {
                    description: format!("ConnectionIds encoding error: {:?}", e),
                })
            })?
        } else {
            let connection_ids = ConnectionIds(vec![conn_id]);
            connection_ids.try_to_vec().map_err(|e| {
                ContextError::ConnectionError(ConnectionError::Other {
                    description: format!("ConnectionIds encoding error: {:?}", e),
                })
            })?
        };
        env::storage_write(&key, &data);
        Ok(())
    }

    fn increase_connection_counter(&mut self) -> Result<(), ContextError> {
        self.connection_counter += 1;
        log!(
            "connection_counter has increased to: {}",
            self.connection_counter
        );
        Ok(())
    }

    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        log!(
            "store_packet_commitment: path: {}, commitment: {:?}",
            commitment_path,
            commitment
        );
        let data = commitment.into_vec();
        let key = commitment_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        //
        record_packet_sequence(
            &mut self.packet_commitment_sequence_sets,
            StorageKey::PacketCommitmentSequenceSet {
                port_id: commitment_path.port_id.clone(),
                channel_id: commitment_path.channel_id.clone(),
            },
            &commitment_path.port_id,
            &commitment_path.channel_id,
            &commitment_path.sequence,
        );
        Ok(())
    }

    fn delete_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
    ) -> Result<(), ContextError> {
        log!("delete_packet_commitment: path: {}", commitment_path);
        let key = commitment_path.to_string().into_bytes();
        env::storage_remove(&key);
        //
        self.packet_commitment_sequence_sets
            .get_mut(&(
                commitment_path.port_id.clone(),
                commitment_path.channel_id.clone(),
            ))
            .map(|sequences| {
                sequences.remove(&commitment_path.sequence);
            });
        Ok(())
    }

    fn store_packet_receipt(
        &mut self,
        receipt_path: &ReceiptPath,
        receipt: Receipt,
    ) -> Result<(), ContextError> {
        log!(
            "store_packet_receipt: path: {}, receipt: {:?}",
            receipt_path,
            receipt
        );
        let data = receipt.try_to_vec().unwrap();
        let key = receipt_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        //
        record_packet_sequence(
            &mut self.packet_receipt_sequence_sets,
            StorageKey::PacketReceiptSequenceSet {
                port_id: receipt_path.port_id.clone(),
                channel_id: receipt_path.channel_id.clone(),
            },
            &receipt_path.port_id,
            &receipt_path.channel_id,
            &receipt_path.sequence,
        );
        Ok(())
    }

    fn store_packet_acknowledgement(
        &mut self,
        ack_path: &AckPath,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError> {
        log!(
            "store_packet_acknowledgement: path: {}, ack_commitment: {:?}",
            ack_path,
            ack_commitment
        );
        let data = ack_commitment.into_vec();
        let key = ack_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        //
        record_packet_sequence(
            &mut self.packet_acknowledgement_sequence_sets,
            StorageKey::PacketAcknowledgementSequenceSet {
                port_id: ack_path.port_id.clone(),
                channel_id: ack_path.channel_id.clone(),
            },
            &ack_path.port_id,
            &ack_path.channel_id,
            &ack_path.sequence,
        );
        Ok(())
    }

    fn delete_packet_acknowledgement(&mut self, ack_path: &AckPath) -> Result<(), ContextError> {
        log!("delete_packet_acknowledgement: path: {}", ack_path,);
        let key = ack_path.to_string().into_bytes();
        env::storage_remove(&key);
        //
        self.packet_acknowledgement_sequence_sets
            .get_mut(&(ack_path.port_id.clone(), ack_path.channel_id.clone()))
            .map(|sequences| {
                sequences.remove(&ack_path.sequence);
            });
        Ok(())
    }

    fn store_channel(
        &mut self,
        channel_end_path: &ChannelEndPath,
        channel_end: ChannelEnd,
    ) -> Result<(), ContextError> {
        log!(
            "store_channel: path: {}, channel_end: {:?}",
            channel_end_path,
            channel_end
        );
        let data = channel_end.encode_vec();
        let key = channel_end_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        //
        self.port_channel_id_set
            .insert((channel_end_path.0.clone(), channel_end_path.1.clone()));
        Ok(())
    }

    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        log!(
            "store_next_sequence_send: path: {}, seq: {:?}",
            seq_send_path,
            seq
        );
        let data = seq.try_to_vec().unwrap();
        let key = seq_send_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        Ok(())
    }

    fn store_next_sequence_recv(
        &mut self,
        seq_recv_path: &SeqRecvPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        log!(
            "store_next_sequence_recv: path: {}, seq: {:?}",
            seq_recv_path,
            seq
        );
        let data = seq.try_to_vec().unwrap();
        let key = seq_recv_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        Ok(())
    }

    fn store_next_sequence_ack(
        &mut self,
        seq_ack_path: &SeqAckPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        log!(
            "store_next_sequence_ack: path: {}, seq: {:?}",
            seq_ack_path,
            seq
        );
        let data = seq.try_to_vec().unwrap();
        let key = seq_ack_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        Ok(())
    }

    fn increase_channel_counter(&mut self) -> Result<(), ContextError> {
        self.channel_counter += 1;
        log!("channel_counter has increased to: {}", self.channel_counter);
        Ok(())
    }

    fn emit_ibc_event(&mut self, event: IbcEvent) -> Result<(), ContextError> {
        let height = self.host_height().unwrap();
        if self.ibc_events_history.contains_key(&height) {
            self.ibc_events_history
                .get_value_by_key_mut(&height)
                .map(|events| events.push(event.clone()));
        } else {
            self.ibc_events_history
                .push_back((height, vec![event.clone()]));
        }
        event.emit();
        Ok(())
    }

    fn log_message(&mut self, message: String) -> Result<(), ContextError> {
        log!("{}", message);
        Ok(())
    }

    fn get_client_execution_context(&mut self) -> &mut Self::E {
        self
    }
}

fn record_packet_sequence(
    lookup_sets: &mut LookupMap<(PortId, ChannelId), UnorderedSet<Sequence>>,
    storage_key: StorageKey,
    port_id: &PortId,
    channel_id: &ChannelId,
    sequence: &Sequence,
) {
    let key = (port_id.clone(), channel_id.clone());
    if !lookup_sets.contains_key(&key) {
        lookup_sets.insert(key.clone(), UnorderedSet::new(storage_key));
    }
    lookup_sets.get_mut(&key).map(|sequences| {
        sequences.insert(sequence.clone());
    });
}
