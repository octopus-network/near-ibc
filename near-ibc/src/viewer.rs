use crate::types::{Qualified, QueryHeight};
use crate::*;
use crate::{types::QueryPacketEventDataRequest, Contract};
use ibc::{
    core::{
        ics03_connection::connection::{ConnectionEnd, IdentifiedConnectionEnd},
        ics04_channel::{
            channel::{ChannelEnd, IdentifiedChannelEnd},
            commitment::{AcknowledgementCommitment, PacketCommitment},
            packet::Sequence,
        },
        ics23_commitment::commitment::CommitmentPrefix,
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    },
    Height,
};

pub trait Viewer {
    /// Get the latest height of the host chain.
    fn get_latest_height(&self) -> Height;
    /// Get the connection end associated with the given connection identifier.
    fn get_connection_end(&self, connection_id: ConnectionId) -> Option<ConnectionEnd>;
    /// Get all of the connection ends stored on this host.
    fn get_connection_ends(&self) -> Vec<(ConnectionId, ConnectionEnd)>;
    /// Get the channel end associated with the given port and channel identifiers.
    fn get_channel_end(&self, port_id: PortId, channel_id: ChannelId) -> Option<ChannelEnd>;
    /// Get the raw client state associated with the given client identifier.
    fn get_client_state(&self, client_id: ClientId) -> Vec<u8>;
    /// Get the heights of all stored consensus states associated with the given client identifier.
    fn get_client_consensus_heights(&self, client_id: ClientId) -> Vec<Height>;
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
    fn get_channels(&self) -> Vec<IdentifiedChannelEnd>;
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
    fn get_next_sequence_receive(&self, port_id: PortId, channel_id: ChannelId)
        -> Option<Sequence>;
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
    /// Get the packet events associated with the given query request.
    fn get_packet_events(
        &self,
        request: QueryPacketEventDataRequest,
    ) -> Vec<(Height, Vec<IbcEvent>)>;
    /// Get the heights that ibc events happened on.
    fn get_ibc_events_heights(&self) -> Vec<u64>;
    /// Get ibc events happened on the given height.
    fn get_ibc_events_at(&self, height: u64) -> Vec<IbcEvent>;
}

#[near_bindgen]
impl Viewer for Contract {
    fn get_latest_height(&self) -> Height {
        Height::new(env::epoch_height(), env::block_height()).unwrap()
    }

    fn get_connection_end(&self, connection_id: ConnectionId) -> Option<ConnectionEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .connections
            .get(&connection_id)
            .map_or_else(|| None, |connection_end| Some(connection_end.clone()))
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

    fn get_channel_end(&self, port_id: PortId, channel_id: ChannelId) -> Option<ChannelEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .channels
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or_else(|| None, |ce| Some(ce.clone()))
    }

    fn get_client_state(&self, client_id: ClientId) -> Vec<u8> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        let option = near_ibc_store.client_states.get(&client_id);
        log!("get_client_state with {:?},result: {:?}", client_id, option);
        option.unwrap().clone()
    }

    fn get_client_consensus_heights(&self, client_id: ClientId) -> Vec<Height> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.consensus_states.get(&client_id).map_or_else(
            || Vec::new(),
            |consensus_states| {
                consensus_states
                    .keys()
                    .iter()
                    .filter(|height| height.is_some())
                    .map(|height| height.unwrap())
                    .collect()
            },
        )
    }

    fn get_client_consensus(&self, client_id: ClientId, consensus_height: Height) -> Vec<u8> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.consensus_states.get(&client_id).map_or_else(
            || Vec::new(),
            |consensus_states| {
                consensus_states
                    .get_value_by_key(&consensus_height.into())
                    .unwrap()
            },
        )
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
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        let stored_sequences = near_ibc_store
            .packet_receipts
            .get(&(port_id, channel_id))
            .map_or_else(|| vec![], |receipts| receipts.keys());
        sequences
            .iter()
            .filter(|sequence| !stored_sequences.contains(&Some(**sequence)))
            .cloned()
            .collect()
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
            .map_or_else(
                || vec![],
                |connections| connections.iter().map(|c| c.clone()).collect_vec(),
            )
    }

    fn get_channels(&self) -> Vec<IdentifiedChannelEnd> {
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
            .map_or_else(
                || vec![],
                |channels| {
                    channels
                        .iter()
                        .filter(|key| near_ibc_store.channels.contains_key(key))
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
                },
            )
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

    fn get_next_sequence_receive(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Option<Sequence> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .next_sequence_recv
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or_else(|| None, |sq| Some(sq.clone()))
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

    fn get_packet_events(
        &self,
        request: QueryPacketEventDataRequest,
    ) -> Vec<(Height, Vec<IbcEvent>)> {
        let mut result: Vec<(Height, Vec<IbcEvent>)> = Vec::new();
        let (target_height, need_to_search_in_range) = match &request.height {
            Qualified::SmallerEqual(query_height) => match query_height {
                QueryHeight::Latest => (self.ibc_events_history.latest_key(), true),
                QueryHeight::Specific(height) => (Some(height.revision_height()), true),
            },
            Qualified::Equal(query_height) => match query_height {
                QueryHeight::Latest => (self.ibc_events_history.latest_key(), false),
                QueryHeight::Specific(height) => (Some(height.revision_height()), false),
            },
        };
        if need_to_search_in_range {
            if let Some(height) = target_height {
                self.ibc_events_history
                    .keys()
                    .iter()
                    .filter(|key| key.is_some())
                    .map(|key| key.unwrap())
                    .filter(|key| *key <= height)
                    .for_each(|key| {
                        gether_ibc_events_with_height(
                            &mut result,
                            key,
                            self.ibc_events_history
                                .get_value_by_key(&key)
                                .map(|events| {
                                    Vec::<IbcEvent>::try_from_slice(&events)
                                        .unwrap_or_else(|_| vec![])
                                })
                                .unwrap_or_else(|| vec![]),
                            &request,
                        );
                    });
            }
        } else {
            let events = self
                .ibc_events_history
                .get_value_by_key(&target_height.unwrap())
                .map_or_else(|| vec![], |events| events);
            let ibc_events = Vec::<IbcEvent>::try_from_slice(&events).unwrap_or_else(|_| vec![]);
            gether_ibc_events_with_height(&mut result, target_height.unwrap(), ibc_events, &request)
        }
        result
    }

    fn get_ibc_events_heights(&self) -> Vec<u64> {
        self.ibc_events_history
            .keys()
            .iter()
            .filter(|h| h.is_some())
            .map(|h| h.unwrap())
            .collect()
    }

    fn get_ibc_events_at(&self, height: u64) -> Vec<IbcEvent> {
        let raw_events = self
            .ibc_events_history
            .get_value_by_key(&height)
            .map_or_else(|| vec![], |events| events);
        Vec::<IbcEvent>::try_from_slice(&raw_events).unwrap_or_else(|_| vec![])
    }
}

fn gether_ibc_events_with_height(
    result: &mut Vec<(Height, Vec<IbcEvent>)>,
    height: u64,
    ibc_events: Vec<IbcEvent>,
    request: &QueryPacketEventDataRequest,
) {
    let events = ibc_events
        .iter()
        .filter(|event| request.event_type.eq(event.event_type()))
        .filter(|event| match event {
            IbcEvent::CreateClient(_) => true,
            IbcEvent::UpdateClient(_) => true,
            IbcEvent::ReceivePacket(receive_packet) => {
                request.source_port_id.eq(receive_packet.src_port_id())
                    && request
                        .source_channel_id
                        .eq(receive_packet.src_channel_id())
                    && request.destination_port_id.eq(receive_packet.dst_port_id())
                    && request
                        .destination_channel_id
                        .eq(receive_packet.dst_channel_id())
                    && request.sequences.contains(&receive_packet.sequence())
            }
            IbcEvent::WriteAcknowledgement(write_ack) => {
                request.source_port_id.eq(write_ack.src_port_id())
                    && request.source_channel_id.eq(write_ack.src_channel_id())
                    && request.destination_port_id.eq(write_ack.dst_port_id())
                    && request
                        .destination_channel_id
                        .eq(write_ack.dst_channel_id())
                    && request.sequences.contains(&write_ack.sequence())
            }
            _ => false,
        })
        .map(|event| event.clone())
        .collect_vec();
    result.push((Height::new(0, height).unwrap(), events));
}
