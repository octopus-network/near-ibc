use crate::context::IbcContext;

use ibc::{
    applications::transfer::{
        MODULE_ID_STR as TRANSFER_MODULE_ID, PORT_ID_STR as TRANSFER_PORT_ID,
    },
    core::{
        ics05_port::{context::PortReader, error::PortError},
        ics24_host::identifier::PortId,
        ics26_routing::context::ModuleId,
    },
};
use std::str::FromStr;

impl PortReader for IbcContext<'_> {
    /// Return the module_id associated with a given port_id
    fn lookup_module_by_port(&self, port_id: &PortId) -> Result<ModuleId, PortError> {
        match port_id.as_str() {
            TRANSFER_PORT_ID => ModuleId::from_str(TRANSFER_MODULE_ID)
                .map_err(|e| PortError::ImplementationSpecific),
            // _ => Err(ICS05Error::module_not_found(port_id.clone())),
            _ => Err(PortError::UnknownPort {
                port_id: port_id.clone(),
            }),
        }
    }
}
