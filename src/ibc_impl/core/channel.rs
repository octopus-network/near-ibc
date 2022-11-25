use crate::context::IbcContext;
use crate::ibc_impl::core::host::type_define::{NearConnectionId, StoreInNear};
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};

use core::{str::FromStr, time::Duration};
use sha2::Digest;
use ibc::{
    core::{
        ics02_client::{
            client_state::ClientState, consensus_state::ConsensusState, context::ClientReader,
        },
        ics03_connection::{
            connection::ConnectionEnd, context::ConnectionReader, error::Error as Ics03Error,
        },
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{
                AcknowledgementCommitment as IbcAcknowledgementCommitment,
                PacketCommitment as IbcPacketCommitment,
            },
            context::{ChannelKeeper, ChannelReader},
            error::Error as Ics04Error,
            packet::{Receipt, Sequence},
        },
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::{
                AcksPath, ChannelEndsPath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
                SeqAcksPath, SeqRecvsPath, SeqSendsPath,
            },
            Path,
        },
    },
    timestamp::Timestamp,
    Height,
};
use ibc_proto::protobuf::Protobuf;
use near_sdk::env::sha256;

impl ChannelReader for IbcContext<'_> {
    /// Returns the ChannelEnd for the given `port_id` and `chan_id`.
    fn channel_end(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<ChannelEnd, Ics04Error> {
        let channel_end_path = ChannelEndsPath(port_id.clone(), channel_id.clone())
            .to_string()
            .as_bytes()
            .to_vec();

        self.near_ibc_store
            .channels
            .get(&channel_end_path)
            .ok_or(Ics04Error::channel_not_found(
                port_id.clone(),
                channel_id.clone(),
            ))
            .and_then(|near_channel_end| {
                near_channel_end
                    .try_into()
                    // ChannelEnd::decode_vec(&near_channel_end)
                    .map_err(|_| Ics04Error::channel_not_found(port_id.clone(), channel_id.clone()))
            })
    }

    /// Returns the ConnectionState for the given identifier `connection_id`.
    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, Ics04Error> {
        ConnectionReader::connection_end(self, connection_id).map_err(Ics04Error::ics03_connection)
    }

    fn connection_channels(
        &self,
        cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, Ics04Error> {
        let connection_channels = self
            .near_ibc_store
            .connection_channels
            .get(&cid.as_bytes().to_vec().into())
            .ok_or(Ics04Error::connection_not_open(cid.clone()))?;

        let mut result: Vec<(PortId, ChannelId)> = vec![];
        for (near_port_id, near_channel_id) in connection_channels {
            result.push((
                near_port_id.try_into().map_err(|e| {
                    Ics04Error::other(format!("Decode ChannelEnds Path format Failed: {:?}", e))
                })?,
                near_channel_id.try_into().map_err(|e| {
                    Ics04Error::other(format!("Decode ChannelEnds Path format Failed: {:?}", e))
                })?,
            ))
        }

        Ok(result)
    }

    /// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
    /// proof verification.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics04Error> {
        ClientReader::client_state(self, client_id)
            .map_err(|e| Ics04Error::ics03_connection(Ics03Error::ics02_client(e)))
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Box<dyn ConsensusState>, Ics04Error> {
        ClientReader::consensus_state(self, client_id, height)
            .map_err(|e| Ics04Error::ics03_connection(Ics03Error::ics02_client(e)))
    }

    fn get_next_sequence_send(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, Ics04Error> {
        self.near_ibc_store
            .next_sequence_send
            .get(&(port_id.as_bytes().into(), channel_id.as_bytes().into()))
            .ok_or(Ics04Error::missing_next_send_seq(
                port_id.clone(),
                channel_id.clone(),
            ))
            .map(Into::into)
    }

    fn get_next_sequence_recv(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, Ics04Error> {
        self.near_ibc_store
            .next_sequence_recv
            .get(&(port_id.as_bytes().into(), channel_id.as_bytes().into()))
            .ok_or(Ics04Error::missing_next_send_seq(
                port_id.clone(),
                channel_id.clone(),
            ))
            .map(Into::into)
    }

    fn get_next_sequence_ack(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, Ics04Error> {
        self.near_ibc_store
            .next_sequence_ack
            .get(&(port_id.as_bytes().into(), channel_id.as_bytes().into()))
            .ok_or(Ics04Error::missing_next_send_seq(
                port_id.clone(),
                channel_id.clone(),
            ))
            .map(Into::into)
    }

    fn get_packet_commitment(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<PacketCommitment, Ics04Error> {
        self.near_ibc_store
            .packet_commitment
            .get(&(port_id.as_bytes().into(), channel_id.as_bytes().into()))
            .ok_or(Ics04Error::missing_next_send_seq(
                port_id.clone(),
                channel_id.clone(),
            ))
            .map(Into::into)
    }

    fn get_packet_receipt(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<Receipt, Ics04Error> {
        self.near_ibc_store
            .packet_receipt
            .get(&(port_id.as_bytes().into(), channel_id.as_bytes().into()))
            .ok_or(Ics04Error::packet_receipt_not_found(sequence))
            .and_then(TryInto::try_into)
    }

    fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<AcknowledgementCommitment, Ics04Error> {
        self.near_ibc_store
            .packet_acknowledgement
            .get(&(
                port_id.as_bytes().into(),
                channel_id.as_bytes().into(),
                sequence.into(),
            ))
            .ok_or(Ics04Error::packet_acknowledgement_not_found(sequence))
            .map(Into::into)
    }

    /// A hashing function for packet commitments
    fn hash(&self, value: Vec<u8>) -> Vec<u8> {
        sha2::Sha256::digest(value).to_vec()
    }

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Height {
        ClientReader::host_height(self)
    }

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(&self, height: Height) -> Result<Box<dyn ConsensusState>, Ics04Error> {
        ConnectionReader::host_consensus_state(self, height).map_err(Ics04Error::ics03_connection)
    }

    /// Returns the pending `ConsensusState` of the host (local) chain.
    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, Ics04Error> {
        ClientReader::pending_host_consensus_state(self)
            .map_err(|e| Ics04Error::ics03_connection(Ics03Error::ics02_client(e)))
    }

    /// Returns the time when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Timestamp, Ics04Error> {
        self.near_ibc_store
            .client_processed_times
            .get(&(client_id.as_bytes().into(), height.into()))
            .ok_or(Ics04Error::processed_time_not_found(
                client_id.clone(),
                height,
            ))
            .and_then(|time| {
                Timestamp::from_nanoseconds(time).map_err(|e| {
                    Ics04Error::other(format!("Timestamp from_nanoseconds failed: {:?}", e))
                })
            })
    }

    /// Returns the height when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Height, Ics04Error> {
        self.near_ibc_store
            .client_processed_heights
            .get(&(client_id.as_bytes().into(), height.into()))
            .ok_or(Ics04Error::processed_height_not_found(
                client_id.clone(),
                height,
            ))
            .map(Into::into)
    }

    /// Returns a counter on the number of channel ids have been created thus far.
    /// The value of this counter should increase only via method
    /// `ChannelKeeper::increase_channel_counter`.
    fn channel_counter(&self) -> Result<u64, Ics04Error> {
        Ok(self.near_ibc_store.channel_ids_counter)
    }

    /// Returns the maximum expected time per block
    fn max_expected_time_per_block(&self) -> Duration {
        // todo set a suitable value
        Duration::from_secs(2)
    }
}

impl ChannelKeeper for IbcContext<'_> {
    fn store_packet_commitment(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), Ics04Error> {
        self.near_ibc_store.packet_commitment.insert(
            &(port_id.as_bytes().into(), channel_id.as_bytes().into()),
            &commitment.into_vec(),
        );

        Ok(())
    }

    fn delete_packet_commitment(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        self.near_ibc_store
            .packet_commitment
            .remove(&(port_id.as_bytes().into(), channel_id.as_bytes().into()));
        Ok(())
    }

    fn store_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        receipt: Receipt,
    ) -> Result<(), Ics04Error> {
        let packet_receipt_path = ReceiptsPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        }
        .to_string()
        .as_bytes()
        .to_vec();

        let receipt = match receipt {
            Receipt::Ok => "Ok",
        };

        self.near_ibc_store.packet_receipt.insert(
            &(port_id.as_bytes().into(), channel_id.as_bytes().into()),
            &receipt.as_bytes().into(),
        );

        Ok(())
    }

    fn store_packet_acknowledgement(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), Ics04Error> {
        self.near_ibc_store.packet_acknowledgement.insert(
            &(
                port_id.as_bytes().into(),
                channel_id.as_bytes().into(),
                sequence.into(),
            ),
            &ack_commitment.into_vec(),
        );

        Ok(())
    }

    fn delete_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: Sequence,
    ) -> Result<(), Ics04Error> {
        let acks_path = AcksPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        }
        .to_string()
        .as_bytes()
        .to_vec();
        self.near_ibc_store.packet_acknowledgement.remove(&(
            port_id.as_bytes().into(),
            channel_id.as_bytes().into(),
            sequence.into(),
        ));
        Ok(())
    }

    fn store_connection_channels(
        &mut self,
        conn_id: ConnectionId,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Result<(), Ics04Error> {
        let near_conn_id = conn_id.as_bytes().to_vec();
        let mut vec = self
            .near_ibc_store
            .connection_channels
            .get(&StoreInNear(near_conn_id))
            .unwrap_or_default();
        vec.push((
            StoreInNear(port_id.as_bytes().to_vec()),
            channel_id.as_bytes().to_vec().into(),
        ));

        self.near_ibc_store
            .connection_channels
            .insert(&conn_id.as_bytes().to_vec().into(), &vec);

        Ok(())
    }

    /// Stores the given channel_end at a path associated with the port_id and channel_id.
    fn store_channel(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> Result<(), Ics04Error> {
        let channel_ends_path = ChannelEndsPath(port_id.clone(), channel_id.clone())
            .to_string()
            .as_bytes()
            .to_vec();

        let channel_end = channel_end
            .encode_vec()
            .map_err(|e| Ics04Error::other(format!("encode channel end failed: {:?}", e)))?;

        self.near_ibc_store
            .channels
            .insert(&channel_ends_path, &StoreInNear(channel_end));

        Ok(())
    }

    fn store_next_sequence_send(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        self.near_ibc_store.next_sequence_send.insert(
            &(port_id.as_bytes().into(), channel_id.as_bytes().into()),
            &(seq.into()),
        );
        Ok(())
    }

    fn store_next_sequence_recv(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        self.near_ibc_store.next_sequence_recv.insert(
            &(port_id.as_bytes().into(), channel_id.as_bytes().into()),
            &(seq.into()),
        );

        Ok(())
    }

    fn store_next_sequence_ack(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), Ics04Error> {
        self.near_ibc_store.next_sequence_ack.insert(
            &(port_id.as_bytes().into(), channel_id.as_bytes().into()),
            &(seq.into()),
        );

        Ok(())
    }

    /// Called upon channel identifier creation (Init or Try message processing).
    /// Increases the counter which keeps track of how many channels have been created.
    /// Should never fail.
    fn increase_channel_counter(&mut self) {
        self.near_ibc_store.channel_ids_counter = self
            .near_ibc_store
            .channel_ids_counter
            .checked_add(1)
            .expect(format!("add channel counter overflow").as_str())
    }
}
