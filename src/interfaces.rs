use crate::ibc_impl::core::host::type_define::NearConsensusState;
use crate::types::{Height, Receipt};
use crate::Contract;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::mock::client_state::MockClientState;
use ibc::mock::consensus_state::MockConsensusState;
use ibc_proto::ibc::core::channel::v1::{
    PacketState, QueryChannelsRequest, QueryChannelsResponse, QueryPacketCommitmentRequest,
    QueryPacketCommitmentsRequest,
};
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;

pub trait Viewer {
    fn get_latest_height(&self) -> Height;
    fn get_connection_end(&self, connection_id: ConnectionId) -> ConnectionEnd;
    fn get_connection_ends(&self) -> Vec<(ConnectionId, ConnectionEnd)>;
    fn get_channel_end(&self, port_id: PortId, channel_id: ChannelId) -> ChannelEnd;

    //todo change IbcContext
    fn get_client_state(&self, client_id: ClientId) -> Vec<u8>;

    // todo definite near ConsensusState
    fn get_client_consensus(&self, client_id: ClientId, consensus_height: Height) -> Vec<u8>;

    // fn get_consensus_state_with_height(&mut self, client_id: &ClientId) -> Vec<(Height, AnyConsensusState)>;

    fn get_packet_receipt(&self, port_id: PortId, channel_id: ChannelId, seq: Sequence) -> Receipt;

    fn get_unreceipt_packet(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequences: Vec<Sequence>,
    ) -> Vec<Sequence>;

    //todo definite near client state
    fn get_clients(&self) -> Vec<MockClientState>;

    fn get_client_counter(&self) -> u64;

    fn get_connections(&self) -> Vec<IdentifiedConnectionEnd>;

    fn get_channels(&self, request: QueryChannelsRequest) -> Vec<IdentifiedChannelEnd>;

    fn get_commitment_packet_state(&self) -> Vec<PacketState>;

    fn get_packet_commitment(&self) -> Vec<u8>;

    fn get_next_sequence_receive(port_id: &PortId, channel_id: &ChannelId) -> Sequence;

    fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    );

    fn get_packet_commitments(&self, _request: QueryPacketCommitmentsRequest) -> Vec<Sequence>;

    fn get_commitment_prefix(&self) -> CommitmentPrefix;
}
