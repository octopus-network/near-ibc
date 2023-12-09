use crate::{
    collections::IndexedAscendingLookupQueue,
    context::{HostHeight, NearTimeStamp},
    module_holder::ModuleHolder,
    *,
};
use ibc::core::{handler::types::events::IbcEvent, host::types::identifiers::Sequence};
use near_sdk::{
    borsh,
    store::{UnorderedMap, UnorderedSet},
};

pub trait StorageMigration {
    fn migrate_state() -> Self;
}

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct OldNearIbcStore {
    /// The client ids of the clients.
    pub client_id_set: UnorderedSet<ClientId>,
    pub client_counter: u64,
    pub client_processed_times: LookupMap<ClientId, UnorderedMap<Height, NearTimeStamp>>,
    pub client_processed_heights: LookupMap<ClientId, UnorderedMap<Height, HostHeight>>,
    /// This collection contains the heights corresponding to all consensus states of
    /// all clients stored in the contract.
    pub client_consensus_state_height_sets: LookupMap<ClientId, UnorderedSet<Height>>,
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
#[borsh(crate = "near_sdk::borsh")]
pub struct OldContract {
    near_ibc_store: LazyOption<OldNearIbcStore>,
    /// To support the mutable borrow in `Router::get_route_mut`.
    module_holder: ModuleHolder,
    governance_account: AccountId,
}

#[near_bindgen]
impl StorageMigration for NearIbcContract {
    #[init(ignore_state)]
    fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        // Create the new contract using the data from the old contract.
        let new_contract = NearIbcContract {
            near_ibc_store: LazyOption::new(
                StorageKey::NearIbcStore,
                Some(&NearIbcStore::from_old_version(
                    old_contract.near_ibc_store.get().unwrap(),
                )),
            ),
            module_holder: old_contract.module_holder,
            governance_account: old_contract.governance_account,
        };
        //
        env::storage_write("version".as_bytes(), VERSION.as_bytes());
        //
        new_contract
    }
}

pub fn get_storage_key_of_lookup_map<T: BorshSerialize>(prefix: &StorageKey, index: &T) -> Vec<u8> {
    [
        borsh::to_vec(&prefix).unwrap(),
        borsh::to_vec(&index).unwrap(),
    ]
    .concat()
}

impl NearIbcStore {
    pub fn from_old_version(old_version: OldNearIbcStore) -> Self {
        Self {
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
