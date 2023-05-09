use core::{
    cell::RefCell,
    fmt::{Debug, Formatter},
    marker::PhantomData,
};

use crate::{
    ibc_impl::{
        applications::transfer::TransferModule,
        core::{
            host::type_define::{
                IbcHostHeight, NearAcknowledgementCommitment, NearAcksPath, NearChannelEnd,
                NearChannelEndsPath, NearChannelId, NearClientId, NearClientStatePath,
                NearClientType, NearClientTypePath, NearCommitmentsPath, NearConnectionEnd,
                NearConnectionId, NearHeight, NearIbcHeight, NearModuleId, NearPacketCommitment,
                NearPortId, NearReceipt, NearRecipientsPath, NearSeqAcksPath, NearSeqRecvsPath,
                NearSeqSendsPath, NearSequence, NearTimeStamp, RawClientState, RawConsensusState,
            },
            routing::{NearRouter, NearRouterBuilder},
        },
    },
    indexed_lookup_queue::IndexedLookupQueue,
    StorageKey,
};
use ibc::{
    applications::transfer,
    core::{
        ics02_client::{
            client_state::ClientState,
            client_type::ClientType,
            consensus_state::ConsensusState,
            context::{ClientKeeper, ClientReader},
        },
        ics03_connection::{
            connection::ConnectionEnd,
            context::{ConnectionKeeper, ConnectionReader},
            error::ConnectionError,
        },
        ics04_channel::{
            channel::ChannelEnd,
            commitment::{AcknowledgementCommitment, PacketCommitment},
            context::{ChannelKeeper, ChannelReader},
            error::{ChannelError, PacketError},
            packet::{Receipt, Sequence},
        },
        ics05_port::context::PortReader,
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::{
                AcksPath, ChannelEndsPath, CommitmentsPath, ConnectionsPath, ReceiptsPath,
                SeqAcksPath, SeqRecvsPath, SeqSendsPath,
            },
            Path,
        },
        ics26_routing::context::{Module, ModuleId, Router, RouterBuilder, RouterContext},
    },
    timestamp::Timestamp,
    Height,
};
use ibc_proto::google::protobuf::Duration;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    store::{LookupMap, UnorderedMap, Vector},
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct NearIbcStore {
    pub client_types: LookupMap<ClientId, ClientType>,
    pub client_states: UnorderedMap<ClientId, RawClientState>,
    pub consensus_states: LookupMap<ClientId, IndexedLookupQueue<Height, RawConsensusState>>,
    pub client_processed_times: LookupMap<ClientId, IndexedLookupQueue<Height, NearTimeStamp>>,
    pub client_processed_heights: LookupMap<ClientId, IndexedLookupQueue<Height, IbcHostHeight>>,
    pub client_ids_counter: u64,
    pub client_connections: LookupMap<ClientId, Vector<ConnectionId>>,
    pub connections: UnorderedMap<ConnectionId, ConnectionEnd>,
    pub connection_ids_counter: u64,
    pub port_to_module: LookupMap<PortId, ModuleId>,
    pub connection_channels: LookupMap<ConnectionId, Vector<(PortId, ChannelId)>>,
    pub channel_ids_counter: u64,
    pub channels: UnorderedMap<(PortId, ChannelId), ChannelEnd>,
    pub next_sequence_send: LookupMap<(PortId, ChannelId), Sequence>,
    pub next_sequence_recv: LookupMap<(PortId, ChannelId), Sequence>,
    pub next_sequence_ack: LookupMap<(PortId, ChannelId), Sequence>,
    pub packet_receipts: LookupMap<(PortId, ChannelId), IndexedLookupQueue<Sequence, Receipt>>,
    pub packet_acknowledgements:
        LookupMap<(PortId, ChannelId), IndexedLookupQueue<Sequence, AcknowledgementCommitment>>,
    pub packet_commitments:
        LookupMap<(PortId, ChannelId), IndexedLookupQueue<Sequence, PacketCommitment>>,
}

pub trait NearIbcStoreHost {
    ///
    fn get_near_ibc_store() -> NearIbcStore {
        let store =
            near_sdk::env::storage_read(&StorageKey::NearIbcStore.try_to_vec().unwrap()).unwrap();
        let store = NearIbcStore::try_from_slice(&store).unwrap();
        store
    }
    ///
    fn set_near_ibc_store(store: &NearIbcStore) {
        let store = store.try_to_vec().unwrap();
        near_sdk::env::storage_write(b"ibc_store", &store);
    }
}

pub struct NearRouterContext {
    pub near_ibc_store: NearIbcStore,
    pub router: NearRouter,
}

impl NearRouterContext {
    pub fn new(store: NearIbcStore) -> Self {
        let router = NearRouterBuilder::default()
            .add_route(transfer::MODULE_ID_STR.parse().unwrap(), TransferModule()) // register transfer Module
            .unwrap()
            .build();

        Self {
            near_ibc_store: store,
            router,
        }
    }
}

impl Debug for NearIbcStore {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "NearIbcStore {{ ... }}")
    }
}
