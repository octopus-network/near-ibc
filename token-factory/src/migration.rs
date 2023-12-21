use crate::*;

pub trait StorageMigration {
    fn migrate_state() -> Self;
}

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct OldContract {
    asset_id_mappings: UnorderedMap<String, CrossChainAsset>,
}

#[near_bindgen]
impl StorageMigration for Contract {
    #[init(ignore_state)]
    fn migrate_state() -> Self {
        // Deserialize the state using the old contract structure.
        let old_contract: OldContract = env::state_read().expect("Old state doesn't exist");
        //
        near_sdk::assert_self();
        //
        // Create the new contract using the data from the old contract.
        let mut new_contract = Contract {
            asset_id_mappings: old_contract.asset_id_mappings,
            denom_to_asset_id_map: LookupMap::new(StorageKey::DenomToAssetIdMap),
        };
        //
        new_contract
            .asset_id_mappings
            .into_iter()
            .for_each(|(k, v)| {
                new_contract
                    .denom_to_asset_id_map
                    .insert(v.asset_denom.clone(), k.clone());
            });
        //
        new_contract
    }
}
