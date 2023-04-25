use ibc::core::ics24_host::identifier::{ChannelId, PortId};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::{Base58CryptoHash, U128},
    near_bindgen,
    serde::{Deserialize, Serialize},
    store::UnorderedMap,
    AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise,
};

/// Initial balance for the token contract to cover storage deposit.
const TOKEN_CONTRACT_INIT_BALANCE: Balance = 5_000_000_000_000_000_000_000_000;
/// Gas attached to the token contract creation.
const GAS_FOR_TOKEN_CONTRACT_INIT: Gas = Gas(5_000_000_000_000);
/// Gas attached to the token contract mint.
const GAS_FOR_TOKEN_CONTRACT_MINT: Gas = Gas(5_000_000_000_000);
/// Gas attached to the token contract burn.
const GAS_FOR_TOKEN_CONTRACT_BURN: Gas = Gas(5_000_000_000_000);

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AssetIdMappings,
    DenomMappings,
    TokenContractWasm,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct AssetDenom {
    pub trace_path: Vec<(PortId, ChannelId)>,
    pub base_denom: String,
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
    //
    fn assert_root_account(&self) {
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        let root_account = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
        assert_eq!(
            env::predecessor_account_id().to_string(),
            root_account,
            "ERR_ONLY_ROOT_ACCOUNT_CAN_CALL_THIS_METHOD"
        );
    }
    ///
    pub fn mint_asset(
        &mut self,
        trace_path: Vec<(PortId, ChannelId)>,
        base_denom: String,
        token_owner: AccountId,
        amount: U128,
    ) {
        self.assert_root_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
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
            self.asset_id_mappings
                .insert(asset_id.clone(), asset_denom.clone());
            self.denom_mappings
                .insert(asset_denom.clone(), asset_id.clone());
            // Create token contract.
            let token_contract_id: AccountId =
                format!("{}.{}", asset_id, env::current_account_id())
                    .parse()
                    .unwrap();
            Promise::new(token_contract_id)
                .create_account()
                .transfer(TOKEN_CONTRACT_INIT_BALANCE)
                .deploy_contract(
                    env::storage_read(&StorageKey::TokenContractWasm.try_to_vec().unwrap())
                        .unwrap(),
                )
                .function_call(
                    "new".to_string(),
                    Vec::new(),
                    0,
                    GAS_FOR_TOKEN_CONTRACT_INIT,
                );
        }
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
            0,
            GAS_FOR_TOKEN_CONTRACT_MINT,
        );
    }
    ///
    pub fn burn_asset(
        &mut self,
        trace_path: Vec<(PortId, ChannelId)>,
        base_denom: String,
        token_owner: AccountId,
        amount: U128,
    ) {
        self.assert_root_account();
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
            GAS_FOR_TOKEN_CONTRACT_BURN,
        );
    }
}

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
