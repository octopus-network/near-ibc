#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};
use ibc::core::host::types::identifiers::ChannelId;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::Base58CryptoHash,
    near_bindgen,
    serde::{Deserialize, Serialize},
    store::UnorderedSet,
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise,
};
use utils::{interfaces::EscrowFactory, ExtraDepositCost};

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    ChannelIdSet,
    EscrowContractWasm,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    channel_id_set: UnorderedSet<ChannelId>,
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
            channel_id_set: UnorderedSet::new(StorageKey::ChannelIdSet),
        }
    }
}

#[near_bindgen]
impl EscrowFactory for Contract {
    #[payable]
    fn create_escrow(&mut self, channel_id: ChannelId) {
        utils::assert_ancestor_account();
        let used_bytes = env::storage_usage();
        ExtraDepositCost::reset();
        if !self.channel_id_set.contains(&channel_id) {
            let escrow_contract_id: AccountId =
                format!("{}.{}", channel_id, env::current_account_id())
                    .parse()
                    .unwrap();
            #[derive(Serialize, Deserialize, Clone)]
            #[serde(crate = "near_sdk::serde")]
            struct Input {
                near_ibc_account: AccountId,
            }
            let args = Input {
                near_ibc_account: env::predecessor_account_id(),
            };
            let args = near_sdk::serde_json::to_vec(&args)
                .expect("ERR_SERIALIZE_ARGS_FOR_ESCROW_CONTRACT_INIT");
            Promise::new(escrow_contract_id)
                .create_account()
                .transfer(NearToken::from_yoctonear(
                    utils::INIT_BALANCE_FOR_CHANNEL_ESCROW_CONTRACT,
                ))
                .deploy_contract(
                    env::storage_read(&borsh::to_vec(&StorageKey::EscrowContractWasm).unwrap())
                        .unwrap(),
                )
                .function_call(
                    "new".to_string(),
                    args,
                    NearToken::from_yoctonear(0),
                    utils::GAS_FOR_SIMPLE_FUNCTION_CALL,
                );
            ExtraDepositCost::add(utils::INIT_BALANCE_FOR_CHANNEL_ESCROW_CONTRACT);
            self.channel_id_set.insert(channel_id);
        }
        utils::refund_deposit(used_bytes);
    }
}

/// View functions.
pub trait Viewer {
    /// Get all channel ids on which the escrow contract is deployed.
    fn get_channel_id_set(&self) -> Vec<ChannelId>;
}

#[near_bindgen]
impl Viewer for Contract {
    fn get_channel_id_set(&self) -> Vec<ChannelId> {
        self.channel_id_set.iter().map(|id| id.clone()).collect()
    }
}

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_wasm_of_channel_escrow() {
    env::setup_panic_hook();
    let _contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    assert_eq!(
        env::predecessor_account_id(),
        env::current_account_id(),
        "ERR_NOT_ALLOWED"
    );
    let input = env::input().expect("ERR_NO_INPUT");
    let sha256_hash = env::sha256(&input);

    let current_len = env::storage_read(&borsh::to_vec(&StorageKey::EscrowContractWasm).unwrap())
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
        &borsh::to_vec(&StorageKey::EscrowContractWasm).unwrap(),
        &input,
    );

    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}
