use crate::NearContext;
use ibc::core::ics02_client::client_state::ClientState;
use ibc::core::ics02_client::client_type::ClientType;
use ibc::core::ics02_client::consensus_state::ConsensusState;
use ibc::core::ics02_client::context::{ClientKeeper, ClientReader};
use ibc::core::ics02_client::error::Error as Ics02Error;
use ibc::core::ics24_host::identifier::ClientId;
use ibc::timestamp::Timestamp;
use ibc::Height;
use ibc_proto::google::protobuf::Any;

impl ClientReader for NearContext {
    /// Returns the ClientType for the given identifier `client_id`.
    fn client_type(&self, client_id: &ClientId) -> Result<ClientType, Ics02Error> {
        todo!()
    }

    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics02Error> {
        todo!()
    }

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, Ics02Error> {
        todo!()
    }

    /// Retrieve the consensus state for the given client ID at the specified
    /// height.
    ///
    /// Returns an error if no such state exists.
    fn consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Box<dyn ConsensusState>, Ics02Error> {
        todo!()
    }

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, Ics02Error> {
        todo!()
    }

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, Ics02Error> {
        todo!()
    }

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Height {
        todo!()
    }

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    fn host_consensus_state(&self, height: Height) -> Result<Box<dyn ConsensusState>, Ics02Error> {
        todo!()
    }

    /// Returns the pending `ConsensusState` of the host (local) chain.
    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, Ics02Error> {
        todo!()
    }

    /// Returns a natural number, counting how many clients have been created thus far.
    /// The value of this counter should increase only via method `ClientKeeper::increase_client_counter`.
    fn client_counter(&self) -> Result<u64, Ics02Error> {
        todo!()
    }
}

impl ClientKeeper for NearContext {
    /// Called upon successful client creation
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), Ics02Error> {
        todo!()
    }

    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: Box<dyn ClientState>,
    ) -> Result<(), Ics02Error> {
        todo!()
    }

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: Box<dyn ConsensusState>,
    ) -> Result<(), Ics02Error> {
        todo!()
    }

    /// Called upon client creation.
    /// Increases the counter which keeps track of how many clients have been created.
    /// Should never fail.
    fn increase_client_counter(&mut self) {
        todo!()
    }

    /// Called upon successful client update.
    /// Implementations are expected to use this to record the specified time as the time at which
    /// this update (or header) was processed.
    fn store_update_time(
        &mut self,
        client_id: ClientId,
        height: Height,
        timestamp: Timestamp,
    ) -> Result<(), Ics02Error> {
        todo!()
    }

    /// Called upon successful client update.
    /// Implementations are expected to use this to record the specified height as the height at
    /// at which this update (or header) was processed.
    fn store_update_height(
        &mut self,
        client_id: ClientId,
        height: Height,
        host_height: Height,
    ) -> Result<(), Ics02Error> {
        todo!()
    }
}
