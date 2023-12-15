use crate::*;

pub trait StorageMigration {
    fn migrate_state() -> Self;
}

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct OldContract {
    /// The account id of IBC/TAO implementation.
    near_ibc_account: AccountId,
    /// The token accounts that this contract is allowed to send tokens to.
    token_contracts: UnorderedMap<AccountId, AssetDenom>,
    /// Accounting for the pending transfer requests.
    pending_transfer_requests: UnorderedMap<AccountId, Ics20TransferRequest>,
}

#[near_bindgen]
impl StorageMigration for Contract {
    #[init(ignore_state)]
    fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");
        //
        utils::assert_parent_account();
        //
        // Create the new contract using the data from the old contract.
        let mut new_contract = Contract {
            near_ibc_account: old_contract.near_ibc_account,
            token_contracts: old_contract.token_contracts,
            pending_transfer_requests: old_contract.pending_transfer_requests,
            denom_to_token_contract_map: LookupMap::new(StorageKey::DenomToTokenContractMap),
        };
        //
        new_contract.token_contracts.into_iter().for_each(|(k, v)| {
            new_contract
                .denom_to_token_contract_map
                .insert(v.clone(), k.clone());
        });
        //
        new_contract
    }
}
