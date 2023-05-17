use near_sdk::{
    env,
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
    AccountId, Balance, Gas, Promise,
};

mod macros;
pub mod types;

/// Gas for calling `check_storage_and_refund` function.
pub const GAS_FOR_CHECK_STORAGE_AND_REFUND: Gas = Gas(5_000_000_000_000);
/// Gas attached to the function call of `setup_asset` on token factory contract.
pub const GAS_FOR_SETUP_ASSET: Gas = Gas(100_000_000_000_000);
/// Gas attached to the function call of `mint_asset` on token factory contract.
pub const GAS_FOR_MINT_ASSET: Gas = Gas(30_000_000_000_000);
/// Gas attached to the function call of `burn_asset` on token factory contract.
pub const GAS_FOR_BURN_ASSET: Gas = Gas(20_000_000_000_000);
/// Gas attached to the token contract creation.
pub const GAS_FOR_TOKEN_CONTRACT_INIT: Gas = Gas(5_000_000_000_000);
/// Gas attached to the token contract mint.
pub const GAS_FOR_TOKEN_CONTRACT_MINT: Gas = Gas(5_000_000_000_000);
/// Gas attached to the token contract burn.
pub const GAS_FOR_TOKEN_CONTRACT_BURN: Gas = Gas(5_000_000_000_000);
/// Initial balance for the token contract to cover storage deposit.
pub const BALANCE_FOR_TOKEN_CONTRACT_INIT: Balance = 3_000_000_000_000_000_000_000_000;
/// The minimum balance for calling `mint` on the token contract.
pub const BALANCE_FOR_TOKEN_CONTRACT_MINT: Balance = 10_000_000_000_000_000_000_000;

pub trait CheckStorageAndRefund {
    /// Check the storage used in previously function call
    /// and refund the unused attached deposit.
    fn check_storage_and_refund(
        &mut self,
        caller: AccountId,
        attached_deposit: U128,
        previously_used_bytes: U64,
    );
}

/// Check the usage of storage of current account and refund the unused attached deposit.
///
/// For calling this function, at least `GAS_FOR_CHECK_STORAGE_AND_REFUND` gas is needed.
/// And the contract also needs to call the `impl_storage_check_and_refund!` macro.
///
/// Better to call this function at the end of a `payable` function
/// by recording the `previously_used_bytes` at the start of the `payable` function.
pub fn refund_deposit(previously_used_bytes: u64, max_refundable_amount: u128) {
    #[derive(Serialize, Deserialize, Clone)]
    #[serde(crate = "near_sdk::serde")]
    struct Input {
        pub caller: AccountId,
        pub max_refundable_amount: U128,
        pub previously_used_bytes: U64,
    }
    let args = Input {
        caller: env::predecessor_account_id(),
        max_refundable_amount: U128(max_refundable_amount),
        previously_used_bytes: U64(previously_used_bytes),
    };
    let args = near_sdk::serde_json::to_vec(&args)
        .expect("ERR_SERIALIZE_ARGS_OF_CHECK_STORAGE_AND_REFUND");
    Promise::new(env::current_account_id()).function_call(
        "check_storage_and_refund".to_string(),
        args,
        0,
        GAS_FOR_CHECK_STORAGE_AND_REFUND,
    );
}

/// Asserts that the predecessor account is the root account.
pub fn assert_root_account() {
    let account_id = String::from(env::current_account_id().as_str());
    let parts = account_id.split(".").collect::<Vec<&str>>();
    let root_account = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    assert_eq!(
        env::predecessor_account_id().to_string(),
        root_account,
        "ERR_ONLY_ROOT_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

// Asserts that the predecessor is the parent account.
pub fn assert_parent_account() {
    let account_id = String::from(env::current_account_id().as_str());
    let (_first, parent) = account_id.split_once(".").unwrap();
    assert_eq!(
        env::predecessor_account_id().as_str(),
        parent,
        "ERR_ONLY_PARENT_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

/// Asserts that the predecessor account is the root account.
pub fn assert_grandparent_account() {
    let account_id = String::from(env::current_account_id().as_str());
    let parts = account_id.split(".").collect::<Vec<&str>>();
    assert!(
        parts.len() > 3,
        "ERR_ONLY_GRANDPARENT_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
    let grandparent_account = parts[2..parts.len()].join(".");
    assert_eq!(
        env::predecessor_account_id().to_string(),
        grandparent_account,
        "ERR_ONLY_GRANDPARENT_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

pub fn get_token_factory_contract_id() -> AccountId {
    format!("tf.transfer.{}", env::current_account_id())
        .parse()
        .unwrap()
}
