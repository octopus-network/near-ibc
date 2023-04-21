use core::{str::FromStr, time::Duration};

use crate::{
    context::NearIbcStore,
    ibc_impl::core::host::type_define::{NearConnectionId, StoreInNear},
};
use ibc::{
    core::{
        ics02_client::{
            client_state::ClientState, consensus_state::ConsensusState, context::ClientReader,
        },
        ics03_connection::{
            connection::ConnectionEnd, context::ConnectionReader, error::ConnectionError,
        },
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            context::{ChannelKeeper, ChannelReader},
            error::{ChannelError, PacketError},
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
use sha2::Digest;

impl ChannelReader for NearIbcStore {
    /// Returns the ChannelEnd for the given `port_id` and `chan_id`.
    fn channel_end(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<ChannelEnd, ChannelError> {
        self.channels
            .get(&(port_id.clone(), channel_id.clone()))
            .ok_or(ChannelError::ChannelNotFound {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            })
    }

    /// Returns the ConnectionState for the given identifier `connection_id`.
    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ChannelError> {
        ConnectionReader::connection_end(self, connection_id)
            // .map_err(ChannelError::ics03_connection)
            .map_err(ChannelError::Connection)
    }

    fn connection_channels(
        &self,
        cid: &ConnectionId,
    ) -> Result<Vec<(PortId, ChannelId)>, ChannelError> {
        let connection_channels = self
            .connection_channels
            .get(&cid)
            // .ok_or(ChannelError::connection_not_open(cid.clone()))?;
            .ok_or(ChannelError::ConnectionNotOpen {
                connection_id: cid.clone(),
            })?;

        let mut result: Vec<(PortId, ChannelId)> = vec![];
        for (near_port_id, near_channel_id) in connection_channels {
            result.push((
                near_port_id.try_into().map_err(|e| ChannelError::Other {
                    description: format!("Decode ChannelEnds Path format Failed: {:?}", e)
                        .to_string(),
                })?,
                near_channel_id
                    .try_into()
                    .map_err(|e| ChannelError::Other {
                        description: format!("Decode ChannelEnds Path format Failed: {:?}", e)
                            .to_string(),
                    })?,
            ))
        }

        Ok(result)
    }

    /// Returns the ClientState for the given identifier `client_id`. Necessary dependency towards
    /// proof verification.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ChannelError> {
        ClientReader::client_state(self, client_id)
            .map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ClientReader::consensus_state(self, client_id, height)
            .map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
    }

    fn get_next_sequence_send(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, PacketError> {
        self.next_sequence_send
            .get(&(port_id.clone(), channel_id.clone()))
            .ok_or(PacketError::MissingNextSendSeq {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            })
    }

    fn get_next_sequence_recv(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, PacketError> {
        self.next_sequence_recv
            .get(&(port_id.clone(), channel_id.clone()))
            .ok_or(PacketError::MissingNextRecvSeq {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            })
            .map(Into::into)
    }

    fn get_next_sequence_ack(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, PacketError> {
        self.next_sequence_ack
            .get(&(port_id.clone(), channel_id.clone()))
            .ok_or(PacketError::MissingNextSendSeq {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            })
    }

    fn get_packet_commitment(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<PacketCommitment, PacketError> {
        self.packet_commitment
            .get(&(port_id.clone(), channel_id.clone(), sequence.clone()))
            .ok_or(PacketError::MissingNextSendSeq {
                port_id: port_id.clone(),
                channel_id: channel_id.clone(),
            })
    }

    fn get_packet_receipt(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<Receipt, PacketError> {
        self.packet_receipt
            .get(&(port_id.clone(), channel_id.clone(), sequence.clone()))
            .ok_or(PacketError::PacketReceiptNotFound {
                sequence: sequence.clone(),
            })
    }

    fn get_packet_acknowledgement(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<AcknowledgementCommitment, PacketError> {
        self.packet_acknowledgement
            .get(&(port_id.clone(), channel_id.clone(), sequence.clone()))
            .ok_or(PacketError::PacketAcknowledgementNotFound {
                sequence: sequence.clone(),
            })
            .map(Into::into)
    }

    /// A hashing function for packet commitments
    fn hash(&self, value: &[u8]) -> Vec<u8> {
        sha2::Sha256::digest(value).to_vec()
    }

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Result<Height, ChannelError> {
        ClientReader::host_height(self).map_err(|error| ChannelError::MissingHeight)
    }

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ConnectionReader::host_consensus_state(self, height).map_err(ChannelError::Connection)
    }

    /// Returns the pending `ConsensusState` of the host (local) chain.
    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, ChannelError> {
        ClientReader::pending_host_consensus_state(self)
            .map_err(|e| ChannelError::Connection(ConnectionError::Client(e)))
    }

    /// Returns the time when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ChannelError> {
        self.client_processed_times
            .get(&(client_id.clone(), height.clone()))
            .ok_or(ChannelError::ProcessedTimeNotFound {
                client_id: client_id.clone(),
                height: height.clone(),
            })
            .and_then(|time| {
                Timestamp::from_nanoseconds(time).map_err(|e| ChannelError::Other {
                    description: format!("Timestamp from_nanoseconds failed: {:?}", e).to_string(),
                })
            })
    }

    /// Returns the height when the client state for the given [`ClientId`] was updated with a header for the given [`Height`]
    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ChannelError> {
        self.client_processed_heights
            .get(&(client_id.clone(), height.clone()))
            .ok_or(ChannelError::ProcessedHeightNotFound {
                client_id: client_id.clone(),
                height: height.clone(),
            })
    }

    /// Returns a counter on the number of channel ids have been created thus far.
    /// The value of this counter should increase only via method
    /// `ChannelKeeper::increase_channel_counter`.
    fn channel_counter(&self) -> Result<u64, ChannelError> {
        Ok(self.channel_ids_counter)
    }

    /// Returns the maximum expected time per block
    fn max_expected_time_per_block(&self) -> Duration {
        // todo set a suitable value
        Duration::from_secs(2)
    }
}

impl ChannelKeeper for NearIbcStore {
    fn store_packet_commitment(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), PacketError> {
        self.packet_commitment.insert(
            &(port_id.clone(), channel_id.clone(), sequence),
            &commitment,
        );

        Ok(())
    }

    fn delete_packet_commitment(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        seq: &Sequence,
    ) -> Result<(), PacketError> {
        self.packet_commitment
            .remove(&(port_id.clone(), channel_id.clone(), seq.clone()));
        Ok(())
    }

    fn store_packet_receipt(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        receipt: Receipt,
    ) -> Result<(), PacketError> {
        let packet_receipt_path = ReceiptsPath {
            port_id: port_id.clone(),
            channel_id: channel_id.clone(),
            sequence,
        }
        .to_string()
        .as_bytes()
        .to_vec();

        self.packet_receipt
            .insert(&(port_id.clone(), channel_id.clone(), sequence), &receipt);

        Ok(())
    }

    fn store_packet_acknowledgement(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        ack_commitment: AcknowledgementCommitment,
    ) -> Result<(), PacketError> {
        self.packet_acknowledgement.insert(
            &(port_id.clone(), channel_id.clone(), sequence.clone()),
            &ack_commitment,
        );

        Ok(())
    }

    fn delete_packet_acknowledgement(
        &mut self,
        port_id: &PortId,
        channel_id: &ChannelId,
        sequence: &Sequence,
    ) -> Result<(), PacketError> {
        self.packet_acknowledgement.remove(&(
            port_id.clone(),
            channel_id.clone(),
            sequence.clone(),
        ));
        Ok(())
    }

    fn store_connection_channels(
        &mut self,
        conn_id: ConnectionId,
        port_id: PortId,
        channel_id: ChannelId,
    ) -> Result<(), ChannelError> {
        let mut vec = self.connection_channels.get(&conn_id).unwrap_or_default();
        vec.push((port_id, channel_id));

        self.connection_channels.insert(&conn_id, &vec);

        Ok(())
    }

    /// Stores the given channel_end at a path associated with the port_id and channel_id.
    fn store_channel(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        channel_end: ChannelEnd,
    ) -> Result<(), ChannelError> {
        self.channels
            .insert(&(port_id.clone(), channel_id.clone()), &channel_end);

        Ok(())
    }

    fn store_next_sequence_send(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.next_sequence_send
            .insert(&(port_id.clone(), channel_id.clone()), &(seq.into()));
        Ok(())
    }

    fn store_next_sequence_recv(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.next_sequence_recv
            .insert(&(port_id.clone(), channel_id.clone()), &(seq.into()));

        Ok(())
    }

    fn store_next_sequence_ack(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        self.next_sequence_ack
            .insert(&(port_id.clone(), channel_id.clone()), &(seq.into()));

        Ok(())
    }

    /// Called upon channel identifier creation (Init or Try message processing).
    /// Increases the counter which keeps track of how many channels have been created.
    /// Should never fail.
    fn increase_channel_counter(&mut self) {
        self.channel_ids_counter = self
            .channel_ids_counter
            .checked_add(1)
            .expect(format!("increase channel counter overflow").as_str())
    }
}
