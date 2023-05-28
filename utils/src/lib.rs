use core::str::FromStr;
use ibc::applications::transfer::PORT_ID_STR;
use near_sdk::{
    env,
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
    AccountId, Balance, Gas, Promise,
};

pub mod interfaces;
mod macros;
pub mod types;

/// Gas for a complex function call.
pub const GAS_FOR_COMPLEX_FUNCTION_CALL: Gas = Gas(150_000_000_000_000);
/// Gas for a simple function call.
pub const GAS_FOR_SIMPLE_FUNCTION_CALL: Gas = Gas(5_000_000_000_000);

/// As the `deliver` function may cause storage changes, the caller needs to attach some NEAR
/// to cover the storage cost. The minimum valid amount is 0.05 NEAR (for 5 kb storage).
pub const MINIMUM_DEPOSIT_FOR_DELEVER_MSG: Balance = 50_000_000_000_000_000_000_000;
/// Initial balance for the token contract to cover storage deposit.
pub const INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT: Balance = 3_500_000_000_000_000_000_000_000;
/// Initial balance for the channel escrow to cover storage deposit.
pub const INIT_BALANCE_FOR_CHANNEL_ESCROW_CONTRACT: Balance = 3_000_000_000_000_000_000_000_000;

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
        GAS_FOR_SIMPLE_FUNCTION_CALL,
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

/// Asserts that the predecessor account is the sub account of current account.
pub fn assert_sub_account() {
    let account_id = env::predecessor_account_id().to_string();
    let (_first, parent) = account_id.split_once(".").unwrap();
    assert!(
        parent.ends_with(env::current_account_id().as_str()),
        "ERR_ONLY_SUB_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

/// Asserts that the predecessor account is an ancestor account of current account.
pub fn assert_ancestor_account() {
    let account_id = env::current_account_id().to_string();
    let (_first, parent) = account_id.split_once(".").unwrap();
    assert!(
        parent.ends_with(env::predecessor_account_id().as_str()),
        "ERR_ONLY_UPPER_LEVEL_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

/// Get the grandparent account id from the current account id.
pub fn get_grandparent_account_id() -> AccountId {
    let account_id = String::from(env::current_account_id().as_str());
    let parts = account_id.split(".").collect::<Vec<&str>>();
    assert!(parts.len() > 3, "ERR_NO_GRANDPARENT_ACCOUNT_FOUND");
    let grandparent_account = parts[2..parts.len()].join(".");
    AccountId::from_str(grandparent_account.as_str()).unwrap()
}

/// Get the token factory contract id by directly appending a certain suffix
/// to the current account id.
pub fn get_token_factory_contract_id() -> AccountId {
    format!("tf.{}.{}", PORT_ID_STR, env::current_account_id())
        .parse()
        .unwrap()
}

/// Get the escrow factory contract id by directly appending a certain suffix
/// to the current account id.
pub fn get_escrow_factory_contract_id() -> AccountId {
    format!("ef.{}.{}", PORT_ID_STR, env::current_account_id())
        .parse()
        .unwrap()
}
