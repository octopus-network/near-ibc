use super::AnyClientState;
use crate::context::NearIbcStore;
use core::fmt::{Debug, Formatter};
use ibc::{
    clients::ics07_tendermint::client_state::ClientState as TmClientState,
    core::{
        ics02_client::{
            client_state::ClientState, client_type::ClientType, consensus_state::ConsensusState,
            error::ClientError,
        },
        ics03_connection::{connection::ConnectionEnd, error::ConnectionError},
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            error::{ChannelError, PacketError},
            packet::{Receipt, Sequence},
        },
        ics23_commitment::commitment::CommitmentPrefix,
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::ClientStatePath,
        },
        router::{Module, ModuleId, Router},
        timestamp::Timestamp,
        ContextError, ValidationContext,
    },
    Height,
};
use ibc_proto::{
    google::protobuf::Any, ibc::lightclients::tendermint::v1::ClientState as RawTmClientState,
    protobuf::Protobuf,
};
use near_sdk::{
    borsh::maybestd::{borrow::Borrow, collections::BTreeMap, sync::Arc},
    env, log,
};

impl ValidationContext for NearIbcStore {
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError> {
        let client_state_path = ClientStatePath(client_id.clone()).to_string().into_bytes();
        match env::storage_read(&client_state_path) {
            Some(data) => {
                let result: AnyClientState =
                    Protobuf::<Any>::decode_vec(&data).map_err(|e| ClientError::Other {
                        description: format!("Decode ClientState failed: {:?}", e).to_string(),
                    })?;
                match result {
                    AnyClientState::Tendermint(tm_client_state) => Ok(Box::new(tm_client_state)),
                }
            }
            None => Err(ContextError::ClientError(
                ClientError::ClientStateNotFound {
                    client_id: client_id.clone(),
                },
            )),
        }
    }

    fn decode_client_state(
        &self,
        client_state: ibc_proto::google::protobuf::Any,
    ) -> Result<Box<dyn ClientState>, ContextError> {
        todo!()
    }

    fn consensus_state(
        &self,
        client_cons_state_path: &ibc::core::ics24_host::path::ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ContextError> {
        todo!()
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
        todo!()
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, ContextError> {
        todo!()
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        todo!()
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        todo!()
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ContextError> {
        todo!()
    }

    fn client_counter(&self) -> Result<u64, ContextError> {
        todo!()
    }

    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        todo!()
    }

    fn validate_self_client(
        &self,
        client_state_of_host_on_counterparty: ibc_proto::google::protobuf::Any,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        todo!()
    }

    fn connection_counter(&self) -> Result<u64, ContextError> {
        todo!()
    }

    fn channel_end(
        &self,
        channel_end_path: &ibc::core::ics24_host::path::ChannelEndPath,
    ) -> Result<ChannelEnd, ContextError> {
        todo!()
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &ibc::core::ics24_host::path::SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        todo!()
    }

    fn get_next_sequence_recv(
        &self,
        seq_recv_path: &ibc::core::ics24_host::path::SeqRecvPath,
    ) -> Result<Sequence, ContextError> {
        todo!()
    }

    fn get_next_sequence_ack(
        &self,
        seq_ack_path: &ibc::core::ics24_host::path::SeqAckPath,
    ) -> Result<Sequence, ContextError> {
        todo!()
    }

    fn get_packet_commitment(
        &self,
        commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
    ) -> Result<PacketCommitment, ContextError> {
        todo!()
    }

    fn get_packet_receipt(
        &self,
        receipt_path: &ibc::core::ics24_host::path::ReceiptPath,
    ) -> Result<Receipt, ContextError> {
        todo!()
    }

    fn get_packet_acknowledgement(
        &self,
        ack_path: &ibc::core::ics24_host::path::AckPath,
    ) -> Result<AcknowledgementCommitment, ContextError> {
        todo!()
    }

    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError> {
        todo!()
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError> {
        todo!()
    }

    fn channel_counter(&self) -> Result<u64, ContextError> {
        todo!()
    }

    fn max_expected_time_per_block(&self) -> std::time::Duration {
        todo!()
    }

    fn validate_message_signer(&self, signer: &ibc::Signer) -> Result<(), ContextError> {
        todo!()
    }
}
