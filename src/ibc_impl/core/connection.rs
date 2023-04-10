use crate::{context::IbcContext, DEFAULT_COMMITMENT_PREFIX};

// use ibc::core::ics03_connection::error::Error as ConnectionError;

use ibc::{
    core::{
        ics02_client::{
            client_state::ClientState, consensus_state::ConsensusState, context::ClientReader,
        },
        ics03_connection::{
            connection::ConnectionEnd,
            context::{ConnectionKeeper, ConnectionReader},
            error::ConnectionError,
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
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, ConnectionError> {
        self.near_ibc_store
            .connections
            .get(&conn_id)
            .ok_or(ConnectionError::ConnectionMismatch {
                connection_id: conn_id.clone(),
            })
    }

    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ConnectionError> {
        ClientReader::client_state(self, client_id)
            // .map_err(ConnectionError::ics02_client)
            .map_err(|e| ConnectionError::Client(e))
    }

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(
        &self,
        client_state: Any,
    ) -> Result<Box<dyn ClientState>, ConnectionError> {
        ClientReader::decode_client_state(self, client_state)
            .map_err(|e| ConnectionError::Client(e))
    }

    /// Returns the current height of the local chain.
    fn host_current_height(&self) -> Result<Height, ConnectionError> {
        Height::new(env::epoch_height(), env::block_height())
            .map_err(|e| ConnectionError::Client(e))
    }

    /// Returns the oldest height available on the local chain.
    fn host_oldest_height(&self) -> Result<Height, ConnectionError> {
        todo!()
    }

    /// Returns the prefix that the local chain uses in the KV store.
    fn commitment_prefix(&self) -> CommitmentPrefix {
        CommitmentPrefix::try_from(DEFAULT_COMMITMENT_PREFIX.as_bytes().to_vec())
            .unwrap_or_default()
    }

    /// Returns the ConsensusState that the given client stores at a specific height.
    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ConnectionError> {
        self.consensus_state(client_id, height)
            .map_err(|e| ConnectionError::Client(e))
    }

    /// Returns the ConsensusState of the host (local) chain at a specific height.
    fn host_consensus_state(
        &self,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, ConnectionError> {
        ClientReader::host_consensus_state(self, height).map_err(ConnectionError::Client)
    }

    /// Returns a counter on how many connections have been created thus far.
    /// The value of this counter should increase only via method
    /// `ConnectionKeeper::increase_connection_counter`.
    fn connection_counter(&self) -> Result<u64, ConnectionError> {
        Ok(self.near_ibc_store.connection_ids_counter)
    }

    fn validate_self_client(&self, counterparty_client_state: Any) -> Result<(), ConnectionError> {
        Ok(())
    }
}

impl ConnectionKeeper for IbcContext<'_> {
    /// Stores the given connection_end at a path associated with the connection_id.
    fn store_connection(
        &mut self,
        connection_id: ConnectionId,
        connection_end: ConnectionEnd,
    ) -> Result<(), ConnectionError> {
        self.near_ibc_store
            .connections
            .insert(&connection_id, &connection_end);
        Ok(())
    }

    /// Stores the given connection_id at a path associated with the client_id.
    fn store_connection_to_client(
        &mut self,
        connection_id: ConnectionId,
        client_id: ClientId,
    ) -> Result<(), ConnectionError> {
        self.near_ibc_store
            .client_connections
            .insert(&client_id, &connection_id);
        Ok(())
    }

    /// Called upon connection identifier creation (Init or Try process).
    /// Increases the counter which keeps track of how many connections have been created.
    /// Should never fail.
    fn increase_connection_counter(&mut self) {
        self.near_ibc_store.connection_ids_counter = self
            .near_ibc_store
            .connection_ids_counter
            .checked_add(1)
            .expect("increase connection counter overflow");
    }
}
