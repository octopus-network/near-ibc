use crate::*;

#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct OldContract {
    asset_id_mappings: UnorderedMap<String, CrossChainAsset>,
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
        let mut new_contract = Contract {
            asset_id_mappings: old_contract.asset_id_mappings,
            asset_denom_mappings: UnorderedMap::new(StorageKey::AssetDenomMappings),
        };
        //
        new_contract
            .asset_id_mappings
            .iter()
            .for_each(|(_, asset)| {
                new_contract
                    .asset_denom_mappings
                    .insert(asset.asset_denom.clone(), asset.asset_id.clone());
            });
        //
        new_contract
    }
}
