use crate::{ibc_impl::applications::transfer::TransferModule, prelude::*};
use ibc::core::{ics24_host::identifier::PortId, router::ModuleId};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

/// A simple struct for supporting the mutable borrow in `Router::get_route_mut`.
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ModuleHolder {
    pub transfer_module: TransferModule,
}

impl ModuleHolder {
    pub fn new() -> Self {
        Self {
            transfer_module: TransferModule(),
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

impl Default for ModuleHolder {
    fn default() -> Self {
        Self::new()
    }
}
