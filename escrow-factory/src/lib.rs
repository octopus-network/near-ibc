use ibc::core::ics24_host::identifier::ChannelId;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::Base58CryptoHash,
    near_bindgen,
    store::UnorderedSet,
    AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, Promise,
};

/// Initial balance for the token contract to cover storage deposit.
const ESCROW_CONTRACT_INIT_BALANCE: Balance = 5_000_000_000_000_000_000_000_000;
/// Gas attached to the token contract creation.
const GAS_FOR_ESCROW_CONTRACT_INIT: Gas = Gas(5_000_000_000_000);

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    ChannelIdSet,
    EscrowContractWasm,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct EscrowFactory {
    channel_id_set: UnorderedSet<ChannelId>,
}

#[near_bindgen]
impl EscrowFactory {
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
    /// Creates escrow contract for the given channel.
    pub fn create_escrow(&mut self, channel_id: ChannelId) {
        assert_root_account();
        if !self.channel_id_set.contains(&channel_id) {
            let escrow_contract_id: AccountId =
                format!("{}.{}", channel_id, env::current_account_id())
                    .parse()
                    .unwrap();
            Promise::new(escrow_contract_id)
                .create_account()
                .transfer(ESCROW_CONTRACT_INIT_BALANCE)
                .deploy_contract(
                    env::storage_read(&StorageKey::EscrowContractWasm.try_to_vec().unwrap())
                        .unwrap(),
                )
                .function_call(
                    "new".to_string(),
                    Vec::new(),
                    0,
                    GAS_FOR_ESCROW_CONTRACT_INIT,
                );
            self.channel_id_set.insert(channel_id);
        }
    }
}

//
fn assert_root_account() {
    let account_id = String::from(env::current_account_id().as_str());
    let parts = account_id.split(".").collect::<Vec<&str>>();
    let root_account = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    assert_eq!(
        env::predecessor_account_id().to_string(),
        root_account,
        "ERR_ONLY_ROOT_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

/// Stores attached data into blob store and returns hash of it.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn store_wasm_of_escrow_contract() {
    env::setup_panic_hook();
    let _contract: EscrowFactory = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
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

    env::storage_write(
        &StorageKey::EscrowContractWasm.try_to_vec().unwrap(),
        &input,
    );
    let mut blob_hash = [0u8; 32];
    blob_hash.copy_from_slice(&sha256_hash);
    let blob_hash_str = near_sdk::serde_json::to_string(&Base58CryptoHash::from(blob_hash))
        .unwrap()
        .into_bytes();

    env::value_return(&blob_hash_str);
}
