use crate::{ibc_impl::applications::transfer::TransferModule, prelude::*};
use ibc::core::{host::types::identifiers::PortId, router::types::module::ModuleId};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};

/// A simple struct for supporting the mutable borrow in `Router::get_route_mut`.
#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
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
            ibc::apps::transfer::types::PORT_ID_STR => Some(ModuleId::new(
                ibc::apps::transfer::types::MODULE_ID_STR.to_string(),
            )),
            _ => None,
        }
    }
}
