use core::str::FromStr;

use ibc::{
    clients::ics07_tendermint::{
        client_state::ClientState as Ics07ClientState,
        consensus_state::ConsensusState as Ics07ConsensusState,
    },
    core::{
        ics02_client::{
            client_state::ClientState,
            client_type::ClientType,
            consensus_state::ConsensusState,
            context::{ClientKeeper, ClientReader},
            error::Error as Ics02Error,
        },
        ics24_host::{
            identifier::ClientId,
            path::{ClientConsensusStatePath, ClientStatePath, ClientTypePath},
        },
    },
    timestamp::Timestamp,
    Height,
};
use ibc_proto::cosmos::gov::v1beta1::VoteOption::No;
use ibc_proto::{google::protobuf::Any, protobuf::Protobuf};

use crate::context::IbcContext;
use crate::ibc_impl::core::host::type_define::NearClientStatePath;
use crate::ibc_impl::core::host::TENDERMINT_CLIENT_TYPE;
use near_sdk::env;

impl ClientReader for IbcContext<'_> {
    /// Returns the ClientType for the given identifier `client_id`.
    fn client_type(&self, client_id: &ClientId) -> Result<ClientType, Ics02Error> {
        let client_type_path = ClientTypePath(client_id.clone())
            .to_string()
            .as_bytes()
            .to_vec();

        self.near_ibc_store
            .client_types
            .get(&(client_type_path))
            .ok_or(Ics02Error::client_not_found(client_id.clone()))
            .and_then(|near_client_type| {
                String::from_utf8(near_client_type)
                    .map_err(|_| Ics02Error::implementation_specific())
            })
            .and_then(|data| match data.as_str() {
                TENDERMINT_CLIENT_TYPE => Ok(ClientType::new(TENDERMINT_CLIENT_TYPE.to_string())),
                unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
            })
    }

    /// Returns the ClientState for the given identifier `client_id`.
    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, Ics02Error> {
        let client_state_path = ClientStatePath(client_id.clone())
            .to_string()
            .as_bytes()
            .to_vec();

        if let Some(client_state) = self.near_ibc_store.client_state.get(&(client_state_path)) {
            return match self.client_type(client_id)?.as_str() {
                TENDERMINT_CLIENT_TYPE => {
                    let result: Ics07ClientState = Protobuf::<Any>::decode_vec(&client_state)
                        .map_err(|e| {
                            Ics02Error::other(format!("Decode Ics07ClientState failed: {:?}", e))
                        })?;
                    Ok(Box::new(result))
                }
                unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
            };
        } else {
            Err(Ics02Error::client_not_found(client_id.clone()))
        }
    }

    /// Tries to decode the given `client_state` into a concrete light client state.
    fn decode_client_state(&self, client_state: Any) -> Result<Box<dyn ClientState>, Ics02Error> {
        if let Ok(client_state) = Ics07ClientState::try_from(client_state.clone()) {
            Ok(client_state.into_box())
        } else {
            Err(Ics02Error::unknown_client_state_type(client_state.type_url))
        }
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
        if let Some(data) = self.near_ibc_store.consensus_states.get(&height.into()) {
            match self.client_type(client_id)?.as_str() {
                TENDERMINT_CLIENT_TYPE => {
                    let result: Ics07ConsensusState = Protobuf::<Any>::decode_vec(&data)
                        .map_err(|_| Ics02Error::implementation_specific())?;
                    return Ok(Box::new(result));
                }
                unimplemented => {
                    return Err(Ics02Error::unknown_client_type(unimplemented.to_string()))
                }
            }
        } else {
            Err(Ics02Error::consensus_state_not_found(
                client_id.clone(),
                height,
            ))
        }
    }

    /// Search for the lowest consensus state higher than `height`.
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, Ics02Error> {
        if let Some(near_consensus_state) = self
            .near_ibc_store
            .consensus_states
            .get_next(&height.into())
        {
            match self.client_type(client_id)?.as_str() {
                TENDERMINT_CLIENT_TYPE => {
                    let result: Ics07ConsensusState =
                        Protobuf::<Any>::decode_vec(&near_consensus_state).map_err(|e| {
                            Ics02Error::other(format!("Decode Ics07ConsensusState failed: {:?}", e))
                        })?;
                    Ok(Some(Box::new(result)))
                }
                unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
            }
        } else {
            Ok(None)
        }
    }

    /// Search for the highest consensus state lower than `height`.
    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: Height,
    ) -> Result<Option<Box<dyn ConsensusState>>, Ics02Error> {
        if let Some(near_consensus_state) =
            self.near_ibc_store.consensus_states.get_pre(&height.into())
        {
            match self.client_type(client_id)?.as_str() {
                TENDERMINT_CLIENT_TYPE => {
                    let result: Ics07ConsensusState =
                        Protobuf::<Any>::decode_vec(&near_consensus_state).map_err(|e| {
                            Ics02Error::other(format!("Decode Ics07ConsensusState failed: {:?}", e))
                        })?;
                    Ok(Some(Box::new(result)))
                }
                unimplemented => Err(Ics02Error::unknown_client_type(unimplemented.to_string())),
            }
        } else {
            Ok(None)
        }
    }

    /// Returns the current height of the local chain.
    fn host_height(&self) -> Height {
        Height::new(env::epoch_height(), env::block_height()).unwrap()
    }

    /// Returns the `ConsensusState` of the host (local) chain at a specific height.
    /// todo impl this
    fn host_consensus_state(&self, height: Height) -> Result<Box<dyn ConsensusState>, Ics02Error> {
        Err(Ics02Error::implementation_specific())
    }

    /// Returns the pending `ConsensusState` of the host (local) chain.
    fn pending_host_consensus_state(&self) -> Result<Box<dyn ConsensusState>, Ics02Error> {
        Err(Ics02Error::implementation_specific())
    }

    /// Returns a natural number, counting how many clients have been created thus far.
    /// The value of this counter should increase only via method `ClientKeeper::increase_client_counter`.
    fn client_counter(&self) -> Result<u64, Ics02Error> {
        Ok(self.near_ibc_store.client_ids_counter)
    }
}

impl ClientKeeper for IbcContext<'_> {
    /// Called upon successful client creation
    fn store_client_type(
        &mut self,
        client_id: ClientId,
        client_type: ClientType,
    ) -> Result<(), Ics02Error> {
        let client_type_path = ClientTypePath(client_id).to_string().as_bytes().to_vec();

        self.near_ibc_store
            .client_types
            .insert(&client_type_path, &client_type.as_str().as_bytes().to_vec());

        Ok(())
    }

    /// Called upon successful client creation and update
    fn store_client_state(
        &mut self,
        client_id: ClientId,
        client_state: Box<dyn ClientState>,
    ) -> Result<(), Ics02Error> {
        let client_state_path = ClientStatePath(client_id).to_string().as_bytes().to_vec();

        let data = client_state
            .encode_vec()
            .map_err(|_| Ics02Error::implementation_specific())?;
        self.near_ibc_store
            .client_state
            .insert(&(client_state_path), &data);

        Ok(())
    }

    /// Called upon successful client creation and update
    fn store_consensus_state(
        &mut self,
        client_id: ClientId,
        height: Height,
        consensus_state: Box<dyn ConsensusState>,
    ) -> Result<(), Ics02Error> {
        let consensus_state = consensus_state
            .encode_vec()
            .map_err(|_| Ics02Error::implementation_specific())?;

        self.near_ibc_store
            .consensus_states
            .insert_from_tail(&height.into(), &consensus_state);

        Ok(())
    }

    /// Called upon client creation.
    /// Increases the counter which keeps track of how many clients have been created.
    /// Should never fail.
    fn increase_client_counter(&mut self) {
        self.near_ibc_store
            .client_ids_counter
            .checked_add(1)
            .expect("increase client counter overflow");
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
        self.near_ibc_store.client_processed_times.insert(
            &(client_id.as_bytes().to_vec(), height.into()),
            &timestamp.nanoseconds(),
        );
        Ok(())
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
        self.near_ibc_store.client_processed_heights.insert(
            &(client_id.as_bytes().to_vec(), height.into()),
            &host_height.into(),
        );
        Ok(())
    }
}
