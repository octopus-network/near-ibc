use crate::*;
use crate::{ibc_impl::core::host::type_define::RawConsensusState, Contract};
use ibc::{
    core::{
        ics02_client::context::ClientReader,
        ics03_connection::{
            connection::{ConnectionEnd, IdentifiedConnectionEnd},
            context::ConnectionReader,
            error::ConnectionError,
        },
        ics04_channel::{
            channel::{ChannelEnd, IdentifiedChannelEnd},
            commitment::{AcknowledgementCommitment, PacketCommitment},
            context::ChannelReader,
            error::ChannelError,
            packet::{Receipt, Sequence},
        },
        ics23_commitment::commitment::CommitmentPrefix,
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    },
    Height,
};
use ibc_proto::{
    google::protobuf::Any,
    ibc::core::{
        channel::v1::{
            PacketState, QueryChannelsRequest, QueryPacketCommitmentRequest,
            QueryPacketCommitmentsRequest,
        },
        client::v1::IdentifiedClientState,
    },
    protobuf::Protobuf,
};
use near_sdk::json_types::U64;

pub trait Viewer {
    /// Get the latest height of the host chain.
    fn get_latest_height(&self) -> Height;
    /// Get the connection end associated with the given connection identifier.
    fn get_connection_end(&self, connection_id: ConnectionId) -> ConnectionEnd;
    /// Get all of the connection ends stored on this host.
    fn get_connection_ends(&self) -> Vec<(ConnectionId, ConnectionEnd)>;
    /// Get the channel end associated with the given port and channel identifiers.
    fn get_channel_end(&self, port_id: PortId, channel_id: ChannelId) -> ChannelEnd;
    /// Get the raw client state associated with the given client identifier.
    fn get_client_state(&self, client_id: ClientId) -> Vec<u8>;
    /// Get the consensus state associated with the given client identifier and height.
    fn get_client_consensus(&self, client_id: ClientId, consensus_height: Height) -> Vec<u8>;
    /// Get the packet receipt associated with the given port, channel, and sequence.
    fn get_packet_receipt(&self, port_id: PortId, channel_id: ChannelId, seq: Sequence) -> Vec<u8>;
    /// Get the unreceived packet sequences associated with the given port and channel.
    fn get_unreceipt_packet(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequences: Vec<Sequence>,
    ) -> Vec<Sequence>;
    /// Get all of the raw client states stored on this host.
    fn get_clients(&self) -> Vec<(ClientId, Vec<u8>)>;
    /// Get the counter for the number of clients stored on this host.
    fn get_client_counter(&self) -> u64;
    /// Get all of the connection ends stored on this host.
    fn get_connections(&self) -> Vec<IdentifiedConnectionEnd>;
    /// Get all connections associated with the given client id.
    fn get_client_connections(&self, client_id: ClientId) -> Vec<ConnectionId>;
    /// Get the channel ends associated with the given query request.
    fn get_channels(&self, request: QueryChannelsRequest) -> Vec<IdentifiedChannelEnd>;
    /// Get the channel ends associated with the given connection id.
    fn get_connection_channels(&self, connection_id: ConnectionId) -> Vec<IdentifiedChannelEnd>;
    /// Get the packet commitment stored on this host.
    fn get_packet_commitment(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Option<PacketCommitment>;
    /// Get the packet commitment sequences associated with the given port, channel.
    fn get_packet_commitments(&self, port_id: PortId, channel_id: ChannelId) -> Vec<Sequence>;
    /// Get the next sequence receive associated with the given port and channel.
    fn get_next_sequence_receive(&self, port_id: PortId, channel_id: ChannelId) -> Sequence;
    /// Get the packet acknowledgement associated with the given port, channel, and sequence.
    fn get_packet_acknowledgement(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Option<AcknowledgementCommitment>;
    /// Get the packet acknowledgement sequences associated with the given port, channel.
    fn get_packet_acknowledgements(&self, port_id: PortId, channel_id: ChannelId) -> Vec<Sequence>;
    /// Get the commitment packet stored on this host.
    fn get_commitment_prefix(&self) -> CommitmentPrefix;
}

#[near_bindgen]
impl Viewer for Contract {
    fn get_latest_height(&self) -> Height {
        Height::new(env::epoch_height(), env::block_height()).unwrap()
    }

    fn get_connection_end(&self, connection_id: ConnectionId) -> ConnectionEnd {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .connections
            .get(&connection_id)
            .ok_or(ConnectionError::ConnectionMismatch {
                connection_id: connection_id.clone(),
            })
            .unwrap()
            .clone()
    }

    fn get_connection_ends(&self) -> Vec<(ConnectionId, ConnectionEnd)> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .connections
            .keys()
            .into_iter()
            .map(|connection_id| {
                (
                    connection_id.clone(),
                    near_ibc_store
                        .connections
                        .get(&connection_id)
                        .unwrap()
                        .clone(),
                )
            })
            .collect()
    }

    fn get_channel_end(&self, port_id: PortId, channel_id: ChannelId) -> ChannelEnd {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .channels
            .get(&(port_id.clone(), channel_id.clone()))
            .ok_or(ChannelError::ChannelNotFound {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            })
            .unwrap()
            .clone()
    }

    fn get_client_state(&self, client_id: ClientId) -> Vec<u8> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        let option = near_ibc_store.client_states.get(&client_id);
        log!("get_client_state with {:?},result: {:?}", client_id, option);
        option.unwrap().clone()
    }

    fn get_client_consensus(&self, client_id: ClientId, consensus_height: Height) -> Vec<u8> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        let option = near_ibc_store
            .consensus_states
            .get(&client_id)
            .unwrap()
            .get_value_by_key(&consensus_height.into());
        log!("get_client_state with {:?},result: {:?}", client_id, option);
        option.unwrap()
    }

    fn get_packet_receipt(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Vec<u8> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .packet_receipts
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or_else(
                || vec![],
                |receipts| {
                    receipts
                        .get_value_by_key(&sequence)
                        .map_or_else(|| vec![], |rcpt| rcpt.try_to_vec().unwrap())
                },
            )
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

    fn get_clients(&self) -> Vec<(ClientId, Vec<u8>)> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .client_states
            .keys()
            .into_iter()
            .map(|client_id| {
                (
                    client_id.clone(),
                    near_ibc_store.client_states.get(client_id).unwrap().clone(),
                )
            })
            .collect()
    }

    fn get_client_counter(&self) -> u64 {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.client_ids_counter
    }

    fn get_connections(&self) -> Vec<IdentifiedConnectionEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .connections
            .iter()
            .map(|(connection_id, connection_end)| IdentifiedConnectionEnd {
                connection_id: connection_id.clone(),
                connection_end: connection_end.clone(),
            })
            .collect()
    }

    fn get_client_connections(&self, client_id: ClientId) -> Vec<ConnectionId> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .client_connections
            .get(&client_id)
            .unwrap()
            .iter()
            .map(|c| c.clone())
            .collect_vec()
    }

    fn get_channels(&self, request: QueryChannelsRequest) -> Vec<IdentifiedChannelEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .channels
            .iter()
            .map(
                |((port_id, channel_id), channel_end)| IdentifiedChannelEnd {
                    port_id: port_id.clone(),
                    channel_id: channel_id.clone(),
                    channel_end: channel_end.clone(),
                },
            )
            .collect()
    }

    fn get_connection_channels(&self, connection_id: ConnectionId) -> Vec<IdentifiedChannelEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .connection_channels
            .get(&connection_id)
            .unwrap()
            .iter()
            .map(|(port_id, channel_id)| IdentifiedChannelEnd {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                channel_end: near_ibc_store
                    .channels
                    .get(&(port_id.clone(), channel_id.clone()))
                    .unwrap()
                    .clone(),
            })
            .collect()
    }

    fn get_packet_commitment(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Option<PacketCommitment> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .packet_commitments
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or_else(
                || None,
                |commitments| commitments.get_value_by_key(&sequence),
            )
    }

    fn get_packet_commitments(&self, port_id: PortId, channel_id: ChannelId) -> Vec<Sequence> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .packet_commitments
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or_else(
                || vec![],
                |commitments| {
                    commitments
                        .keys()
                        .iter()
                        .filter(|sq| sq.is_some())
                        .map(|sq| sq.unwrap())
                        .collect()
                },
            )
    }

    fn get_next_sequence_receive(&self, port_id: PortId, channel_id: ChannelId) -> Sequence {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .next_sequence_recv
            .get(&(port_id.clone(), channel_id.clone()))
            .unwrap()
            .clone()
    }

    fn get_packet_acknowledgement(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Option<AcknowledgementCommitment> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .packet_acknowledgements
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or_else(|| None, |acks| acks.get_value_by_key(&sequence))
    }

    fn get_packet_acknowledgements(&self, port_id: PortId, channel_id: ChannelId) -> Vec<Sequence> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .packet_acknowledgements
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or_else(
                || vec![],
                |acks| {
                    acks.keys()
                        .iter()
                        .filter(|sq| sq.is_some())
                        .map(|sq| sq.unwrap())
                        .collect()
                },
            )
    }

    fn get_commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from(DEFAULT_COMMITMENT_PREFIX.as_bytes().to_vec())
            .unwrap_or_default()
    }
}
