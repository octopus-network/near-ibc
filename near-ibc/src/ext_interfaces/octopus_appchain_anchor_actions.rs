use crate::*;
use ibc::{
    clients::ics07_tendermint::{
        client_state::ClientState as TmClientState,
        consensus_state::ConsensusState as TmConsensusState,
    },
    core::ics02_client::msgs::{create_client::MsgCreateClient, ClientMsg},
};

pub trait OctopusAppchainAnchorActions {
    /// Create client for the corresponding Octopus appchain.
    fn create_client_for_appchain(
        &mut self,
        client_state: TmClientState,
        consensus_state: TmConsensusState,
    );
    /// Send a VSC packet to the corresponding Octopus appchain.
    fn send_vsc_packet(&mut self, vsc_packet_data: VscPacketData);
}

#[near_bindgen]
impl OctopusAppchainAnchorActions for NearIbcContract {
    //
    fn create_client_for_appchain(
        &mut self,
        client_state: TmClientState,
        consensus_state: TmConsensusState,
    ) {
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let predecessor_account_id = env::predecessor_account_id().to_string();
        let (_, parent_account) = predecessor_account_id.split_once(".").unwrap();
        assert!(
            parent_account
                == self
                    .module_holder
                    .octopus_lpos_module
                    .appchain_registry_account
                    .to_string()
                    .as_str(),
            "ERR_INVALID_CALLER, only octopus appchain anchor accounts can call this function."
        );
        let msg = MsgCreateClient {
            client_state: client_state.into(),
            consensus_state: consensus_state.into(),
            signer: Signer::from(env::current_account_id().to_string()),
        };
        match ibc::core::dispatch(
            &mut near_ibc_store,
            self,
            MsgEnvelope::Client(ClientMsg::CreateClient(msg)),
        ) {
            Ok(()) => (),
            Err(e) => {
                log!("Error occurred in client creation: {:?}", e);
            }
        }
        near_ibc_store.flush();
        self.near_ibc_store.set(&near_ibc_store);
    }
    //
    fn send_vsc_packet(&mut self, vsc_packet_data: VscPacketData) {
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let predecessor_account_id = env::predecessor_account_id().to_string();
        let (appchain_id, parent_account) = predecessor_account_id.split_once(".").unwrap();
        assert!(
            parent_account
                == self
                    .module_holder
                    .octopus_lpos_module
                    .appchain_registry_account
                    .to_string()
                    .as_str(),
            "ERR_INVALID_CALLER, only octopus appchain anchor accounts can call this function."
        );
        if let Err(e) = octopus_lpos::send_vsc_packet(
            &mut near_ibc_store,
            &mut self.module_holder.octopus_lpos_module,
            MsgValidatorSetChange {
                chain_id: appchain_id.to_string(),
                packet_data: octopus_lpos::packet::vsc::VscPacketData {
                    validator_pubkeys: vsc_packet_data
                        .validator_pubkeys
                        .into_iter()
                        .map(
                            |validator_key_and_power| octopus_lpos::packet::vsc::ValidatorUpdate {
                                public_key: tendermint::PublicKey::Ed25519(
                                    tendermint::crypto::ed25519::VerificationKey::try_from(
                                        validator_key_and_power.public_key.get(1..).unwrap(),
                                    )
                                    .expect("ERR_INVALID_PUBLIC_KEY"),
                                ),
                                power: validator_key_and_power.power.0,
                            },
                        )
                        .collect(),
                    validator_set_id: vsc_packet_data.validator_set_id.0,
                    slash_acks: vsc_packet_data
                        .slash_acks
                        .into_iter()
                        .map(|bytes| hex::encode(&bytes))
                        .collect(),
                },
                timeout_height_on_b: TimeoutHeight::Never,
                timeout_timestamp_on_b: Timestamp::none(),
            },
        ) {
            log!("ERR_SEND_VSC_PACKET: {:?}", e);
        }
    }
}
