use crate::{context::NearIbcStore, prelude::*};
use core::fmt::{Debug, Formatter};
use ibc::{
    core::{
        events::IbcEvent,
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
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
        router::{Module, ModuleId, Router},
        timestamp::Timestamp,
        ContextError, ExecutionContext, ValidationContext,
    },
    Height,
};
use near_sdk::{
    borsh::maybestd::{borrow::Borrow, collections::BTreeMap, sync::Arc},
    env, log,
};

impl ExecutionContext for NearIbcStore {
    fn store_client_state(
        &mut self,
        client_state_path: ibc::core::ics24_host::path::ClientStatePath,
        client_state: Box<dyn ClientState>,
    ) -> Result<(), ContextError> {
        log!("store_client_state, client_state: {:?}", client_state);
        let data = client_state.encode_vec();
        let key = client_state_path.to_string().into_bytes();
        env::storage_write(&key, &data);
        Ok(())
    }

    fn store_consensus_state(
        &mut self,
        consensus_state_path: ibc::core::ics24_host::path::ClientConsensusStatePath,
        consensus_state: Box<dyn ConsensusState>,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn increase_client_counter(&mut self) {
        todo!()
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_connection(
        &mut self,
        connection_path: &ibc::core::ics24_host::path::ConnectionPath,
        connection_end: ConnectionEnd,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_connection_to_client(
        &mut self,
        client_connection_path: &ibc::core::ics24_host::path::ClientConnectionPath,
        conn_id: ConnectionId,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn increase_connection_counter(&mut self) {
        todo!()
    }

    fn store_packet_commitment(
        &mut self,
        commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_packet_commitment(
        &mut self,
        commitment_path: &ibc::core::ics24_host::path::CommitmentPath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_receipt(
        &mut self,
        receipt_path: &ibc::core::ics24_host::path::ReceiptPath,
        receipt: Receipt,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_packet_acknowledgement(
        &mut self,
        ack_path: &ibc::core::ics24_host::path::AckPath,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn delete_packet_acknowledgement(
        &mut self,
        ack_path: &ibc::core::ics24_host::path::AckPath,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_channel(
        &mut self,
        channel_end_path: &ibc::core::ics24_host::path::ChannelEndPath,
        channel_end: ChannelEnd,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &ibc::core::ics24_host::path::SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_recv(
        &mut self,
        seq_recv_path: &ibc::core::ics24_host::path::SeqRecvPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn store_next_sequence_ack(
        &mut self,
        seq_ack_path: &ibc::core::ics24_host::path::SeqAckPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        todo!()
    }

    fn increase_channel_counter(&mut self) {
        todo!()
    }

    fn emit_ibc_event(&mut self, event: IbcEvent) {
        todo!()
    }

    fn log_message(&mut self, message: String) {
        todo!()
    }
}
