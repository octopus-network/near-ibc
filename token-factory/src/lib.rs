use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::{Base58CryptoHash, U128},
    near_bindgen,
    serde::{Deserialize, Serialize},
    store::UnorderedMap,
    AccountId, BorshStorageKey, PanicOnDefault, Promise,
};
use utils::{
    interfaces::{ext_wrapped_token, TokenFactory},
    types::{AssetDenom, CrossChainAsset},
    ExtraDepositCost,
};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    TokenContractWasm,
    AssetIdMappings,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    asset_id_mappings: UnorderedMap<String, CrossChainAsset>,
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
            + env::storage_byte_cost()
                * (32 + cross_chain_asset.try_to_vec().unwrap().len()) as u128;
        assert!(
            env::attached_deposit() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        // Generate asset id.
        let mut asset_id = hex::encode(env::sha256(asset_denom.try_to_vec().unwrap().as_slice()))
            .get(0..32)
            .unwrap()
            .to_string();
        let mut retry: u8 = 0;
        while self.asset_id_mappings.contains_key(&asset_id) {
            let mut bytes = asset_denom.try_to_vec().unwrap();
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
            .transfer(utils::INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT)
            .deploy_contract(
                env::storage_read(&StorageKey::TokenContractWasm.try_to_vec().unwrap()).unwrap(),
            )
            .function_call(
                "new".to_string(),
                args,
                0,
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
        let maybe_asset = self
            .asset_id_mappings
            .iter()
            .find(|asset| asset.1.asset_denom == asset_denom);
        assert!(maybe_asset.is_some(), "ERR_ASSET_NEEDS_TO_BE_SETUP");
        // Mint tokens.
        let token_contract_id: AccountId =
            format!("{}.{}", maybe_asset.unwrap().0, env::current_account_id())
                .parse()
                .unwrap();
        ext_wrapped_token::ext(token_contract_id)
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 3)
            .with_unused_gas_weight(0)
            .mint(token_owner, amount);
    }
}

/// View functions.
#[near_bindgen]
impl Contract {
    pub fn get_cross_chain_assets(&self) -> Vec<CrossChainAsset> {
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

    let current_len = env::storage_read(&StorageKey::TokenContractWasm.try_to_vec().unwrap())
        .map_or_else(|| 0, |bytes| bytes.len());
    let blob_len = input.len();
    if blob_len > current_len {
        let storage_cost = (env::storage_usage() + blob_len as u64 - current_len as u64) as u128
            * env::storage_byte_cost();
        assert!(
            env::account_balance() >= storage_cost,
            "ERR_NOT_ENOUGH_ACCOUNT_BALANCE, needs {} more.",
            storage_cost - env::account_balance()
        );
    }

    env::storage_write(&StorageKey::TokenContractWasm.try_to_vec().unwrap(), &input);

    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}
