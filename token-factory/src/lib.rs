use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::{Base58CryptoHash, U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    store::UnorderedMap,
    AccountId, BorshStorageKey, PanicOnDefault, Promise,
};
use utils::types::AssetDenom;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AssetIdMappings,
    DenomMappings,
    TokenContractWasm,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct TokenFactory {
    asset_id_mappings: UnorderedMap<String, AssetDenom>,
    denom_mappings: UnorderedMap<AssetDenom, String>,
}

#[near_bindgen]
impl TokenFactory {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "ERR_ALREADY_INITIALIZED");
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "ERR_CONTRACT_MUST_BE_DEPLOYED_IN_SUB_ACCOUNT",
        );
        Self {
            asset_id_mappings: UnorderedMap::new(StorageKey::AssetIdMappings),
            denom_mappings: UnorderedMap::new(StorageKey::DenomMappings),
        }
    }
    #[payable]
    /// Create a new token contract.
    pub fn setup_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        metadata: FungibleTokenMetadata,
    ) {
        utils::assert_grandparent_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        let minimum_deposit = utils::BALANCE_FOR_TOKEN_CONTRACT_INIT
            + env::storage_byte_cost() * (asset_denom.try_to_vec().unwrap().len() + 32) as u128 * 2;
        assert!(
            env::attached_deposit() >= minimum_deposit,
            "ERR_NOT_ENOUGH_DEPOSIT, must not less than {} yocto",
            minimum_deposit
        );
        let used_bytes = env::storage_usage();
        if !self.denom_mappings.contains_key(&asset_denom) {
            // Generate asset id.
            let mut asset_id =
                hex::encode(env::sha256(asset_denom.try_to_vec().unwrap().as_slice()))
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
            // Create token contract.
            let token_contract_id: AccountId =
                format!("{}.{}", asset_id, env::current_account_id())
                    .parse()
                    .unwrap();
            #[derive(Serialize, Deserialize, Clone)]
            #[serde(crate = "near_sdk::serde")]
            struct Input {
                pub metadata: FungibleTokenMetadata,
            }
            let args = Input { metadata };
            let args =
                near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_MINT_FUNCTION");
            Promise::new(token_contract_id)
                .create_account()
                .transfer(utils::BALANCE_FOR_TOKEN_CONTRACT_INIT)
                .deploy_contract(
                    env::storage_read(&StorageKey::TokenContractWasm.try_to_vec().unwrap())
                        .unwrap(),
                )
                .function_call(
                    "new".to_string(),
                    args,
                    0,
                    utils::GAS_FOR_TOKEN_CONTRACT_INIT,
                );
            // Store mappings.
            self.asset_id_mappings
                .insert(asset_id.clone(), asset_denom.clone());
            self.denom_mappings
                .insert(asset_denom.clone(), asset_id.clone());
        }
        // Refund unused deposit.
        utils::refund_deposit(
            used_bytes,
            env::attached_deposit() - utils::BALANCE_FOR_TOKEN_CONTRACT_INIT,
        );
    }

    /// Create a new token contract and mint the given amount of tokens to the given owner.
    #[payable]
    pub fn mint_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        token_owner: AccountId,
        amount: U128,
    ) {
        utils::assert_grandparent_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        assert!(
            env::attached_deposit()
                > utils::BALANCE_FOR_TOKEN_CONTRACT_MINT
                    + env::storage_byte_cost()
                        * (asset_denom.try_to_vec().unwrap().len() + 32) as u128
                        * 2,
            "ERR_NOT_ENOUGH_DEPOSIT"
        );
        assert!(
            self.denom_mappings.contains_key(&asset_denom),
            "ERR_ASSET_NEEDS_TO_BE_SETUP"
        );
        let used_bytes = env::storage_usage();
        // Mint tokens.
        let asset_id = self.denom_mappings.get(&asset_denom).unwrap();
        let token_contract_id: AccountId = format!("{}.{}", asset_id, env::current_account_id())
            .parse()
            .unwrap();
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub account_id: AccountId,
            pub amount: U128,
        }
        let args = Input {
            account_id: token_owner,
            amount,
        };
        let args =
            near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_MINT_FUNCTION");
        Promise::new(token_contract_id).function_call(
            "mint".to_string(),
            args,
            utils::BALANCE_FOR_TOKEN_CONTRACT_MINT,
            utils::GAS_FOR_TOKEN_CONTRACT_MINT,
        );
        // Refund unused deposit.
        utils::refund_deposit(
            used_bytes,
            env::attached_deposit() - utils::BALANCE_FOR_TOKEN_CONTRACT_MINT,
        );
    }
    ///
    pub fn burn_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        token_owner: AccountId,
        amount: U128,
    ) {
        utils::assert_grandparent_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        assert!(
            self.denom_mappings.contains_key(&asset_denom),
            "ERR_ASSET_NOT_FOUND"
        );
        // Burn tokens.
        let asset_id = self.denom_mappings.get(&asset_denom).unwrap();
        let token_contract_id: AccountId = format!("{}.{}", asset_id, env::current_account_id())
            .parse()
            .unwrap();
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub account_id: AccountId,
            pub amount: U128,
        }
        let args = Input {
            account_id: token_owner,
            amount,
        };
        let args =
            near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_BURN_FUNCTION");
        Promise::new(token_contract_id).function_call(
            "burn".to_string(),
            args,
            0,
            utils::GAS_FOR_TOKEN_CONTRACT_BURN,
        );
    }
}

utils::impl_storage_check_and_refund!(TokenFactory);

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_wasm_of_token_contract() {
    env::setup_panic_hook();
    let _contract: TokenFactory = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    assert_eq!(
        env::predecessor_account_id(),
        env::current_account_id(),
        "ERR_NOT_ALLOWED"
    );
    let input = env::input().expect("ERR_NO_INPUT");
    let sha256_hash = env::sha256(&input);

    let blob_len = input.len();
    let storage_cost = ((blob_len + 32) as u128) * env::storage_byte_cost();
    assert!(
        env::attached_deposit() >= storage_cost,
        "ERR_NOT_ENOUGH_DEPOSIT:{}",
        storage_cost
    );

    env::storage_write(&StorageKey::TokenContractWasm.try_to_vec().unwrap(), &input);
    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}
