use crate::context::NearRouterContext;
use core::fmt::{Debug, Formatter};
use ibc::{
    core::{
        ics02_client::{
            client_state::ClientState,
            client_type::ClientType,
            consensus_state::ConsensusState,
            context::{ClientKeeper, ClientReader},
            error::ClientError,
        },
        ics03_connection::{
            connection::ConnectionEnd,
            context::{ConnectionKeeper, ConnectionReader},
            error::ConnectionError,
        },
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            context::{ChannelKeeper, ChannelReader},
            error::{ChannelError, PacketError},
            packet::{Receipt, Sequence},
        },
        ics05_port::{context::PortReader, error::PortError},
        ics23_commitment::commitment::CommitmentPrefix,
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
        ics26_routing::context::{Module, ModuleId, Router, RouterBuilder, RouterContext},
    },
    timestamp::Timestamp,
    Height,
};
use near_sdk::{
    borsh::maybestd::{borrow::Borrow, collections::BTreeMap, sync::Arc},
    log,
};

#[derive(Default)]
pub struct NearRouterBuilder(NearRouter);

impl RouterBuilder for NearRouterBuilder {
    type Router = NearRouter;

    fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
        match self.0 .0.insert(module_id, Arc::new(module)) {
            None => Ok(self),
            Some(_) => Err("Duplicate module_id".to_owned()),
        }
    }

    fn build(self) -> Self::Router {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct NearRouter(BTreeMap<ModuleId, Arc<dyn Module>>);

impl Debug for NearRouter {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0.keys().collect::<Vec<&ModuleId>>())
    }
}

impl Router for NearRouter {
    fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
        self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
    }

    fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
        log!("All module id(s) in NearRouter: {:?}", self.0.keys());
        self.0.get(module_id.borrow()).is_some()
    }
}

impl RouterContext for NearRouterContext {
    type Router = NearRouter;

    fn router(&self) -> &Self::Router {
        &self.router
    }

    fn router_mut(&mut self) -> &mut Self::Router {
        &mut self.router
    }
}

impl ClientReader for NearRouterContext {
    fn client_type(&self, client_id: &ClientId) -> Result<ClientType, ClientError> {
        self.near_ibc_store.client_type(client_id)
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ClientError> {
        ClientReader::client_state(&self.near_ibc_store, client_id)
    }

    fn decode_client_state(
        &self,
        client_state: ibc_proto::google::protobuf::Any,
    ) -> Result<Box<dyn ClientState>, ClientError> {
        ClientReader::decode_client_state(&self.near_ibc_store, client_state)
    }

    fn consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ClientError> {
        self.near_ibc_store.consensus_state(client_id, height)
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, ClientError> {
        self.near_ibc_store.next_consensus_state(client_id, height)
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, ClientError> {
        self.near_ibc_store.prev_consensus_state(client_id, height)
    }

    fn host_height(&self) -> Result<Height, ClientError> {
        ClientReader::host_height(&self.near_ibc_store)
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ClientError> {
        ClientReader::host_consensus_state(&self.near_ibc_store, height)
    }

    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ClientError> {
        ClientReader::pending_host_consensus_state(&self.near_ibc_store)
    }

    fn client_counter(&self) -> Result<u64, ClientError> {
        self.near_ibc_store.client_counter()
    }
}

impl ClientKeeper for NearRouterContext {
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), ClientError> {
        self.near_ibc_store
            .store_client_type(client_id, client_type)
    }

    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: Box<dyn ClientState>,
    ) -> Result<(), ClientError> {
        self.near_ibc_store
            .store_client_state(client_id, client_state)
    }

    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: Box<dyn ConsensusState>,
    ) -> Result<(), ClientError> {
        self.near_ibc_store
            .store_consensus_state(client_id, height, consensus_state)
    }

    fn increase_client_counter(&mut self) {
        self.near_ibc_store.increase_client_counter()
    }

    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), ClientError> {
        self.near_ibc_store
            .store_update_time(client_id, height, timestamp)
    }

    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), ClientError> {
        self.near_ibc_store
            .store_update_height(client_id, height, host_height)
    }
}

impl ConnectionReader for NearRouterContext {
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ConnectionError> {
        ConnectionReader::connection_end(&self.near_ibc_store, conn_id)
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ConnectionError> {
        ConnectionReader::client_state(&self.near_ibc_store, client_id)
    }

    fn decode_client_state(
        &self,
        client_state: ibc_proto::google::protobuf::Any,
    ) -> Result<Box<dyn ClientState>, ConnectionError> {
        ConnectionReader::decode_client_state(&self.near_ibc_store, client_state)
    }

    fn host_current_height(&self) -> Result<Height, ConnectionError> {
        self.near_ibc_store.host_current_height()
    }

    fn host_oldest_height(&self) -> Result<Height, ConnectionError> {
        self.near_ibc_store.host_oldest_height()
    }

    fn commitment_prefix(&self) -> CommitmentPrefix {
        self.near_ibc_store.commitment_prefix()
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ConnectionError> {
        ConnectionReader::client_consensus_state(&self.near_ibc_store, client_id, height)
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ConnectionError> {
        ConnectionReader::host_consensus_state(&self.near_ibc_store, height)
    }

    fn connection_counter(&self) -> Result<u64, ConnectionError> {
        self.near_ibc_store.connection_counter()
    }

    fn validate_self_client(
        &self,
        counterparty_client_state: ibc_proto::google::protobuf::Any,
    ) -> Result<(), ConnectionError> {
        self.near_ibc_store
            .validate_self_client(counterparty_client_state)
    }
}

impl ConnectionKeeper for NearRouterContext {
    fn store_connection(
        &mut self,
        connection_id: ConnectionId,
        connection_end: ConnectionEnd,
    ) -> Result<(), ConnectionError> {
        self.near_ibc_store
            .store_connection(connection_id, connection_end)
    }

    fn store_connection_to_client(
        &mut self,
        connection_id: ConnectionId,
        client_id: ClientId,
    ) -> Result<(), ConnectionError> {
        self.near_ibc_store
            .store_connection_to_client(connection_id, client_id)
    }

    fn increase_connection_counter(&mut self) {
        self.near_ibc_store.increase_connection_counter()
    }
}

impl ChannelReader for NearRouterContext {
    fn channel_end(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<ChannelEnd, ChannelError> {
        self.near_ibc_store.channel_end(port_id, channel_id)
    }

    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
        ChannelReader::connection_end(&self.near_ibc_store, connection_id)
    }

    fn connection_channels(
        &self,
        cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
        self.near_ibc_store.connection_channels(cid)
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ChannelError> {
        ChannelReader::client_state(&self.near_ibc_store, client_id)
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ChannelReader::client_consensus_state(&self.near_ibc_store, client_id, height)
    }

    fn get_next_sequence_send(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, PacketError> {
        self.near_ibc_store
            .get_next_sequence_send(port_id, channel_id)
    }

    fn get_next_sequence_recv(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, PacketError> {
        self.near_ibc_store
            .get_next_sequence_recv(port_id, channel_id)
    }

    fn get_next_sequence_ack(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, PacketError> {
        self.near_ibc_store
            .get_next_sequence_ack(port_id, channel_id)
    }

    fn get_packet_commitment(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<PacketCommitment, PacketError> {
        self.near_ibc_store
            .get_packet_commitment(port_id, channel_id, sequence)
    }

    fn get_packet_receipt(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<Receipt, PacketError> {
        self.near_ibc_store
            .get_packet_receipt(port_id, channel_id, sequence)
    }

    fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<AcknowledgementCommitment, PacketError> {
        self.near_ibc_store
            .get_packet_acknowledgement(port_id, channel_id, sequence)
    }

    fn hash(&self, value: &[u8]) -> Vec<u8> {
        self.near_ibc_store.hash(value)
    }

    fn host_height(&self) -> Result<Height, ChannelError> {
        ChannelReader::host_height(&self.near_ibc_store)
    }

    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ChannelReader::host_consensus_state(&self.near_ibc_store, height)
    }

    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ChannelReader::pending_host_consensus_state(&self.near_ibc_store)
    }

    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ChannelError> {
        self.near_ibc_store.client_update_time(client_id, height)
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ChannelError> {
        self.near_ibc_store.client_update_height(client_id, height)
    }

    fn channel_counter(&self) -> Result<u64, ChannelError> {
        self.near_ibc_store.channel_counter()
    }

    fn max_expected_time_per_block(&self) -> std::time::Duration {
        self.near_ibc_store.max_expected_time_per_block()
    }
}

impl ChannelKeeper for NearRouterContext {
    fn store_packet_commitment(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), PacketError> {
        self.near_ibc_store
            .store_packet_commitment(port_id, channel_id, sequence, commitment)
    }

    fn delete_packet_commitment(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        seq: &Sequence,
    ) -> Result<(), PacketError> {
        self.near_ibc_store
            .delete_packet_commitment(port_id, channel_id, seq)
    }

    fn store_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        receipt: Receipt,
    ) -> Result<(), PacketError> {
        self.near_ibc_store
            .store_packet_receipt(port_id, channel_id, sequence, receipt)
    }

    fn store_packet_acknowledgement(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), PacketError> {
        self.near_ibc_store.store_packet_acknowledgement(
            port_id,
            channel_id,
            sequence,
            ack_commitment,
        )
    }

    fn delete_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<(), PacketError> {
        self.near_ibc_store
            .delete_packet_acknowledgement(port_id, channel_id, sequence)
    }

    fn store_connection_channels(
        &mut self,
        conn_id: ConnectionId,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Result<(), ChannelError> {
        self.near_ibc_store
            .store_connection_channels(conn_id, port_id, channel_id)
    }

    fn store_channel(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> Result<(), ChannelError> {
        self.near_ibc_store
            .store_channel(port_id, channel_id, channel_end)
    }

    fn store_next_sequence_send(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.near_ibc_store
            .store_next_sequence_send(port_id, channel_id, seq)
    }

    fn store_next_sequence_recv(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.near_ibc_store
            .store_next_sequence_recv(port_id, channel_id, seq)
    }

    fn store_next_sequence_ack(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.near_ibc_store
            .store_next_sequence_ack(port_id, channel_id, seq)
    }

    fn increase_channel_counter(&mut self) {
        self.near_ibc_store.increase_channel_counter()
    }
}

impl PortReader for NearRouterContext {
    fn lookup_module_by_port(&self, port_id: &PortId) -> Result<ModuleId, PortError> {
        self.near_ibc_store.lookup_module_by_port(port_id)
    }
}
