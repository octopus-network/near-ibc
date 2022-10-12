use crate::NearContext;
use ibc::core::ics05_port::context::PortReader;
use ibc::core::ics24_host::identifier::PortId;
use ibc::core::ics26_routing::context::ModuleId;
use ibc::core::ics05_port::error::Error as Ics05Error;

impl PortReader for NearContext {
    /// Return the module_id associated with a given port_id
    fn lookup_module_by_port(&self, port_id: &PortId) -> Result<ModuleId, Ics05Error> {
        todo!()
    }
}