use crate::*;

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
            near_ibc_store: old_contract.near_ibc_store,
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
