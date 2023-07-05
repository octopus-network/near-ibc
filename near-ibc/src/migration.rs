use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
struct IndexedLookupQueue<K, V>
where
    K: BorshDeserialize + BorshSerialize + Clone + Ord,
    V: BorshDeserialize + BorshSerialize + Clone,
{
    /// The map of index to K.
    index_map: LookupMap<u64, K>,
    /// The map of K to V.
    value_map: LookupMap<K, V>,
    /// The start index of valid anchor event.
    start_index: u64,
    /// The end index of valid anchor event.
    end_index: u64,
    /// The max length of queue.
    max_length: u64,
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct OldContract {
    near_ibc_store: LazyOption<NearIbcStore>,
    ibc_events_history: IndexedLookupQueue<u64, Vec<u8>>,
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
            near_ibc_store: LazyOption::new(StorageKey::NearIbcStore, Some(&NearIbcStore::new())),
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
