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
    string::{String, ToString},
    vec::Vec,
};
use core::str::FromStr;
use ibc::apps::transfer::types::TracePath;
use near_contract_standards::{
    fungible_token::{
        events::{FtBurn, FtMint},
        metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
        FungibleToken, FungibleTokenCore, FungibleTokenResolver,
    },
    storage_management::{StorageBalance, StorageBalanceBounds, StorageManagement},
};
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::{U128, U64},
    near_bindgen,
    store::UnorderedMap,
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseOrValue,
};
use utils::{
    interfaces::{
        ext_transfer_request_handler, NearIbcAccountAssertion, ProcessTransferRequestCallback,
        WrappedToken,
    },
    types::Ics20TransferRequest,
};

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    Token,
    Metadata,
    PendingBurnings,
}

/// A simple wrapper of NEP-141 fungible token.
///
/// An instance of this contract is used to represent a certain fungible token
/// from another chain on NEAR protocol.
#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    /// The NEP-141 fungible token implementation.
    token: FungibleToken,
    /// The metadata of the token.
    metadata: LazyOption<FungibleTokenMetadata>,
    /// The trace path of the token, in ICS-20 of IBC protocol.
    trace_path: String,
    /// The base denom of the token, in ICS-20 of IBC protocol.
    base_denom: String,
    /// The account id of IBC/TAO implementation.
    near_ibc_account: AccountId,
    /// Accounting for the pending transfer requests.
    pending_transfer_requests: UnorderedMap<AccountId, Ics20TransferRequest>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        metadata: FungibleTokenMetadata,
        trace_path: String,
        base_denom: String,
        near_ibc_account: AccountId,
    ) -> Self {
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 3,
            "ERR_CONTRACT_MUST_BE_DEPLOYED_IN_SUB_ACCOUNT_OF_FACTORY",
        );
        metadata.assert_valid();
        assert!(
            env::current_account_id()
                .to_string()
                .ends_with(near_ibc_account.as_str()),
            "ERR_NEAR_IBC_ACCOUNT_MUST_HAVE_THE_SAME_ROOT_ACOUNT_AS_CURRENT_ACCOUNT"
        );
        let maybe_trace_path =
            TracePath::from_str(trace_path.as_str()).expect("ERR_INVALID_TRACE_PATH");
        // As this contract will only be initialized by the first time a cross chain asset
        // is received by `near-ibc`, the trace path will at least have 1 trace prefix
        // which is composed of the receiving port id and receiving channel id.
        assert!(
            !maybe_trace_path.is_empty(),
            "ERR_TRACE_PATH_MUST_NOT_BE_EMPTY"
        );
        let mut this = Self {
            token: FungibleToken::new(StorageKey::Token),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            trace_path,
            base_denom,
            near_ibc_account,
            pending_transfer_requests: UnorderedMap::new(StorageKey::PendingBurnings),
        };
        this.token
            .internal_register_account(&env::current_account_id());
        this
    }
    /// Request a transfer by burning a certain amount of tokens,
    /// for sending them to another chain.
    ///
    /// This function is called by a certain token holder, when he/she wants to redeem
    /// the token on NEAR protocol back to the source chain. It will send
    /// a transfer plan to the IBC/TAO implementation.
    pub fn request_transfer(
        &mut self,
        receiver_id: String,
        amount: U128,
        timeout_seconds: Option<U64>,
    ) {
        assert!(amount.0 > 0, "ERR_AMOUNT_MUST_BE_GREATER_THAN_ZERO");
        let sender_id = env::predecessor_account_id();
        assert!(
            self.token.ft_balance_of(sender_id.clone()) >= amount,
            "ERR_NOT_ENOUGH_BALANCE"
        );
        assert!(
            !self.pending_transfer_requests.contains_key(&sender_id),
            "ERR_PENDING_TRANSFER_REQUEST_EXISTS"
        );
        let trace_path_parts: Vec<&str> = self.trace_path.split('/').collect();
        // Schedule a call to `process_transfer_request` on `near-ibc` contract.
        // As the `self.trace_path` is already validated in the constructor,
        // we can safely use the first 2 parts of `self.trace_path` as the source port id
        // and source channel id.
        let transfer_request = Ics20TransferRequest {
            port_on_a: trace_path_parts[0].to_string(),
            chan_on_a: trace_path_parts[1].to_string(),
            token_trace_path: self.trace_path.clone(),
            token_denom: self.base_denom.clone(),
            amount,
            sender: sender_id.to_string(),
            receiver: receiver_id,
            timeout_seconds,
        };
        ext_transfer_request_handler::ext(self.near_ibc_account.clone())
            .with_attached_deposit(NearToken::from_yoctonear(0))
            .with_static_gas(utils::GAS_FOR_COMPLEX_FUNCTION_CALL)
            .with_unused_gas_weight(0)
            .process_transfer_request(transfer_request.clone());
        // Record the pending transfer request.
        self.pending_transfer_requests
            .insert(sender_id.clone(), transfer_request);
        // Transfer the tokens to the current account.
        self.token.internal_withdraw(&sender_id, amount.into());
        self.token
            .internal_deposit(&env::current_account_id(), amount.into());
        // Generate events.
        FtBurn {
            owner_id: &sender_id,
            amount,
            memo: None,
        }
        .emit();
        FtMint {
            owner_id: &env::current_account_id(),
            amount,
            memo: None,
        }
        .emit();
    }
    /// Assert that the given account has a pending transfer request with the given amount.
    fn checked_remove_pending_transfer_request(
        &mut self,
        trace_path: &String,
        base_denom: &String,
        account_id: &AccountId,
        amount: U128,
    ) {
        assert!(
            self.pending_transfer_requests.contains_key(account_id),
            "ERR_NO_PENDING_TRANSFER_REQUEST"
        );
        let req = self.pending_transfer_requests.get(account_id).unwrap();
        assert!(
            req.amount == amount
                && req.token_denom.eq(base_denom)
                && req.token_trace_path.eq(trace_path),
            "ERR_PENDING_TRANSFER_REQUEST_NOT_MATCHED"
        );
        self.pending_transfer_requests.remove(account_id);
    }
    ///
    pub fn get_pending_accounts(&self) -> Vec<AccountId> {
        self.pending_transfer_requests
            .keys()
            .map(|account_id| account_id.clone())
            .collect()
    }
    ///
    pub fn get_pending_transfer_request_of(
        &self,
        account_id: AccountId,
    ) -> Option<Ics20TransferRequest> {
        self.pending_transfer_requests
            .get(&account_id)
            .map(|req| req.clone())
    }
}

#[near_bindgen]
impl FungibleTokenCore for Contract {
    fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }

    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        self.token.ft_transfer(receiver_id, amount, memo)
    }

    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: U128,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.token.ft_transfer_call(receiver_id, amount, memo, msg)
    }
}

#[near_bindgen]
impl FungibleTokenResolver for Contract {
    fn ft_resolve_transfer(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) -> U128 {
        self.token
            .ft_resolve_transfer(sender_id, receiver_id, amount)
    }
}

#[near_bindgen]
impl StorageManagement for Contract {
    fn storage_deposit(
        &mut self,
        account_id: Option<AccountId>,
        registration_only: Option<bool>,
    ) -> StorageBalance {
        self.token.storage_deposit(account_id, registration_only)
    }

    fn storage_withdraw(&mut self, amount: Option<NearToken>) -> StorageBalance {
        self.token.storage_withdraw(amount)
    }

    fn storage_unregister(&mut self, force: Option<bool>) -> bool {
        self.token.storage_unregister(force)
    }

    fn storage_balance_bounds(&self) -> StorageBalanceBounds {
        self.token.storage_balance_bounds()
    }

    fn storage_balance_of(&self, account_id: AccountId) -> Option<StorageBalance> {
        self.token.storage_balance_of(account_id)
    }
}

#[near_bindgen]
impl WrappedToken for Contract {
    #[payable]
    fn mint(&mut self, account_id: AccountId, amount: U128) {
        utils::assert_parent_account();
        let used_bytes = env::storage_usage();
        self.storage_deposit(Some(account_id.clone()), None);
        self.token.internal_deposit(&account_id, amount.into());
        utils::refund_deposit(used_bytes);
        FtMint {
            owner_id: &account_id,
            amount,
            memo: None,
        }
        .emit();
    }

    #[payable]
    fn set_icon(&mut self, icon: String) {
        utils::assert_parent_account();
        assert!(
            env::attached_deposit().as_yoctonear()
                >= env::storage_byte_cost().as_yoctonear()
                    * icon.clone().into_bytes().len() as u128,
            "ERR_NOT_ENOUGH_DEPOSIT"
        );
        let used_bytes = env::storage_usage();
        let mut metadata = self.metadata.get().unwrap();
        metadata.icon = Some(icon);
        self.metadata.set(&metadata);
        // Refund the unused attached deposit.
        utils::refund_deposit(used_bytes);
    }
}

#[near_bindgen]
impl ProcessTransferRequestCallback for Contract {
    fn apply_transfer_request(
        &mut self,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    ) {
        self.assert_near_ibc_account();
        self.checked_remove_pending_transfer_request(&trace_path, &base_denom, &sender_id, amount);
        self.token
            .internal_withdraw(&env::current_account_id(), amount.into());
        FtBurn {
            owner_id: &env::current_account_id(),
            amount,
            memo: None,
        }
        .emit()
    }

    fn cancel_transfer_request(
        &mut self,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    ) {
        self.assert_near_ibc_account();
        self.checked_remove_pending_transfer_request(&trace_path, &base_denom, &sender_id, amount);
        self.token
            .internal_withdraw(&env::current_account_id(), amount.into());
        self.token.internal_deposit(&sender_id, amount.into());
        FtBurn {
            owner_id: &env::current_account_id(),
            amount,
            memo: None,
        }
        .emit();
        FtMint {
            owner_id: &sender_id,
            amount,
            memo: None,
        }
        .emit();
    }
}

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

impl NearIbcAccountAssertion for Contract {
    fn near_ibc_account(&self) -> AccountId {
        self.near_ibc_account.clone()
    }
}

/// Re-deploy the contract code.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn update_contract_code() {
    env::setup_panic_hook();
    let _contract: Contract = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    utils::assert_parent_account();

    let input = env::input().expect("ERR_NO_INPUT");
    Promise::new(env::current_account_id()).deploy_contract(input);
}
