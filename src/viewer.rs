use crate::ibc_impl::core::host::type_define::NearConsensusState;
use crate::interfaces::Viewer;
use crate::Contract;
use crate::*;
use ibc::core::ics02_client::context::ClientReader;
use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::ics03_connection::context::ConnectionReader;
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::context::ChannelReader;
use ibc::core::ics04_channel::packet::{Receipt, Sequence};
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::mock::client_state::MockClientState;
use ibc::mock::consensus_state::MockConsensusState;
use ibc::Height;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::{
    PacketState, QueryChannelsRequest, QueryPacketCommitmentRequest, QueryPacketCommitmentsRequest,
};
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;
use ibc_proto::protobuf::Protobuf;

impl Viewer for Contract {
    fn query_connection_end(&mut self, connection_id: ConnectionId) -> ConnectionEnd {
        let context = self.build_ibc_context();
        ChannelReader::connection_end(&context, &connection_id).unwrap()
    }

    fn query_channel_end(&mut self, port_id: PortId, channel_id: ChannelId) -> ChannelEnd {
        let context = self.build_ibc_context();
        context.channel_end(&port_id, &channel_id).unwrap()
    }

    fn query_client_state(&mut self, client_id: ClientId) -> MockClientState {
        // let context = self.build_ibc_context();
        // context.client_state(&client_id).unwrap()
        todo!() // add near client state
    }

    fn query_client_consensus(
        &mut self,
        client_id: ClientId,
        consensus_height: Height,
    ) -> MockConsensusState {
        todo!()
    }

    fn get_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Receipt {
        let context = self.build_ibc_context();
        context
            .get_packet_receipt(&port_id, &channel_id, seq)
            .unwrap()
    }

    fn get_unreceipt_packet(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequences: Vec<Sequence>,
    ) -> Vec<Sequence> {
        let context = self.build_ibc_context();

        // let near_port_id =

        // sequences
        //     .iter()
        //     .filter(|&e| context.near_ibc_store.packet_receipt.contains_key((p)))
        //     .collect()
        todo!()

        // sequences.iter().filter(|e|  )

        // context.near_ibc_store
        //     .packet_receipt
        //     .get(&(port_id.as_bytes().into(), channel_id.as_bytes().into()))
        //     .filter(|e|)
        // context.get_packet_receipt(&port_id, &channel_id)
        // context.get_unre(&port_id, &channel_id, seq).unwrap()
    }

    fn get_clients(&mut self) -> Vec<MockClientState> {
        todo!()
    }

    fn get_connections(&mut self) -> Vec<IdentifiedConnectionEnd> {
        self.ibc_store
            .connections
            .iter()
            .map(|(connection_id, connection_end)| IdentifiedConnectionEnd {
                connection_id: connection_id.clone(),
                connection_end: connection_end.clone(),
            })
            .collect()
    }

    /// ignore pagination now, return all datas
    fn get_channels(&mut self, request: QueryChannelsRequest) -> Vec<IdentifiedChannelEnd> {
        let context = self.build_ibc_context();
        context
            .near_ibc_store
            .channels
            .iter()
            .map(
                |((port_id, channel_id), channel_end)| IdentifiedChannelEnd {
                    port_id,
                    channel_id,
                    channel_end,
                },
            )
            .collect()
    }

    fn get_commitment_packet_state(&mut self) -> Vec<PacketState> {
        let context = self.build_ibc_context();
        todo!()
    }

    fn get_packet_commitment(&mut self) -> Vec<u8> {
        todo!()
    }

    fn query_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Vec<u8> {
        todo!()
    }

    fn query_next_sequence_receive(port_id: &PortId, channel_id: &ChannelId) -> Sequence {
        todo!()
    }

    fn query_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) {
        todo!()
    }

    fn query_packet_commitments(
        &mut self,
        _request: QueryPacketCommitmentsRequest,
    ) -> Vec<Sequence> {
        todo!()
    }

    // fn query_packet_commitment(&mut self, request: QueryPacketCommitmentRequest, include_proof: IncludeProof) -> Vec<u8> {
    //     todo!()
    // }
}
