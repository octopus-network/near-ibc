use crate::{context::NearIbcStore, prelude::*};
use ibc::core::ics24_host::identifier::PortId;
use ibc::core::router::{Module, ModuleId, Router};

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
    /// Return the module_id associated with a given port_id
    fn lookup_module(&self, port_id: &PortId) -> Option<ModuleId> {
        self.module_holder.get_module_id(port_id)
    }
}
