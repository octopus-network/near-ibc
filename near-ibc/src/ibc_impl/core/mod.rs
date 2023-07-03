use ibc::{
    clients::ics07_tendermint::client_state::ClientState as TmClientState,
    core::ics02_client::error::ClientError,
};
use ibc_proto::{
    google::protobuf::Any, ibc::lightclients::tendermint::v1::ClientState as RawTmClientState,
    protobuf::Protobuf,
};
use serde::{Deserialize, Serialize};

mod execution_context;
pub mod host;
mod router;
mod validation_context;

const TENDERMINT_CLIENT_STATE_TYPE_URL: &str = "/ibc.lightclients.tendermint.v1.ClientState";

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum AnyClientState {
    Tendermint(TmClientState),
}

impl Protobuf<Any> for AnyClientState {}

impl TryFrom<Any> for AnyClientState {
    type Error = ClientError;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            "" => Err(ClientError::UnknownClientStateType {
                client_state_type: raw.type_url,
            }),
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
            AnyClientState::Tendermint(value) => Any {
                type_url: TENDERMINT_CLIENT_STATE_TYPE_URL.to_string(),
                value: Protobuf::<RawTmClientState>::encode_vec(&value),
            },
        }
    }
}
