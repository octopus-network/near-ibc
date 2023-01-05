use crate::ibc_impl::core::host::type_define::NearConsensusState;
use crate::Contract;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::height::Height;
use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::packet::{Receipt, Sequence};
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::mock::client_state::MockClientState;
use ibc::mock::consensus_state::MockConsensusState;
use ibc_proto::ibc::core::channel::v1::{
    PacketState, QueryChannelsRequest, QueryChannelsResponse, QueryPacketCommitmentRequest,
    QueryPacketCommitmentsRequest,
};
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;

pub trait Viewer {
    fn query_connection_end(&mut self, connection_id: ConnectionId) -> ConnectionEnd;
    fn query_channel_end(&mut self, port_id: PortId, channel_id: ChannelId) -> ChannelEnd;

    //todo definite near client state
    fn query_client_state(&mut self, client_id: ClientId) -> MockClientState;

    // todo definite near ConsensusState
    fn query_client_consensus(
        &mut self,
        client_id: ClientId,
        consensus_height: Height,
    ) -> MockConsensusState;

    // fn get_consensus_state_with_height(&mut self, client_id: &ClientId) -> Vec<(Height, AnyConsensusState)>;

    fn get_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Receipt;

    fn get_unreceipt_packet(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequences: Vec<Sequence>,
    ) -> Vec<Sequence>;

    //todo definite near client state
    fn get_clients(&mut self) -> Vec<MockClientState>;

    fn get_connections(&mut self) -> Vec<IdentifiedConnectionEnd>;

    fn get_channels(&mut self, request: QueryChannelsRequest) -> Vec<IdentifiedChannelEnd>;

    fn get_commitment_packet_state(&mut self) -> Vec<PacketState>;

    fn get_packet_commitment(&mut self) -> Vec<u8>;

    fn query_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Vec<u8>;

    fn query_next_sequence_receive(port_id: &PortId, channel_id: &ChannelId) -> Sequence;

    fn query_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    );

    // fn query_packet_commitment(
    //     &mut self,
    //     request: QueryPacketCommitmentRequest,
    //     include_proof: IncludeProof,
    // ) -> Vec<u8>;

    fn query_packet_commitments(
        &mut self,
        _request: QueryPacketCommitmentsRequest,
    ) -> Vec<Sequence>;
}
