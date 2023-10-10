use crate::{
    ibc_impl::applications::{octopus_lpos::OctopusLposModule, transfer::TransferModule},
    prelude::*,
    StorageKey,
};
use ibc::core::{ics24_host::identifier::PortId, router::ModuleId};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    AccountId,
};

/// A simple struct for supporting the mutable borrow in `Router::get_route_mut`.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ModuleHolder {
    pub transfer_module: TransferModule,
    pub octopus_lpos_module: LazyOption<OctopusLposModule>,
}

impl ModuleHolder {
    pub fn new(appchain_registry_account: AccountId) -> Self {
        Self {
            transfer_module: TransferModule(),
            octopus_lpos_module: LazyOption::new(
                StorageKey::OctopusLposModule,
                Some(&OctopusLposModule::new(appchain_registry_account)),
            ),
        }
    }
    ///
    pub fn get_module_id(&self, port_id: &PortId) -> Option<ModuleId> {
        match port_id.as_str() {
            ibc::applications::transfer::PORT_ID_STR => Some(ModuleId::new(
                ibc::applications::transfer::MODULE_ID_STR.to_string(),
            )),
            _ => None,
        }
    }
}
