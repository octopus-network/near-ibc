use crate::ibc_impl::core::host::type_define::{
    IbcHostHeight, NearAcknowledgementCommitment, NearAcksPath, NearChannelEnd,
    NearChannelEndsPath, NearChannelId, NearClientId, NearClientState, NearClientStatePath,
    NearClientType, NearClientTypePath, NearCommitmentsPath, NearConnectionEnd, NearConnectionId,
    NearConsensusState, NearHeight, NearIbcHeight, NearModuleId, NearPacketCommitment, NearPortId,
    NearReceipt, NearRecipientsPath, NearSeqAcksPath, NearSeqRecvsPath, NearSeqSendsPath,
    NearSequence, NearTimeStamp,
};
use crate::ibc_impl::core::routing::{NearRouter, NearRouterBuilder};
use crate::link_map::KeySortLinkMap;
use crate::*;
use ibc::core::ics02_client::client_type::ClientType;
use ibc::core::ics02_client::height::Height;
use ibc::core::ics03_connection::connection::ConnectionEnd;
use ibc::core::ics04_channel::channel::ChannelEnd;
use ibc::core::ics04_channel::commitment::{AcknowledgementCommitment, PacketCommitment};
use ibc::core::ics04_channel::packet::{Receipt, Sequence};
use ibc::core::ics24_host::identifier::{ChainId, ChannelId, ClientId, ConnectionId, PortId};
use ibc::core::ics24_host::path::ClientTypePath;
use ibc::{
    applications::transfer::MODULE_ID_STR,
    core::ics26_routing::context::{Module, ModuleId, RouterBuilder},
};
use ibc_proto::google::protobuf::Duration;
use near_sdk::collections::{LookupMap, UnorderedMap};

// #[derive(Debug)]
// pub struct HostType;

/// A context implementing the dependencies necessary for Any Ibc Module.
// #[derive(Debug)]
pub struct IbcContext<'a> {
    /// The type of host chain underlying this Near Context.
    // pub host_chain_type: HostType,

    /// Host chain identifier.
    /// todo confirm how to name it
    // pub host_chain_id: ChainId,

    /// Maximum size for the history of the host chain. Any block older than this is pruned.
    /// todo the mock ibc use it when impl host_consensus_state
    // pub max_history_size: usize,

    // The chain of blocks underlying this context. A vector of size up to `max_history_size`
    // blocks, ascending order by their height (latest block is on the last position).
    /// todo the mock ibc use it when impl host_consensus_state
    // history: Vec<HostBlock>,

    /// Average time duration between blocks
    /// todo not sure, maybe 1.5s?
    // pub block_time: Duration,

    /// An Object that store all Ibc related data.
    pub near_ibc_store: &'a mut NearIbcStore,

    /// Ics26 Router impl
    pub router: NearRouter,
}

impl<'a> IbcContext<'a> {
    pub fn new(store: &'a mut NearIbcStore) -> Self {
        let r = NearRouterBuilder::default()
            // todo wait for ics20
            // .add_route(MODULE_ID_STR.parse().unwrap(), IbcTransferModule(PhantomData::<T>)) // register transfer Module
            // .unwrap()
            .build();

        IbcContext {
            near_ibc_store: store,
            router: Default::default(),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearIbcStore {
    // pub clients: LookupMap<NearClientId, NearClientRecord>,
    pub client_types: LookupMap<ClientId, ClientType>,
    pub client_state: UnorderedMap<ClientId, NearClientState>,
    pub consensus_states: KeySortLinkMap<Height, NearConsensusState>,
    pub client_processed_times: LookupMap<(ClientId, Height), NearTimeStamp>,
    pub client_processed_heights: LookupMap<(ClientId, Height), IbcHostHeight>,

    pub client_ids_counter: u64,

    pub client_connections: LookupMap<ClientId, ConnectionId>,

    pub connections: UnorderedMap<ConnectionId, ConnectionEnd>,

    pub connection_ids_counter: u64,

    pub connection_channels: LookupMap<ConnectionId, Vec<(PortId, ChannelId)>>,

    pub channel_ids_counter: u64,

    pub channels: UnorderedMap<(PortId, ChannelId), ChannelEnd>,

    pub next_sequence_send: LookupMap<(PortId, ChannelId), Sequence>,

    pub next_sequence_recv: LookupMap<(PortId, ChannelId), Sequence>,

    pub next_sequence_ack: LookupMap<(PortId, ChannelId), Sequence>,

    pub packet_receipt: LookupMap<(PortId, ChannelId, Sequence), Receipt>,

    pub packet_acknowledgement: LookupMap<(PortId, ChannelId, Sequence), AcknowledgementCommitment>,

    pub port_to_module: LookupMap<PortId, ModuleId>,

    pub packet_commitment: LookupMap<(PortId, ChannelId, Sequence), PacketCommitment>,
}
