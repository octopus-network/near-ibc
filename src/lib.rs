#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

pub mod module;
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use near_sdk::collections::UnorderedMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use module::core::ics26_routing::NearRouter;

#[derive(Debug)]
pub struct HostType;

/// A context implementing the dependencies necessary for Any Ibc Module.
#[derive(Debug)]
pub struct NearContext {
    /// The type of host chain underlying this Near Context.
    host_chain_type: HostType,

    /// Host chain identifier.
    host_chain_id: ChainId,

    /// Maximum size for the history of the host chain. Any block odler thatn this is pruned.
    max_history_size: usize,

    // The chain of blocks underlying this context. A vector of size up to `max_history_size`
    // blocks, ascending order by their height (latest block is on the last position).
    // history: Vec<HostBlock>,
    /// Average time duration between blocks
    block_time: Duration,

    /// An Object that store all Ibc related data.
    pub ibc_store: Arc<Mutex<NearIbcStore>>,

    /// Ics26 Router impl
    router: NearRouter,
}

#[derive(Clone, Debug, Default)]
pub struct NearIbcStore;
