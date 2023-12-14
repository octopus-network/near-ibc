use super::consensus_state::AnyConsensusState;
use crate::context::NearEd25519Verifier;
use crate::{collections::IndexedAscendingQueueViewer, context::NearIbcStore, prelude::*};
use ibc::core::ics02_client::client_state::Status;
use ibc::{
    clients::ics07_tendermint::client_state::ClientState as TmClientState,
    core::{
        ics02_client::{
            client_state::{
                ClientStateCommon, ClientStateExecution, ClientStateValidation, UpdateKind,
            },
            client_type::ClientType,
            error::ClientError,
        },
        ics23_commitment::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
        ics24_host::{
            identifier::ClientId,
            path::{ClientConsensusStatePath, Path},
        },
        timestamp::Timestamp,
        ContextError, ValidationContext,
    },
    Height,
};
use ibc_proto::{
    google::protobuf::Any, ibc::lightclients::tendermint::v1::ClientState as RawTmClientState,
    protobuf::Protobuf,
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
                value: Protobuf::<RawTmClientState>::encode_vec(&client_state),
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

impl ibc::clients::ics07_tendermint::CommonContext for NearIbcStore {
    type ConversionError = ClientError;

    type AnyConsensusState = AnyConsensusState;

    fn consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Self::AnyConsensusState, ContextError> {
        ValidationContext::consensus_state(self, client_cons_state_path)
    }
}

impl ibc::clients::ics07_tendermint::ValidationContext for NearIbcStore {
    fn host_timestamp(&self) -> Result<Timestamp, ContextError> {
        ValidationContext::host_timestamp(self)
    }

    fn next_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Option<Self::AnyConsensusState>, ContextError> {
        if let Some(consensus_state_keys) = self.client_consensus_state_height_sets.get(client_id) {
            consensus_state_keys
                .get_next_key_by_key(height)
                .map(|next_height| {
                    self.consensus_state(&ClientConsensusStatePath::new(client_id, next_height))
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
            consensus_state_keys
                .get_previous_key_by_key(height)
                .map(|next_height| {
                    self.consensus_state(&ClientConsensusStatePath::new(client_id, next_height))
                })
                .map_or_else(|| Ok(None), |cs| Ok(Some(cs.unwrap())))
        } else {
            Err(ContextError::ClientError(
                ClientError::MissingRawConsensusState,
            ))
        }
    }
}
