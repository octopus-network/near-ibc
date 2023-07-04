use crate::{context::NearIbcStore, prelude::*};
use ibc::core::{
    ics24_host::identifier::PortId,
    router::{Module, ModuleId, Router},
};

impl Router for NearIbcStore {
    //
    fn get_route(&self, module_id: &ModuleId) -> Option<&dyn Module> {
        match module_id.to_string().as_str() {
            ibc::applications::transfer::MODULE_ID_STR => Some(&self.module_holder.transfer_module),
            _ => None,
        }
    }
    //
    fn get_route_mut(&mut self, module_id: &ModuleId) -> Option<&mut dyn Module> {
        match module_id.to_string().as_str() {
            ibc::applications::transfer::MODULE_ID_STR => {
                Some(&mut self.module_holder.transfer_module)
            }
            _ => None,
        }
    }
    //
    fn has_route(&self, module_id: &ModuleId) -> bool {
        match module_id.to_string().as_str() {
            ibc::applications::transfer::MODULE_ID_STR => true,
            _ => false,
        }
    }
    //
    fn lookup_module_by_port(&self, port_id: &PortId) -> Option<ModuleId> {
        self.port_to_module
            .get(port_id)
            .map(|module_id| module_id.clone())
    }
}
