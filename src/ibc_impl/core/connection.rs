use crate::context::IbcContext;

// use ibc::core::ics03_connection::error::Error as Ics03Error;

use ibc::relayer::ics18_relayer::context::Ics18Context;
use ibc::{
    core::{
        ics02_client::{
            client_state::ClientState, consensus_state::ConsensusState, context::ClientReader,
        },
        ics03_connection::{
            connection::ConnectionEnd,
            context::{ConnectionKeeper, ConnectionReader},
            error::Error as Ics03Error,
        },
        ics23_commitment::commitment::CommitmentPrefix,
        ics24_host::{
            identifier::{ClientId, ConnectionId},
            path::{ClientConnectionsPath, ConnectionsPath},
        },
    },
    Height,
};
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};
use near_sdk::env;

impl ConnectionReader for IbcContext<'_> {
    /// Returns the ConnectionEnd for the given identifier `conn_id`.
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
        self.near_ibc_store
            .connections
            .get(&conn_id.as_bytes().to_vec().into())
            .ok_or(Ics03Error::connection_mismatch(conn_id.clone()))
            .and_then(|data| {
                ConnectionEnd::decode_vec(&data)
                    .map_err(|e| Ics03Error::other(format!("Decode ConnectionEnd failed: {:?}", e)))
            })
    }

    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics03Error> {
        ClientReader::client_state(self, client_id).map_err(Ics03Error::ics02_client)
    }

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, Ics03Error> {
        ClientReader::decode_client_state(self, client_state).map_err(Ics03Error::ics02_client)
    }

    /// Returns the current height of the local chain.
    fn host_current_height(&self) -> Height {
        Height::new(env::epoch_height(), env::block_height()).unwrap()
    }

    /// Returns the oldest height available on the local chain.
    fn host_oldest_height(&self) -> Height {
        todo!()
    }

    /// Returns the prefix that the local chain uses in the KV store.
    fn commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from(b"Ibc".to_vec()).unwrap_or_default()
    }

    /// Returns the ConsensusState that the given client stores at a specific height.
    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Box<dyn ConsensusState>, Ics03Error> {
        self.consensus_state(client_id, height)
            .map_err(Ics03Error::ics02_client)
    }

    /// Returns the ConsensusState of the host (local) chain at a specific height.
    fn host_consensus_state(&self, height: Height) -> Result<Box<dyn ConsensusState>, Ics03Error> {
        ClientReader::host_consensus_state(self, height).map_err(Ics03Error::ics02_client)
    }

    /// Returns a counter on how many connections have been created thus far.
    /// The value of this counter should increase only via method
    /// `ConnectionKeeper::increase_connection_counter`.
    fn connection_counter(&self) -> Result<u64, Ics03Error> {
        Ok(self.near_ibc_store.connection_ids_counter)
    }

    fn validate_self_client(&self, counterparty_client_state: Any) -> Result<(), Ics03Error> {
        Ok(())
    }
}

impl ConnectionKeeper for IbcContext<'_> {
    /// Stores the given connection_end at a path associated with the connection_id.
    fn store_connection(
        &mut self,
        connection_id: ConnectionId,
        connection_end: &ConnectionEnd,
    ) -> Result<(), Ics03Error> {
        let data = connection_end
            .encode_vec()
            .map_err(|e| Ics03Error::other(format!("Encode ConnectionEnd failed: {:?}", e)))?;
        self.near_ibc_store
            .connections
            .insert(&connection_id.as_bytes().to_vec().into(), &data);
        Ok(())
    }

    /// Stores the given connection_id at a path associated with the client_id.
    fn store_connection_to_client(
        &mut self,
        connection_id: ConnectionId,
        client_id: &ClientId,
    ) -> Result<(), Ics03Error> {
        self.near_ibc_store.client_connections.insert(
            &client_id.as_bytes().to_vec(),
            &connection_id.as_bytes().to_vec().into(),
        );
        Ok(())
    }

    /// Called upon connection identifier creation (Init or Try process).
    /// Increases the counter which keeps track of how many connections have been created.
    /// Should never fail.
    fn increase_connection_counter(&mut self) {
        self.near_ibc_store.connection_ids_counter += 1
    }
}
