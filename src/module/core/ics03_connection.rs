use crate::NearContext;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::consensus_state::ConsensusState;
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics03_connection::context::{ConnectionKeeper, ConnectionReader};
use ibc::core::ics03_connection::error::Error as Ics03Error;
use ibc::core::ics23_commitment::commitment::CommitmentPrefix;
use ibc::core::ics24_host::identifier::{ClientId, ConnectionId};
use ibc::Height;
use ibc_proto::google::protobuf::Any;

impl ConnectionReader for NearContext {
    /// Returns the ConnectionEnd for the given identifier `conn_id`.
    fn connection_end(&self, conn_id: &ConnectionId) -> Result<ConnectionEnd, Ics03Error> {
        todo!()
    }

    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics03Error> {
        todo!()
    }

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, Ics03Error> {
        todo!()
    }

    /// Returns the current height of the local chain.
    fn host_current_height(&self) -> Height {
        todo!()
    }

    /// Returns the oldest height available on the local chain.
    fn host_oldest_height(&self) -> Height {
        todo!()
    }

    /// Returns the prefix that the local chain uses in the KV store.
    fn commitment_prefix(&self) -> CommitmentPrefix {
        todo!()
    }

    /// Returns the ConsensusState that the given client stores at a specific height.
    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Box<dyn ConsensusState>, Ics03Error> {
        todo!()
    }

    /// Returns the ConsensusState of the host (local) chain at a specific height.
    fn host_consensus_state(&self, height: Height) -> Result<Box<dyn ConsensusState>, Ics03Error> {
        todo!()
    }

    /// Returns a counter on how many connections have been created thus far.
    /// The value of this counter should increase only via method
    /// `ConnectionKeeper::increase_connection_counter`.
    fn connection_counter(&self) -> Result<u64, Ics03Error> {
        todo!()
    }
}

impl ConnectionKeeper for NearContext {
    /// Stores the given connection_end at a path associated with the connection_id.
    fn store_connection(
        &mut self,
        connection_id: ConnectionId,
        connection_end: &ConnectionEnd,
    ) -> Result<(), Ics03Error> {
        todo!()
    }

    /// Stores the given connection_id at a path associated with the client_id.
    fn store_connection_to_client(
        &mut self,
        connection_id: ConnectionId,
        client_id: &ClientId,
    ) -> Result<(), Ics03Error> {
        todo!()
    }

    /// Called upon connection identifier creation (Init or Try process).
    /// Increases the counter which keeps track of how many connections have been created.
    /// Should never fail.
    fn increase_connection_counter(&mut self) {
        todo!()
    }
}
