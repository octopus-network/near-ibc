use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::{Base58CryptoHash, U128},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    store::{LookupMap, UnorderedMap},
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseResult,
};
use utils::{
    interfaces::{ext_wrapped_token, TokenFactory},
    types::{AssetDenom, CrossChainAsset},
    ExtraDepositCost,
};

mod migration;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    TokenContractWasm,
    AssetIdMappings,
    DenomToAssetIdMap,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    /// Maps asset id to cross chain asset.
    asset_id_mappings: UnorderedMap<String, CrossChainAsset>,
    /// Maps asset denom to asset id.
    denom_to_asset_id_map: LookupMap<AssetDenom, String>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "ERR_CONTRACT_MUST_BE_DEPLOYED_IN_SUB_ACCOUNT",
        );
        Self {
            asset_id_mappings: UnorderedMap::new(StorageKey::AssetIdMappings),
            denom_to_asset_id_map: LookupMap::new(StorageKey::DenomToAssetIdMap),
        }
    }
    ///
    fn assert_asset_not_registered(&self, cross_chain_asset: &CrossChainAsset) {
        self.asset_id_mappings.iter().for_each(|(_, asset)| {
            assert!(
                asset.asset_denom != cross_chain_asset.asset_denom,
                "ERR_ASSET_ALREADY_REGISTERED"
            );
        });
    }
}

#[near_bindgen]
impl TokenFactory for Contract {
    #[payable]
    fn setup_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        metadata: FungibleTokenMetadata,
    ) {
        utils::assert_ancestor_account();
        let asset_denom = AssetDenom {
            trace_path: trace_path.clone(),
            base_denom: base_denom.clone(),
        };
        let mut cross_chain_asset = CrossChainAsset {
            asset_id: "00000000000000000000000000000000".to_string(),
            asset_denom: asset_denom.clone(),
            metadata: metadata.clone(),
        };
        self.assert_asset_not_registered(&cross_chain_asset);
        let minimum_deposit = utils::INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT
            + env::storage_byte_cost().as_yoctonear()
                * (32 + borsh::to_vec(&cross_chain_asset).unwrap().len()) as u128;
        assert!(
            env::attached_deposit().as_yoctonear() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        // Generate asset id.
        let mut asset_id =
            hex::encode(env::sha256(borsh::to_vec(&asset_denom).unwrap().as_slice()))
                .get(0..32)
                .unwrap()
                .to_string();
        let mut retry: u8 = 0;
        while self.asset_id_mappings.contains_key(&asset_id) {
            let mut bytes = borsh::to_vec(&asset_denom).unwrap();
            bytes.push(retry);
            asset_id = hex::encode(env::sha256(bytes.as_slice()))
                .get(0..32)
                .unwrap()
                .to_string();
            retry += 1;
            assert!(retry < 255, "ERR_TOO_MANY_RETRIES_IN_ASSET_ID_GENERATION");
        }
        cross_chain_asset.asset_id = asset_id.clone();
        // Create token contract.
        let token_contract_id: AccountId = format!("{}.{}", asset_id, env::current_account_id())
            .parse()
            .unwrap();
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub metadata: FungibleTokenMetadata,
            trace_path: String,
            base_denom: String,
            near_ibc_account: AccountId,
        }
        let args = Input {
            metadata: metadata.clone(),
            trace_path,
            base_denom,
            near_ibc_account: env::predecessor_account_id(),
        };
        let args =
            near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_MINT_FUNCTION");
        Promise::new(token_contract_id)
            .create_account()
            .transfer(NearToken::from_yoctonear(
                utils::INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT,
            ))
            .deploy_contract(
                env::storage_read(&borsh::to_vec(&StorageKey::TokenContractWasm).unwrap()).unwrap(),
            )
            .function_call(
                "new".to_string(),
                args,
                NearToken::from_yoctonear(0),
                utils::GAS_FOR_SIMPLE_FUNCTION_CALL,
            );
        ExtraDepositCost::add(utils::INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT);
        // Store mappings.
        self.asset_id_mappings
            .insert(asset_id.clone(), cross_chain_asset.clone());
        // Refund unused deposit.
        utils::refund_deposit(used_bytes);
    }

    #[payable]
    fn mint_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        token_owner: AccountId,
        amount: U128,
    ) {
        utils::assert_ancestor_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        let maybe_asset_id = self
            .denom_to_asset_id_map
            .get(&asset_denom)
            .map(|v| v.clone());
        assert!(maybe_asset_id.is_some(), "ERR_ASSET_NEEDS_TO_BE_SETUP");
        // Mint tokens.
        let token_contract_id: AccountId =
            format!("{}.{}", maybe_asset_id.unwrap(), env::current_account_id())
                .parse()
                .unwrap();
        ext_wrapped_token::ext(token_contract_id.clone())
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(2))
            .with_unused_gas_weight(0)
            .mint(token_owner.clone(), amount)
            .then(
                ext_mint_callback::ext(env::current_account_id())
                    .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                    .with_unused_gas_weight(0)
                    .mint_callback(
                        asset_denom.to_string(),
                        token_contract_id,
                        token_owner,
                        amount,
                    ),
            );
    }
}

#[ext_contract(ext_mint_callback)]
pub trait MintCallback {
    fn mint_callback(
        &mut self,
        denom: String,
        token_contract: AccountId,
        token_owner: AccountId,
        amount: U128,
    );
}

#[near_bindgen]
impl MintCallback for Contract {
    #[private]
    fn mint_callback(
        &mut self,
        denom: String,
        token_contract: AccountId,
        token_owner: AccountId,
        amount: U128,
    ) {
        match env::promise_result(0) {
            PromiseResult::Successful(_bytes) => {
                log!(
                    r#"EVENT_JSON:{{"standard":"nep297","version":"1.0.0","event":"MINT_SUCCEEDED","denom":"{}","token_contract":"{}","token_owner":"{}","amount":"{}"}}"#,
                    denom,
                    token_contract,
                    token_owner,
                    amount.0,
                );
            }
            PromiseResult::Failed => {
                log!(
                    r#"EVENT_JSON:{{"standard":"nep297","version":"1.0.0","event":"ERR_MINT","denom":"{}","token_contract":"{}","receiver_id":"{}","amount":"{}"}}"#,
                    denom,
                    token_contract,
                    token_owner,
                    amount.0,
                );
            }
        }
    }
}

/// View functions.
pub trait Viewer {
    /// Get all cross chain assets.
    fn get_cross_chain_assets(&self) -> Vec<CrossChainAsset>;
}

#[near_bindgen]
impl Viewer for Contract {
    fn get_cross_chain_assets(&self) -> Vec<CrossChainAsset> {
        let mut assets = Vec::new();
        for asset in self.asset_id_mappings.values() {
            assets.push(asset.clone());
        }
        assets
    }
}

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_wasm_of_token_contract() {
    env::setup_panic_hook();
    let _contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    assert_eq!(
        env::predecessor_account_id(),
        env::current_account_id(),
        "ERR_NOT_ALLOWED"
    );
    let input = env::input().expect("ERR_NO_INPUT");
    let sha256_hash = env::sha256(&input);

    let current_len = env::storage_read(&borsh::to_vec(&StorageKey::TokenContractWasm).unwrap())
        .map_or_else(|| 0, |bytes| bytes.len());
    let blob_len = input.len();
    if blob_len > current_len {
        let storage_cost = (env::storage_usage() + blob_len as u64 - current_len as u64) as u128
            * env::storage_byte_cost().as_yoctonear();
        assert!(
            env::account_balance().as_yoctonear() >= storage_cost,
            "ERR_NOT_ENOUGH_ACCOUNT_BALANCE, needs {} more.",
            storage_cost - env::account_balance().as_yoctonear()
        );
    }

    env::storage_write(
        &borsh::to_vec(&StorageKey::TokenContractWasm).unwrap(),
        &input,
    );

    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}
