use crate::*;
use crate::{
    collections::IndexedAscendingQueueViewer,
    types::{Qualified, QueryHeight, QueryPacketEventDataRequest},
};
use ibc::{
    clients::tendermint::context::CommonContext,
    core::{
        channel::types::{
            channel::{ChannelEnd, IdentifiedChannelEnd},
            commitment::{AcknowledgementCommitment, PacketCommitment},
        },
        client::types::Height,
        connection::types::{ConnectionEnd, IdentifiedConnectionEnd},
        handler::types::events::IbcEvent,
        host::{
            types::{
                identifiers::{ChannelId, ClientId, ConnectionId, PortId, Sequence},
                path::{
                    AckPath, ChannelEndPath, ClientConnectionPath, ClientConsensusStatePath,
                    ClientStatePath, CommitmentPath, ReceiptPath, SeqRecvPath,
                },
            },
            ValidationContext,
        },
    },
};
use itertools::Itertools;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    env, near_bindgen,
};

pub trait Viewer {
    /// Show the version of the contract.
    fn version(&self) -> String;
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
    /// Get the channel ends associated with the given connection id.
    fn get_connection_channels(&self, connection_id: ConnectionId) -> Vec<IdentifiedChannelEnd>;
    /// Get the channel ends associated with the given query request.
    fn get_channels(&self) -> Vec<IdentifiedChannelEnd>;
    /// Get the packet commitment stored on this host.
    fn get_packet_commitment(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Option<PacketCommitment>;
    /// Get the packet commitment sequences associated with the given port, channel.
    fn get_packet_commitment_sequences(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Vec<Sequence>;
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
    fn get_packet_acknowledgement_sequences(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Vec<Sequence>;
    /// Get the commitment packet stored on this host.
    fn get_commitment_prefix(&self) -> Vec<u8>;
    /// Get the packet events associated with the given query request.
    fn get_packet_events(
        &self,
        request: QueryPacketEventDataRequest,
    ) -> Vec<(Height, Vec<IbcEvent>)>;
    /// Get the heights that ibc events happened on.
    fn get_ibc_events_heights(&self) -> Vec<Height>;
    /// Get ibc events happened on the given height.
    fn get_ibc_events_at(&self, height: Height) -> Vec<IbcEvent>;
}

#[near_bindgen]
impl Viewer for NearIbcContract {
    fn version(&self) -> String {
        VERSION.to_string()
    }

    fn get_latest_height(&self) -> Height {
        Height::new(0, env::block_height()).unwrap()
    }

    fn get_connection_end(&self, connection_id: ConnectionId) -> Option<ConnectionEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .connection_end(&connection_id)
            .map_or(None, |ce| Some(ce))
    }

    fn get_connection_ends(&self) -> Vec<(ConnectionId, ConnectionEnd)> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .connection_id_set
            .iter()
            .map(|connection_id| {
                (
                    connection_id.clone(),
                    near_ibc_store.connection_end(&connection_id).unwrap(),
                )
            })
            .collect()
    }

    fn get_channel_end(&self, port_id: PortId, channel_id: ChannelId) -> Option<ChannelEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .channel_end(&ChannelEndPath::new(&port_id, &channel_id))
            .map_or(None, |ce| Some(ce))
    }

    fn get_client_state(&self, client_id: ClientId) -> Vec<u8> {
        let client_state_key = ClientStatePath(client_id.clone()).to_string().into_bytes();
        env::storage_read(&client_state_key).unwrap_or(vec![])
    }

    fn get_client_consensus_heights(&self, client_id: ClientId) -> Vec<Height> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .consensus_state_heights(&client_id)
            .unwrap_or(Vec::new())
    }

    fn get_client_consensus(&self, client_id: ClientId, consensus_height: Height) -> Vec<u8> {
        let consensus_state_key = ClientConsensusStatePath::new(
            client_id,
            consensus_height.revision_number(),
            consensus_height.revision_height(),
        )
        .to_string()
        .into_bytes();
        env::storage_read(&consensus_state_key).unwrap_or(vec![])
    }

    fn get_packet_receipt(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Vec<u8> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .get_packet_receipt(&ReceiptPath::new(&port_id, &channel_id, sequence))
            .map_or(vec![], |receipt| borsh::to_vec(&receipt).unwrap())
    }

    fn get_unreceipt_packet(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequences: Vec<Sequence>,
    ) -> Vec<Sequence> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        let stored_sequences = near_ibc_store
            .packet_receipt_sequence_sets
            .get(&(port_id, channel_id))
            .map_or_else(|| vec![], |receipts| receipts.iter().collect());
        sequences
            .iter()
            .filter(|sequence| !stored_sequences.contains(&sequence))
            .cloned()
            .collect()
    }

    fn get_clients(&self) -> Vec<(ClientId, Vec<u8>)> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .client_id_set
            .iter()
            .map(|id| (id.clone(), self.get_client_state(id.clone())))
            .collect()
    }

    fn get_client_counter(&self) -> u64 {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.client_counter
    }

    fn get_connections(&self) -> Vec<IdentifiedConnectionEnd> {
        self.get_connection_ends()
            .iter()
            .map(|(connection_id, connection_end)| IdentifiedConnectionEnd {
                connection_id: connection_id.clone(),
                connection_end: connection_end.clone(),
            })
            .collect()
    }

    fn get_client_connections(&self, client_id: ClientId) -> Vec<ConnectionId> {
        #[derive(BorshDeserialize, BorshSerialize, Debug)]
        #[borsh(crate = "near_sdk::borsh")]
        struct ConnectionIds(pub Vec<ConnectionId>);
        let key = ClientConnectionPath::new(&client_id)
            .to_string()
            .into_bytes();
        env::storage_read(&key)
            .map(|bytes| ConnectionIds::try_from_slice(&bytes).unwrap().0)
            .unwrap_or(vec![])
    }

    fn get_connection_channels(&self, connection_id: ConnectionId) -> Vec<IdentifiedChannelEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .port_channel_id_set
            .iter()
            .filter(|(port_id, channel_id)| {
                near_ibc_store
                    .channel_end(&ChannelEndPath::new(&port_id, &channel_id))
                    .map_or(false, |channel_end| {
                        channel_end.connection_hops.contains(&connection_id)
                    })
            })
            .map(|(port_id, channel_id)| IdentifiedChannelEnd {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                channel_end: near_ibc_store
                    .channel_end(&ChannelEndPath::new(&port_id, &channel_id))
                    .unwrap(),
            })
            .collect()
    }

    fn get_channels(&self) -> Vec<IdentifiedChannelEnd> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .port_channel_id_set
            .iter()
            .map(|(port_id, channel_id)| IdentifiedChannelEnd {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
                channel_end: near_ibc_store
                    .channel_end(&ChannelEndPath::new(&port_id, &channel_id))
                    .unwrap(),
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
            .get_packet_commitment(&CommitmentPath::new(&port_id, &channel_id, sequence))
            .map_or(None, |commitment| Some(commitment))
    }

    fn get_packet_commitment_sequences(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Vec<Sequence> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .packet_commitment_sequence_sets
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or(vec![], |sequences| {
                sequences.iter().map(|seq| seq.clone()).collect()
            })
    }

    fn get_next_sequence_receive(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Option<Sequence> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .get_next_sequence_recv(&SeqRecvPath::new(&port_id, &channel_id))
            .map_or(None, |sq| Some(sq))
    }

    fn get_packet_acknowledgement(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
    ) -> Option<AcknowledgementCommitment> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .get_packet_acknowledgement(&AckPath::new(&port_id, &channel_id, sequence))
            .map_or(None, |ack| Some(ack))
    }

    fn get_packet_acknowledgement_sequences(
        &self,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Vec<Sequence> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .packet_acknowledgement_sequence_sets
            .get(&(port_id.clone(), channel_id.clone()))
            .map_or(vec![], |sequences| {
                sequences.iter().map(|seq| seq.clone()).collect()
            })
    }

    fn get_commitment_prefix(&self) -> Vec<u8> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store.commitment_prefix().into_vec()
    }

    fn get_packet_events(
        &self,
        request: QueryPacketEventDataRequest,
    ) -> Vec<(Height, Vec<IbcEvent>)> {
        let mut result: Vec<(Height, Vec<IbcEvent>)> = Vec::new();
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        let (target_height, need_to_search_in_range) = match &request.height {
            Qualified::SmallerEqual(query_height) => match query_height {
                QueryHeight::Latest => (near_ibc_store.ibc_events_history.latest_key(), true),
                QueryHeight::Specific(height) => (Some(height), true),
            },
            Qualified::Equal(query_height) => match query_height {
                QueryHeight::Latest => (near_ibc_store.ibc_events_history.latest_key(), false),
                QueryHeight::Specific(height) => (Some(height), false),
            },
        };
        if need_to_search_in_range {
            if let Some(height) = target_height {
                near_ibc_store
                    .ibc_events_history
                    .keys()
                    .iter()
                    .filter(|key| key.is_some())
                    .map(|key| key.unwrap())
                    .filter(|key| *key <= height)
                    .for_each(|key| {
                        gether_ibc_events_with_height(
                            &mut result,
                            key,
                            near_ibc_store
                                .ibc_events_history
                                .get_value_by_key(&key)
                                .map(|events| events.clone())
                                .unwrap_or_else(|| vec![]),
                            &request,
                        );
                    });
            }
        } else {
            let ibc_events = near_ibc_store
                .ibc_events_history
                .get_value_by_key(&target_height.unwrap())
                .map(|events| events.clone())
                .unwrap_or_else(|| vec![]);
            gether_ibc_events_with_height(&mut result, target_height.unwrap(), ibc_events, &request)
        }
        result
    }

    fn get_ibc_events_heights(&self) -> Vec<Height> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .ibc_events_history
            .keys()
            .iter()
            .filter(|h| h.is_some())
            .map(|h| h.unwrap().clone())
            .collect()
    }

    fn get_ibc_events_at(&self, height: Height) -> Vec<IbcEvent> {
        let near_ibc_store = self.near_ibc_store.get().unwrap();
        near_ibc_store
            .ibc_events_history
            .get_value_by_key(&height)
            .map(|events| events.clone())
            .unwrap_or_else(|| vec![])
    }
}

fn gether_ibc_events_with_height(
    result: &mut Vec<(Height, Vec<IbcEvent>)>,
    height: &Height,
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
                request.source_port_id.eq(receive_packet.port_id_on_b())
                    && request.source_channel_id.eq(receive_packet.chan_id_on_b())
                    && request
                        .destination_port_id
                        .eq(receive_packet.port_id_on_a())
                    && request
                        .destination_channel_id
                        .eq(receive_packet.chan_id_on_a())
                    && request.sequences.contains(&receive_packet.seq_on_b())
            }
            IbcEvent::WriteAcknowledgement(write_ack) => {
                request.source_port_id.eq(write_ack.port_id_on_a())
                    && request.source_channel_id.eq(write_ack.chan_id_on_a())
                    && request.destination_port_id.eq(write_ack.port_id_on_b())
                    && request.destination_channel_id.eq(write_ack.chan_id_on_b())
                    && request.sequences.contains(&write_ack.seq_on_a())
            }
            _ => false,
        })
        .map(|event| event.clone())
        .collect_vec();
    result.push((height.clone(), events));
}
