#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    rust_2018_idioms
)]

extern crate alloc;

use core::str::FromStr;
use ibc::apps::transfer::types::PORT_ID_STR;
use near_contract_standards::fungible_token::Balance;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env, AccountId, Gas, NearToken, Promise,
};
use prelude::*;

pub mod interfaces;
mod prelude;
pub mod types;

/// Gas for a complex function call.
pub const GAS_FOR_COMPLEX_FUNCTION_CALL: Gas = Gas::from_tgas(150);
/// Gas for a simple function call.
pub const GAS_FOR_SIMPLE_FUNCTION_CALL: Gas = Gas::from_tgas(5);

/// As the `deliver` function may cause storage changes, the caller needs to attach some NEAR
/// to cover the storage cost. The minimum valid amount is 0.05 NEAR (for 5 kb storage).
pub const MINIMUM_DEPOSIT_FOR_DELEVER_MSG: Balance = 50_000_000_000_000_000_000_000;
/// The storage deposit for registering an account in the token contract. (0.0125 NEAR)
pub const STORAGE_DEPOSIT_FOR_MINT_TOKEN: Balance = 12_500_000_000_000_000_000_000;
/// Initial balance for the token contract to cover storage deposit.
pub const INIT_BALANCE_FOR_WRAPPED_TOKEN_CONTRACT: Balance = 3_500_000_000_000_000_000_000_000;
/// Initial balance for the channel escrow to cover storage deposit.
pub const INIT_BALANCE_FOR_CHANNEL_ESCROW_CONTRACT: Balance = 3_000_000_000_000_000_000_000_000;

const STORAGE_KEY_FOR_EXTRA_DEPOSIT_COST: &[u8] = b"extra_deposit_cost";

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct ExtraDepositCost(u128);

impl ExtraDepositCost {
    /// Reset the extra deposit cost to 0.
    pub fn reset() {
        env::storage_write(
            STORAGE_KEY_FOR_EXTRA_DEPOSIT_COST,
            &borsh::to_vec(&Self(0)).unwrap(),
        );
    }
    /// Add the extra deposit cost.
    pub fn add(cost: u128) {
        let mut extra_deposit_cost = Self::get();
        extra_deposit_cost.0 += cost;
        env::storage_write(
            STORAGE_KEY_FOR_EXTRA_DEPOSIT_COST,
            &borsh::to_vec(&extra_deposit_cost).unwrap(),
        );
    }
    /// Get the extra deposit cost.
    pub fn get() -> Self {
        match env::storage_read(STORAGE_KEY_FOR_EXTRA_DEPOSIT_COST) {
            Some(bytes) => Self::try_from_slice(&bytes).unwrap(),
            None => Self(0),
        }
    }
}

/// Check the usage of storage of current account and refund the unused attached deposit.
///
/// For calling this function, at least `GAS_FOR_CHECK_STORAGE_AND_REFUND` gas is needed.
/// And the contract also needs to call the `impl_storage_check_and_refund!` macro.
///
/// Better to call this function at the end of a `payable` function
/// by recording the `previously_used_bytes` at the start of the `payable` function.
pub fn refund_deposit(previously_used_bytes: u64) {
    let mut refund_amount = env::attached_deposit().as_yoctonear();
    let extra_deposit_cost = ExtraDepositCost::get().0;
    if env::storage_usage() > previously_used_bytes || extra_deposit_cost > 0 {
        let storage_increment = match env::storage_usage() > previously_used_bytes {
            true => env::storage_usage() - previously_used_bytes,
            false => 0,
        };
        near_sdk::log!(
            "Storage increment: {}, extra deposit cost: {}",
            storage_increment,
            extra_deposit_cost
        );
        let cost = env::storage_byte_cost().as_yoctonear() * storage_increment as u128
            + extra_deposit_cost;
        if cost >= refund_amount {
            return;
        } else {
            refund_amount -= cost;
        }
    }
    Promise::new(env::predecessor_account_id()).transfer(NearToken::from_yoctonear(refund_amount));
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
    let prefixed_current_account = format!(".{}", env::current_account_id());
    assert!(
        account_id.ends_with(prefixed_current_account.as_str()),
        "ERR_ONLY_SUB_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

/// Asserts that the predecessor account is an ancestor account of current account.
pub fn assert_ancestor_account() {
    let account_id = env::current_account_id().to_string();
    let prefixed_predecessor_account = format!(".{}", env::predecessor_account_id());
    assert!(
        account_id.ends_with(prefixed_predecessor_account.as_str()),
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
