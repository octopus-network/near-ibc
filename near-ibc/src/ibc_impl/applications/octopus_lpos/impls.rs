use super::OctopusLposModule;
use crate::{context::NearIbcStoreHost, prelude::*};
use core::str::FromStr;
use ibc::core::{
    ics04_channel::channel::ChannelEnd,
    ics24_host::{
        identifier::{ChannelId, PortId},
        path::ChannelEndPath,
    },
    ValidationContext,
};
use near_sdk::{ext_contract, json_types::U64, AccountId};
use octopus_lpos::{
    context::{OctopusLposExecutionContext, OctopusLposValidationContext},
    error::OctopusLposError,
    packet::consumer::SlashPacketData,
    ConsumerChainId, PORT_ID_STR,
};

/// The callback interface for `ext_transfer_request_handler`.
#[ext_contract(ext_octopus_appchain_anchor_ibc)]
pub trait OctopusAppchainAnchorIbc {
    /// Interface for near-ibc to call when slash packet is received.
    fn slash_validator(&mut self, slach_packet_data: SlashPacketData);
    /// Interface for near-ibc to call when vsc_matured packet is received.
    fn on_vsc_matured(&mut self, validator_set_id: U64);
    /// Interface for near-ibc to call when distribute_reward packet is received.
    fn distribute_reward(&mut self, validator_set_id: U64);
}

impl NearIbcStoreHost for OctopusLposModule {}

impl OctopusLposValidationContext for OctopusLposModule {
    fn get_port(&self) -> Result<PortId, OctopusLposError> {
        Ok(PortId::from_str(PORT_ID_STR).unwrap())
    }

    fn get_channel_end(
        &self,
        consumer_chain_id: &ConsumerChainId,
    ) -> Result<(PortId, ChannelId, ChannelEnd), OctopusLposError> {
        let near_ibc_store = OctopusLposModule::get_near_ibc_store();
        let port_id = self.get_port()?;
        let channel_id = self
            .chain_id_channel_map
            .get(consumer_chain_id)
            .ok_or_else(|| OctopusLposError::InvalidConsumerChainId {
                chain_id: consumer_chain_id.clone(),
            })?;
        let channel_end = near_ibc_store.channel_end(&ChannelEndPath::new(&port_id, channel_id))?;
        Ok((port_id, channel_id.clone(), channel_end))
    }

    fn get_consumer_chain_id(
        &self,
        channel_id: &ChannelId,
    ) -> Result<ConsumerChainId, OctopusLposError> {
        self.chain_id_channel_map
            .into_iter()
            .find(|id| id.1.clone() == *channel_id)
            .map(|id| id.0.clone())
            .ok_or_else(|| OctopusLposError::InvalidChannelId {
                channel_id: channel_id.clone(),
            })
    }
}

impl OctopusLposExecutionContext for OctopusLposModule {
    fn slash_validator(
        &mut self,
        consumer_chain_id: &ConsumerChainId,
        slach_packet_data: SlashPacketData,
    ) -> Result<(), OctopusLposError> {
        let anchor_account_id = format!(
            "{}.{}",
            consumer_chain_id.as_str(),
            self.appchain_registry_account.to_string()
        );
        ext_octopus_appchain_anchor_ibc::ext(
            AccountId::try_from(anchor_account_id.clone()).map_err(|_| {
                OctopusLposError::Unexpected {
                    description: format!("Invalid anchor account id: {}", anchor_account_id),
                }
            })?,
        )
        .with_attached_deposit(0)
        .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 10)
        .with_unused_gas_weight(0)
        .slash_validator(slach_packet_data);
        Ok(())
    }

    fn on_vsc_matured(
        &mut self,
        consumer_chain_id: &ConsumerChainId,
        validator_set_id: u64,
    ) -> Result<(), OctopusLposError> {
        let anchor_account_id = format!(
            "{}.{}",
            consumer_chain_id.as_str(),
            self.appchain_registry_account.to_string()
        );
        ext_octopus_appchain_anchor_ibc::ext(
            AccountId::try_from(anchor_account_id.clone()).map_err(|_| {
                OctopusLposError::Unexpected {
                    description: format!("Invalid anchor account id: {}", anchor_account_id),
                }
            })?,
        )
        .with_attached_deposit(0)
        .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 6)
        .with_unused_gas_weight(0)
        .on_vsc_matured(U64::from(validator_set_id));
        Ok(())
    }

    fn distribute_reward(
        &mut self,
        consumer_chain_id: &ConsumerChainId,
        validator_set_id: u64,
    ) -> Result<(), OctopusLposError> {
        let anchor_account_id = format!(
            "{}.{}",
            consumer_chain_id.as_str(),
            self.appchain_registry_account.to_string()
        );
        ext_octopus_appchain_anchor_ibc::ext(
            AccountId::try_from(anchor_account_id.clone()).map_err(|_| {
                OctopusLposError::Unexpected {
                    description: format!("Invalid anchor account id: {}", anchor_account_id),
                }
            })?,
        )
        .with_attached_deposit(0)
        .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 10)
        .with_unused_gas_weight(0)
        .distribute_reward(U64::from(validator_set_id));
        Ok(())
    }
}
