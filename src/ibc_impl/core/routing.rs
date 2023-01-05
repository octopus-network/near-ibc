use crate::context::IbcContext;
use ibc::core::ics26_routing::context::{Module, ModuleId, Router, RouterBuilder, RouterContext};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

#[derive(Default)]
pub struct NearRouterBuilder(NearRouter);

impl RouterBuilder for NearRouterBuilder {
    type Router = NearRouter;

    fn add_route(mut self, module_id: ModuleId, module: impl Module) -> Result<Self, String> {
        match self.0 .0.insert(module_id, Arc::new(module)) {
            None => Ok(self),
            Some(_) => Err("Duplicate module_id".to_owned()),
        }
    }

    fn build(self) -> Self::Router {
        self.0
    }
}

#[derive(Clone, Default)]
pub struct NearRouter(BTreeMap<ModuleId, Arc<dyn Module>>);

impl Debug for NearRouter {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0.keys().collect::<Vec<&ModuleId>>())
    }
}

impl Router for NearRouter {
    fn get_route_mut(&mut self, module_id: &impl Borrow<ModuleId>) -> Option<&mut dyn Module> {
        self.0.get_mut(module_id.borrow()).and_then(Arc::get_mut)
    }

    fn has_route(&self, module_id: &impl Borrow<ModuleId>) -> bool {
        self.0.get(module_id.borrow()).is_some()
    }
}

impl RouterContext for IbcContext<'_> {
    type Router = NearRouter;

    fn router(&self) -> &Self::Router {
        &self.router
    }

    fn router_mut(&mut self) -> &mut Self::Router {
        &mut self.router
    }
}
