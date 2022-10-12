use crate::NearContext;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::consensus_state::ConsensusState;
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics04_channel::channel::ChannelEnd;
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics04_channel::context::{ChannelKeeper, ChannelReader};
use ibc::core::ics04_channel::error::Error as Ics04Error;
use ibc::core::ics04_channel::packet::{Receipt, Sequence};
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use ibc::timestamp::Timestamp;
use ibc::Height;
use std::time::Duration;

impl ChannelReader for NearContext {
    /// Returns the ChannelEnd for the given `port_id` and `chan_id`.
    fn channel_end(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<ChannelEnd, Ics04Error> {
        todo!()
    }

    /// Returns the ConnectionState for the given identifier `connection_id`.
    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error> {
        todo!()
    }

    fn connection_channels(
        &self,
        cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, Ics04Error> {
        todo!()
    }

    /// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
    /// proof verification.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics04Error> {
        todo!()
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Box<dyn ConsensusState>, Ics04Error> {
        todo!()
    }

    fn get_next_sequence_send(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, Ics04Error> {
        todo!()
    }

    fn get_next_sequence_recv(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, Ics04Error> {
        todo!()
    }

    fn get_next_sequence_ack(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, Ics04Error> {
        todo!()
    }

    fn get_packet_commitment(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<PacketCommitment, Ics04Error> {
        todo!()
    }

    fn get_packet_receipt(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<Receipt, Ics04Error> {
        todo!()
    }

    fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<AcknowledgementCommitment, Ics04Error> {
        todo!()
    }

    /// A hashing function for packet commitments
    fn hash(&self, value: Vec<u8>) -> Vec<u8> {
        todo!()
    }

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Height {
        todo!()
    }

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(&self, height: Height) -> Result<Box<dyn ConsensusState>, Ics04Error> {
        todo!()
    }

    /// Returns the pending `ConsensusState` of the host (local) chain.
    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, Ics04Error> {
        todo!()
    }

    /// Returns the time when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Timestamp, Ics04Error> {
        todo!()
    }

    /// Returns the height when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Height, Ics04Error> {
        todo!()
    }

    /// Returns a counter on the number of channel ids have been created thus far.
    /// The value of this counter should increase only via method
    /// `ChannelKeeper::increase_channel_counter`.
    fn channel_counter(&self) -> Result<u64, Ics04Error> {
        todo!()
    }

    /// Returns the maximum expected time per block
    fn max_expected_time_per_block(&self) -> Duration {
        todo!()
    }
}

impl ChannelKeeper for NearContext {
    fn store_packet_commitment(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn delete_packet_commitment(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn store_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        receipt: Receipt,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn store_packet_acknowledgement(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn delete_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn store_connection_channels(
        &mut self,
        conn_id: ConnectionId,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    /// Stores the given channel_end at a path associated with the port_id and channel_id.
    fn store_channel(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn store_next_sequence_send(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn store_next_sequence_recv(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    fn store_next_sequence_ack(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        todo!()
    }

    /// Called upon channel identifier creation (Init or Try message processing).
    /// Increases the counter which keeps track of how many channels have been created.
    /// Should never fail.
    fn increase_channel_counter(&mut self) {
        todo!()
    }
}
