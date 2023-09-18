use super::{client_state::AnyClientState, consensus_state::AnyConsensusState};
use crate::{context::NearIbcStore, prelude::*};
use core::{str::FromStr, time::Duration};
use ibc::{
    core::{
        ics02_client::error::ClientError,
        ics03_connection::{connection::ConnectionEnd, error::ConnectionError},
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            error::{ChannelError, PacketError},
            packet::{Receipt, Sequence},
        },
        ics23_commitment::commitment::CommitmentPrefix,
        ics24_host::{
            identifier::{ClientId, ConnectionId},
            path::{
                AckPath, ChannelEndPath, ClientConsensusStatePath, ClientStatePath, CommitmentPath,
                ConnectionPath, ReceiptPath, SeqAckPath, SeqRecvPath, SeqSendPath,
            },
        },
        timestamp::Timestamp,
        ContextError, ValidationContext,
    },
    Height, Signer,
};
use ibc_proto::{
    google::protobuf::Any,
    ibc::core::{
        channel::v1::Channel as RawChannelEnd, connection::v1::ConnectionEnd as RawConnectionEnd,
    },
    protobuf::Protobuf,
};
use near_sdk::{borsh::BorshDeserialize, env, AccountId};

/// Constants for commitment prefix generation.
/// This column id is used when storing Key-Value data from a contract on an `account_id`.
const CONTRACT_DATA: u8 = 9;
const ACCOUNT_DATA_SEPARATOR: u8 = b',';

impl ValidationContext for NearIbcStore {
    type ClientValidationContext = Self;

    type E = Self;

    type AnyConsensusState = AnyConsensusState;

    type AnyClientState = AnyClientState;

    fn client_state(&self, client_id: &ClientId) -> Result<Self::AnyClientState, ContextError> {
        let client_state_key = ClientStatePath(client_id.clone()).to_string().into_bytes();
        match env::storage_read(&client_state_key) {
            Some(data) => {
                let result: AnyClientState =
                    Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
                        description: format!("Decode ClientState failed: {:?}", e).to_string(),
                    })?;
                Ok(result)
            }
            None => Err(ContextError::ClientError(
                ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                },
            )),
        }
    }

    fn decode_client_state(&self, client_state: Any) -> Result<Self::AnyClientState, ContextError> {
        Ok(AnyClientState::try_from(client_state)?)
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        let consensus_state_key = client_cons_state_path.to_string().into_bytes();
        match env::storage_read(&consensus_state_key) {
            Some(data) => {
                let result: AnyConsensusState =
                    Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
                        description: format!("Decode ConsensusState failed: {:?}", e).to_string(),
                    })?;
                Ok(result)
            }
            None => Err(ContextError::ClientError(
                ClientError::ConsensusStateNotFound {
                    client_id: client_cons_state_path.client_id.clone(),
                    height: Height::new(
                        client_cons_state_path.epoch,
                        client_cons_state_path.height,
                    )?,
                },
            )),
        }
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        Height::new(0, env::block_height()).map_err(|e| ContextError::ClientError(e))
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        Timestamp::from_nanoseconds(env::block_timestamp())
            .map_err(|e| ContextError::ClientError(ClientError::InvalidPacketTimestamp(e)))
    }

    fn host_consensus_state(
        &self,
        _height: &Height,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        Err(ContextError::ClientError(ClientError::ClientSpecific {
            description: format!("The `host_consensus_state` is not supported on NEAR protocol."),
        }))
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        Ok(self.client_counter)
    }

    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        let path = ConnectionPath(conn_id.clone());
        let connection_end_key = path.to_string().into_bytes();
        match env::storage_read(&connection_end_key) {
            Some(data) => {
                let result: ConnectionEnd = Protobuf::<RawConnectionEnd>::decode_vec(&data)
                    .map_err(|e| ClientError::Other {
                        description: format!("Decode ConnectionEnd failed: {:?}", e).to_string(),
                    })?;
                Ok(result)
            }
            None => Err(ContextError::ConnectionError(
                ConnectionError::ConnectionNotFound {
                    connection_id: conn_id.clone(),
                },
            )),
        }
    }

    fn validate_self_client(
        &self,
        _client_state_of_host_on_counterparty: Any,
    ) -> Result<(), ContextError> {
        // As we can not validate the client state of self chain,
        // we return OK until we can find a way to do this.
        Ok(())
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        let mut prefix_bytes = vec![];
        prefix_bytes.push(CONTRACT_DATA);
        prefix_bytes.extend(env::current_account_id().as_bytes());
        prefix_bytes.push(ACCOUNT_DATA_SEPARATOR);
        CommitmentPrefix::try_from(prefix_bytes).unwrap()
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        Ok(self.connection_counter)
    }

    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        let channel_end_key = channel_end_path.to_string().into_bytes();
        match env::storage_read(&channel_end_key) {
            Some(data) => {
                let result: ChannelEnd =
                    Protobuf::<RawChannelEnd>::decode_vec(&data).map_err(|e| {
                        ClientError::Other {
                            description: format!("Decode ChannelEnd failed: {:?}", e).to_string(),
                        }
                    })?;
                Ok(result)
            }
            None => Err(ContextError::ChannelError(ChannelError::ChannelNotFound {
                port_id: channel_end_path.0.clone(),
                channel_id: channel_end_path.1.clone(),
            })),
        }
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        let seq_send_key = seq_send_path.to_string().into_bytes();
        match env::storage_read(&seq_send_key) {
            Some(data) => {
                let result = Sequence::try_from_slice(&data).map_err(|e| ClientError::Other {
                    description: format!("Decode Sequence failed: {:?}", e).to_string(),
                })?;
                Ok(result)
            }
            None => Err(ContextError::PacketError(PacketError::MissingNextSendSeq {
                port_id: seq_send_path.0.clone(),
                channel_id: seq_send_path.1.clone(),
            })),
        }
    }

    fn get_next_sequence_recv(
        &self,
        seq_recv_path: &SeqRecvPath,
    ) -> Result<Sequence, ContextError> {
        let seq_recv_key = seq_recv_path.to_string().into_bytes();
        match env::storage_read(&seq_recv_key) {
            Some(data) => {
                let result = Sequence::try_from_slice(&data).map_err(|e| ClientError::Other {
                    description: format!("Decode Sequence failed: {:?}", e).to_string(),
                })?;
                Ok(result)
            }
            None => Err(ContextError::PacketError(PacketError::MissingNextRecvSeq {
                port_id: seq_recv_path.0.clone(),
                channel_id: seq_recv_path.1.clone(),
            })),
        }
    }

    fn get_next_sequence_ack(&self, seq_ack_path: &SeqAckPath) -> Result<Sequence, ContextError> {
        let seq_ack_key = seq_ack_path.to_string().into_bytes();
        match env::storage_read(&seq_ack_key) {
            Some(data) => {
                let result = Sequence::try_from_slice(&data).map_err(|e| ClientError::Other {
                    description: format!("Decode Sequence failed: {:?}", e).to_string(),
                })?;
                Ok(result)
            }
            None => Err(ContextError::PacketError(PacketError::MissingNextAckSeq {
                port_id: seq_ack_path.0.clone(),
                channel_id: seq_ack_path.1.clone(),
            })),
        }
    }

    fn get_packet_commitment(
        &self,
        commitment_path: &CommitmentPath,
    ) -> Result<PacketCommitment, ContextError> {
        let commitment_key = commitment_path.to_string().into_bytes();
        match env::storage_read(&commitment_key) {
            Some(data) => Ok(PacketCommitment::from(data)),
            None => Err(ContextError::PacketError(
                PacketError::PacketCommitmentNotFound {
                    sequence: commitment_path.sequence,
                },
            )),
        }
    }

    fn get_packet_receipt(&self, receipt_path: &ReceiptPath) -> Result<Receipt, ContextError> {
        let receipt_key = receipt_path.to_string().into_bytes();
        match env::storage_read(&receipt_key) {
            Some(data) => {
                let result = Receipt::try_from_slice(&data).map_err(|e| ClientError::Other {
                    description: format!("Decode Receipt failed: {:?}", e).to_string(),
                })?;
                Ok(result)
            }
            None => Err(ContextError::PacketError(
                PacketError::PacketReceiptNotFound {
                    sequence: receipt_path.sequence,
                },
            )),
        }
    }

    fn get_packet_acknowledgement(
        &self,
        ack_path: &AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        let ack_key = ack_path.to_string().into_bytes();
        match env::storage_read(&ack_key) {
            Some(data) => Ok(AcknowledgementCommitment::from(data)),
            None => Err(ContextError::PacketError(
                PacketError::PacketAcknowledgementNotFound {
                    sequence: ack_path.sequence,
                },
            )),
        }
    }

    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError> {
        self.client_processed_times
            .get(client_id)
            .and_then(|processed_times| processed_times.get_value_by_key(height))
            .map(|ts| Timestamp::from_nanoseconds(*ts).unwrap())
            .ok_or_else(|| {
                ContextError::ClientError(ClientError::Other {
                    description: format!(
                        "Client update time not found. client_id: {}, height: {}",
                        client_id, height
                    ),
                })
            })
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError> {
        self.client_processed_heights
            .get(client_id)
            .and_then(|processed_heights| processed_heights.get_value_by_key(height))
            .map(|height: &Height| height.clone())
            .ok_or_else(|| {
                ContextError::ClientError(ClientError::Other {
                    description: format!(
                        "Client update height not found. client_id: {}, height: {}",
                        client_id, height
                    ),
                })
            })
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        Ok(self.channel_counter)
    }

    fn max_expected_time_per_block(&self) -> Duration {
        // In NEAR protocol, the block time is 1 second.
        // Considering factors such as network latency, as a precaution,
        // we set the duration to 3 seconds.
        Duration::from_secs(3)
    }

    fn validate_message_signer(&self, signer: &Signer) -> Result<(), ContextError> {
        AccountId::from_str(signer.as_ref()).map_err(|e| {
            ContextError::ClientError(ClientError::Other {
                description: format!("Invalid signer: {:?}", e).to_string(),
            })
        })?;
        Ok(())
    }

    fn get_client_validation_context(&self) -> &Self::ClientValidationContext {
        &self
    }
}
