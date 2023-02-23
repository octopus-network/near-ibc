use crate::ibc_impl::core::host::type_define::NearConsensusState;
use crate::interfaces::Viewer;
use crate::types::{Height, Receipt};
use crate::Contract;
use crate::*;
use ibc::core::ics02_client::context::ClientReader;
use ibc::core::ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd};
use ibc::core::ics03_connection::context::ConnectionReader;
use ibc::core::ics03_connection::error::ConnectionError;
use ibc::core::ics04_channel::channel::{ChannelEnd, IdentifiedChannelEnd};
use ibc::core::ics04_channel::context::ChannelReader;
use ibc::core::ics04_channel::error::ChannelError;
use ibc::core::ics04_channel::packet::Sequence;
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId};
use ibc::mock::client_state::MockClientState;
use ibc::mock::consensus_state::MockConsensusState;
use ibc_proto::google::protobuf::Any;
use ibc_proto::ibc::core::channel::v1::{
    PacketState, QueryChannelsRequest, QueryPacketCommitmentRequest, QueryPacketCommitmentsRequest,
};
use ibc_proto::ibc::core::client::v1::IdentifiedClientState;
use ibc_proto::protobuf::Protobuf;
use near_sdk::json_types::U64;

#[near_bindgen]
impl Viewer for Contract {
    fn get_latest_height(&self) -> Height {
        Height {
            revision_height: U64(env::epoch_height()),
            revision_number: U64(env::block_height()),
        }
    }

    fn get_connection_end(&self, connection_id: ConnectionId) -> ConnectionEnd {
        self.near_ibc_store
            .connections
            .get(&connection_id)
            .ok_or(ConnectionError::ConnectionMismatch {
                connection_id: connection_id.clone(),
            })
            .unwrap()
    }

    fn get_connection_ends(&self) -> Vec<(ConnectionId, ConnectionEnd)> {
        self.near_ibc_store.connections.to_vec()
    }

    fn get_channel_end(&self, port_id: PortId, channel_id: ChannelId) -> ChannelEnd {
        self.near_ibc_store
            .channels
            .get(&(port_id.clone(), channel_id.clone()))
            .ok_or(ChannelError::ChannelNotFound {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            })
            .unwrap()
    }

    fn get_client_state(&self, client_id: ClientId) -> Vec<u8> {
        let option = self.near_ibc_store.client_state.get(&client_id);
        log!("get_client_state with {:?},result: {:?}", client_id, option);
        option.unwrap()
    }

    fn get_client_consensus(&self, client_id: ClientId, consensus_height: Height) -> Vec<u8> {
        let option = self
            .near_ibc_store
            .consensus_states
            .get(&client_id)
            .unwrap()
            .get(&consensus_height.into());
        log!("get_client_state with {:?},result: {:?}", client_id, option);
        option.unwrap()
    }

    fn get_packet_receipt(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Receipt {
        todo!()
    }

    fn get_unreceipt_packet(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequences: Vec<Sequence>,
    ) -> Vec<Sequence> {
        // let context = self.build_ibc_context();

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

    fn get_clients(&self) -> Vec<MockClientState> {
        todo!()
    }

    fn get_client_counter(&self) -> u64 {
        self.near_ibc_store.client_ids_counter
    }

    fn get_connections(&self) -> Vec<IdentifiedConnectionEnd> {
        self.near_ibc_store
            .connections
            .iter()
            .map(|(connection_id, connection_end)| IdentifiedConnectionEnd {
                connection_id: connection_id.clone(),
                connection_end: connection_end.clone(),
            })
            .collect()
    }

    /// ignore pagination now, return all datas
    fn get_channels(&self, request: QueryChannelsRequest) -> Vec<IdentifiedChannelEnd> {
        self.near_ibc_store
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

    fn get_commitment_packet_state(&self) -> Vec<PacketState> {
        // let context = self.build_ibc_context();
        todo!()
    }

    fn get_packet_commitment(&self) -> Vec<u8> {
        todo!()
    }

    fn get_next_sequence_receive(port_id: &PortId, channel_id: &ChannelId) -> Sequence {
        todo!()
    }

    fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) {
        todo!()
    }

    fn get_packet_commitments(&self, _request: QueryPacketCommitmentsRequest) -> Vec<Sequence> {
        todo!()
    }

    fn get_commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from(b"Ibc".to_vec()).unwrap_or_default()
    }
}
