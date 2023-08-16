use crate::{
    collections::{IndexedAscendingLookupQueue, IndexedAscendingSimpleQueue},
    context::{HostHeight, NearTimeStamp},
    module_holder::ModuleHolder,
    *,
};
use ibc::core::{events::IbcEvent, ics04_channel::packet::Sequence, router::ModuleId};
use near_sdk::store::UnorderedSet;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OldNearIbcStore {
    /// To support the mutable borrow in `Router::get_route_mut`.
    pub module_holder: ModuleHolder,
    pub port_to_module: LookupMap<PortId, ModuleId>,
    /// The client ids of the clients.
    pub client_id_set: UnorderedSet<ClientId>,
    pub client_counter: u64,
    pub client_processed_times:
        LookupMap<ClientId, IndexedAscendingLookupQueue<Height, NearTimeStamp>>,
    pub client_processed_heights:
        LookupMap<ClientId, IndexedAscendingLookupQueue<Height, HostHeight>>,
    /// This collection contains the heights corresponding to all consensus states of
    /// all clients stored in the contract.
    pub client_consensus_state_height_sets:
        LookupMap<ClientId, IndexedAscendingSimpleQueue<Height>>,
    /// The connection ids of the connections.
    pub connection_id_set: UnorderedSet<ConnectionId>,
    pub connection_counter: u64,
    /// The port and channel id tuples of the channels.
    pub port_channel_id_set: UnorderedSet<(PortId, ChannelId)>,
    pub channel_counter: u64,
    /// The sequence numbers of the packet commitments.
    pub packet_commitment_sequence_sets: LookupMap<(PortId, ChannelId), UnorderedSet<Sequence>>,
    /// The sequence numbers of the packet receipts.
    pub packet_receipt_sequence_sets: LookupMap<(PortId, ChannelId), UnorderedSet<Sequence>>,
    /// The sequence numbers of the packet acknowledgements.
    pub packet_acknowledgement_sequence_sets:
        LookupMap<(PortId, ChannelId), UnorderedSet<Sequence>>,
    /// The history of IBC events.
    pub ibc_events_history: IndexedAscendingLookupQueue<Height, Vec<IbcEvent>>,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContract {
    near_ibc_store: LazyOption<OldNearIbcStore>,
    governance_account: AccountId,
}

#[near_bindgen]
impl Contract {
    #[init(ignore_state)]
    pub fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        // Create the new contract using the data from the old contract.
        let new_contract = Contract {
            near_ibc_store: LazyOption::new(
                StorageKey::NearIbcStore,
                Some(&NearIbcStore::from_old_version(
                    old_contract.near_ibc_store.get().unwrap(),
                )),
            ),
            governance_account: old_contract.governance_account,
        };
        //
        //
        new_contract
    }
}

pub fn get_storage_key_of_lookup_map<T: BorshSerialize>(prefix: &StorageKey, index: &T) -> Vec<u8> {
    [prefix.try_to_vec().unwrap(), index.try_to_vec().unwrap()].concat()
}

impl NearIbcStore {
    pub fn from_old_version(old_version: OldNearIbcStore) -> Self {
        Self {
            module_holder: old_version.module_holder,
            client_id_set: old_version.client_id_set,
            client_counter: old_version.client_counter,
            client_processed_times: old_version.client_processed_times,
            client_processed_heights: old_version.client_processed_heights,
            client_consensus_state_height_sets: old_version.client_consensus_state_height_sets,
            connection_id_set: old_version.connection_id_set,
            connection_counter: old_version.connection_counter,
            port_channel_id_set: old_version.port_channel_id_set,
            channel_counter: old_version.channel_counter,
            packet_commitment_sequence_sets: old_version.packet_commitment_sequence_sets,
            packet_receipt_sequence_sets: old_version.packet_receipt_sequence_sets,
            packet_acknowledgement_sequence_sets: old_version.packet_acknowledgement_sequence_sets,
            ibc_events_history: old_version.ibc_events_history,
        }
    }
}
