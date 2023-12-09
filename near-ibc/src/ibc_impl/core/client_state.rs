use super::consensus_state::AnyConsensusState;
use crate::context::NearEd25519Verifier;
use crate::{context::NearIbcStore, prelude::*};
use ibc::{
    clients::tendermint::client_state::ClientState as TmClientState,
    core::{
        client::context::{
            client_state::{ClientStateCommon, ClientStateExecution, ClientStateValidation},
            types::{error::ClientError, Height, Status, UpdateKind},
            ClientValidationContext,
        },
        commitment_types::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
        handler::types::error::ContextError,
        host::{
            types::{
                identifiers::{ClientId, ClientType},
                path::{ClientConsensusStatePath, Path},
            },
            ValidationContext,
        },
    },
    primitives::Timestamp,
};
use ibc_proto::{
    google::protobuf::Any, ibc::lightclients::tendermint::v1::ClientState as RawTmClientState,
    Protobuf,
};
use serde::{Deserialize, Serialize};

const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum AnyClientState {
    Tendermint(TmClientState<NearEd25519Verifier>),
}

impl Protobuf<Any> for AnyClientState {}

impl TryFrom<Any> for AnyClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            TENDERMINT_CLIENT_STATE_TYPE_URL => Ok(AnyClientState::Tendermint(
                Protobuf::<RawTmClientState>::decode_vec(&raw.value).map_err(|e| {
                    ClientError::ClientSpecific {
                        description: e.to_string(),
                    }
                })?,
            )),
            _ => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
        }
    }
}

impl From<AnyClientState> for Any {
    fn from(value: AnyClientState) -> Self {
        match value {
            AnyClientState::Tendermint(client_state) => Any {
                type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
                value: Protobuf::<RawTmClientState>::encode_vec(client_state),
            },
        }
    }
}

impl ClientStateValidation<NearIbcStore> for AnyClientState {
    fn verify_client_message(
        &self,
        ctx: &NearIbcStore,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.verify_client_message(ctx, client_id, client_message, update_kind)
            }
        }
    }

    fn check_for_misbehaviour(
        &self,
        ctx: &NearIbcStore,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<bool, ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.check_for_misbehaviour(ctx, client_id, client_message, update_kind)
            }
        }
    }
    fn status(&self, ctx: &NearIbcStore, client_id: &ClientId) -> Result<Status, ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => client_state.status(ctx, client_id),
        }
    }
}

impl ClientStateCommon for AnyClientState {
    fn verify_consensus_state(&self, consensus_state: Any) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.verify_consensus_state(consensus_state)
            }
        }
    }

    fn client_type(&self) -> ClientType {
        match self {
            AnyClientState::Tendermint(client_state) => client_state.client_type(),
        }
    }

    fn latest_height(&self) -> Height {
        match self {
            AnyClientState::Tendermint(client_state) => client_state.latest_height(),
        }
    }

    fn validate_proof_height(&self, proof_height: Height) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.validate_proof_height(proof_height)
            }
        }
    }

    fn verify_upgrade_client(
        &self,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
        proof_upgrade_client: CommitmentProofBytes,
        proof_upgrade_consensus_state: CommitmentProofBytes,
        root: &CommitmentRoot,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => client_state.verify_upgrade_client(
                upgraded_client_state,
                upgraded_consensus_state,
                proof_upgrade_client,
                proof_upgrade_consensus_state,
                root,
            ),
        }
    }

    fn verify_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
        value: Vec<u8>,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.verify_membership(prefix, proof, root, path, value)
            }
        }
    }

    fn verify_non_membership(
        &self,
        prefix: &CommitmentPrefix,
        proof: &CommitmentProofBytes,
        root: &CommitmentRoot,
        path: Path,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.verify_non_membership(prefix, proof, root, path)
            }
        }
    }
}

impl From<TmClientState<NearEd25519Verifier>> for AnyClientState {
    fn from(value: TmClientState<NearEd25519Verifier>) -> Self {
        AnyClientState::Tendermint(value)
    }
}

impl ClientStateExecution<NearIbcStore> for AnyClientState {
    fn initialise(
        &self,
        ctx: &mut NearIbcStore,
        client_id: &ClientId,
        consensus_state: Any,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.initialise(ctx, client_id, consensus_state)
            }
        }
    }

    fn update_state(
        &self,
        ctx: &mut NearIbcStore,
        client_id: &ClientId,
        header: Any,
    ) -> Result<Vec<Height>, ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => {
                client_state.update_state(ctx, client_id, header)
            }
        }
    }

    fn update_state_on_misbehaviour(
        &self,
        ctx: &mut NearIbcStore,
        client_id: &ClientId,
        client_message: Any,
        update_kind: &UpdateKind,
    ) -> Result<(), ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => client_state.update_state_on_misbehaviour(
                ctx,
                client_id,
                client_message,
                update_kind,
            ),
        }
    }

    fn update_state_on_upgrade(
        &self,
        ctx: &mut NearIbcStore,
        client_id: &ClientId,
        upgraded_client_state: Any,
        upgraded_consensus_state: Any,
    ) -> Result<Height, ClientError> {
        match self {
            AnyClientState::Tendermint(client_state) => client_state.update_state_on_upgrade(
                ctx,
                client_id,
                upgraded_client_state,
                upgraded_consensus_state,
            ),
        }
    }
}

impl ibc::clients::tendermint::context::CommonContext for NearIbcStore {
    type ConversionError = ClientError;

    type AnyConsensusState = AnyConsensusState;

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        ValidationContext::consensus_state(self, client_cons_state_path)
    }

    fn consensus_state_heights(&self, client_id: &ClientId) -> Result<Vec<Height>, ContextError> {
        Ok(self
            .client_consensus_state_height_sets
            .get(&client_id)
            .map_or_else(
                || Vec::new(),
                |heights| heights.iter().map(|height| height.clone()).collect(),
            ))
    }

    fn host_height(&self) -> Result<Height, ContextError> {
        ValidationContext::host_height(self)
    }

    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        ValidationContext::host_timestamp(self)
    }
}

impl ibc::clients::tendermint::context::ValidationContext for NearIbcStore {
    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        if let Some(consensus_state_keys) = self.client_consensus_state_height_sets.get(client_id) {
            get_next_height(
                height,
                consensus_state_keys.iter().map(|h| h.clone()).collect(),
            )
            .map(|next_height| {
                self.consensus_state(&ClientConsensusStatePath::new(
                    client_id.clone(),
                    next_height.revision_number(),
                    next_height.revision_height(),
                ))
            })
            .map_or_else(|| Ok(None), |cs| Ok(Some(cs.unwrap())))
        } else {
            Err(ContextError::ClientError(
                ClientError::MissingRawConsensusState,
            ))
        }
    }

    fn prev_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        if let Some(consensus_state_keys) = self.client_consensus_state_height_sets.get(client_id) {
            get_previous_height(
                height,
                consensus_state_keys.iter().map(|h| h.clone()).collect(),
            )
            .map(|next_height| {
                self.consensus_state(&ClientConsensusStatePath::new(
                    client_id.clone(),
                    next_height.revision_number(),
                    next_height.revision_height(),
                ))
            })
            .map_or_else(|| Ok(None), |cs| Ok(Some(cs.unwrap())))
        } else {
            Err(ContextError::ClientError(
                ClientError::MissingRawConsensusState,
            ))
        }
    }
}

impl ClientValidationContext for NearIbcStore {
    fn client_update_time(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Timestamp, ContextError> {
        self.client_processed_times
            .get(client_id)
            .and_then(|processed_times| processed_times.get(height))
            .map(|ts| Timestamp::from_nanoseconds(*ts).unwrap())
            .ok_or_else(|| {
                ContextError::ClientError(ClientError::Other {
                    description: format!(
                        "Client update time not found. client_id: {}, height: {}",
                        client_id, height
                    ),
                })
            })
    }

    fn client_update_height(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Height, ContextError> {
        self.client_processed_heights
            .get(client_id)
            .and_then(|processed_heights| processed_heights.get(height))
            .map(|height: &Height| height.clone())
            .ok_or_else(|| {
                ContextError::ClientError(ClientError::Other {
                    description: format!(
                        "Client update height not found. client_id: {}, height: {}",
                        client_id, height
                    ),
                })
            })
    }
}

fn get_previous_height(height: &Height, heights: Vec<Height>) -> Option<Height> {
    let mut heights = heights;
    heights.sort();
    heights.reverse();
    heights
        .iter()
        .find(|h| **h < *height)
        .and_then(|h| Some(h.clone()))
}

fn get_next_height(height: &Height, heights: Vec<Height>) -> Option<Height> {
    let mut heights = heights;
    heights.sort();
    heights
        .iter()
        .find(|h| **h > *height)
        .and_then(|h| Some(h.clone()))
}

#[cfg(test)]
mod tests {
    use ibc::core::client::types::Height;

    #[test]
    fn test_get_previous_next_height() {
        let heights = vec![
            Height::new(0, 6).unwrap(),
            Height::new(0, 1).unwrap(),
            Height::new(0, 2).unwrap(),
            Height::new(0, 3).unwrap(),
            Height::new(0, 4).unwrap(),
            Height::new(0, 5).unwrap(),
        ];
        let height = Height::new(0, 3).unwrap();
        assert!(
            super::get_previous_height(&height, heights.clone()).unwrap()
                == Height::new(0, 2).unwrap()
        );
        assert!(
            super::get_next_height(&height, heights.clone()).unwrap() == Height::new(0, 4).unwrap()
        );
    }
}
