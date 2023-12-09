use crate::{
    ibc_impl::applications::{octopus_lpos::OctopusLposModule, transfer::TransferModule},
    prelude::*,
};
use ibc::core::{host::types::identifiers::PortId, router::types::module::ModuleId};
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    AccountId,
};

/// A simple struct for supporting the mutable borrow in `Router::get_route_mut`.
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct ModuleHolder {
    pub transfer_module: TransferModule,
    pub octopus_lpos_module: OctopusLposModule,
}

impl ModuleHolder {
    pub fn new(appchain_registry_account: AccountId) -> Self {
        Self {
            transfer_module: TransferModule(),
            octopus_lpos_module: OctopusLposModule::new(appchain_registry_account),
        }
    }
    ///
    pub fn get_module_id(&self, port_id: &PortId) -> Option<ModuleId> {
        match port_id.as_str() {
            ibc::apps::transfer::types::PORT_ID_STR => Some(ModuleId::new(
                ibc::apps::transfer::types::MODULE_ID_STR.to_string(),
            )),
            octopus_lpos::PORT_ID_STR => {
                Some(ModuleId::new(octopus_lpos::MODULE_ID_STR.to_string()))
            }
            _ => None,
        }
    }
}
