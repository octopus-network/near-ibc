#![no_std]
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
    vec,
    vec::Vec,
};
use ibc::applications::transfer::PORT_ID_STR;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json,
    store::UnorderedMap,
    AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use utils::{
    interfaces::{
        ext_transfer_request_handler, ChannelEscrow, NearIbcAccountAssertion,
        ProcessTransferRequestCallback,
    },
    types::{AssetDenom, Ics20TransferRequest},
};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    TokenContracts,
    PendingTransferRequests,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FtOnTransferMsg {
    pub receiver: String,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RegisteredAsset {
    pub token_contract: AccountId,
    pub asset_denom: AssetDenom,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    /// The account id of IBC/TAO implementation.
    near_ibc_account: AccountId,
    /// The token accounts that this contract is allowed to send tokens to.
    token_contracts: UnorderedMap<AccountId, AssetDenom>,
    /// Accounting for the pending transfer requests.
    pending_transfer_requests: UnorderedMap<AccountId, Ics20TransferRequest>,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(near_ibc_account: AccountId) -> Self {
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "ERR_CONTRACT_MUST_BE_DEPLOYED_IN_SUB_ACCOUNT",
        );
        Self {
            near_ibc_account,
            token_contracts: UnorderedMap::new(StorageKey::TokenContracts),
            pending_transfer_requests: UnorderedMap::new(StorageKey::PendingTransferRequests),
        }
    }
    /// Callback function for `ft_transfer_call` of NEP-141 compatible contracts
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let token_denom = self.token_contracts.get(&env::predecessor_account_id());
        assert!(token_denom.is_some(), "ERR_UNREGISTERED_TOKEN_CONTRACT");
        assert!(
            !self.pending_transfer_requests.contains_key(&sender_id),
            "ERR_PENDING_TRANSFER_REQUEST_EXISTS"
        );
        let parse_result: Result<FtOnTransferMsg, _> = serde_json::from_str(msg.as_str());
        assert!(
            parse_result.is_ok(),
            "Invalid msg '{}' attached in `ft_transfer_call`. Refund deposit.",
            msg
        );
        let msg = parse_result.unwrap();
        let current_account_id = env::current_account_id();
        let (channel_id, _) = current_account_id.as_str().split_once(".").unwrap();
        let token_denom = token_denom.unwrap();
        let transfer_request = Ics20TransferRequest {
            port_on_a: PORT_ID_STR.to_string(),
            chan_on_a: channel_id.to_string(),
            token_trace_path: token_denom.trace_path.clone(),
            token_denom: token_denom.base_denom.clone(),
            amount,
            sender: sender_id.to_string(),
            receiver: msg.receiver,
        };
        ext_transfer_request_handler::ext(self.near_ibc_account())
            .with_attached_deposit(0)
            .with_static_gas(utils::GAS_FOR_COMPLEX_FUNCTION_CALL)
            .with_unused_gas_weight(0)
            .process_transfer_request(transfer_request.clone());
        self.pending_transfer_requests
            .insert(sender_id, transfer_request);

        PromiseOrValue::Value(0.into())
    }
    /// Assert that the given account has a pending burning request with the given amount.
    fn checked_remove_pending_transfer_request(
        &mut self,
        trace_path: &String,
        base_denom: &String,
        account_id: &AccountId,
        amount: U128,
    ) {
        assert!(
            self.pending_transfer_requests.contains_key(&account_id),
            "ERR_NO_PENDING_TRANSFER_REQUEST"
        );
        let req = self.pending_transfer_requests.get(&account_id).unwrap();
        assert!(
            req.amount == amount
                && req.token_denom.eq(base_denom)
                && req.token_trace_path.eq(trace_path),
            "ERR_PENDING_TRANSFER_REQUEST_NOT_MATCHED"
        );
        self.pending_transfer_requests.remove(&account_id);
    }
    // Get token contract account id corresponding to the asset denom.
    fn get_token_contract_by_asset_denom(&self, asset_denom: &AssetDenom) -> Option<AccountId> {
        self.token_contracts
            .iter()
            .find(|(_, value)| *value == asset_denom)
            .map(|(id, _)| id.clone())
    }
}

#[near_bindgen]
impl ChannelEscrow for Contract {
    #[payable]
    fn register_asset(
        &mut self,
        trace_path: String,
        base_denom: String,
        token_contract: AccountId,
    ) {
        self.assert_near_ibc_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        let maybe_existed_token_contract = self.get_token_contract_by_asset_denom(&asset_denom);
        assert!(
            maybe_existed_token_contract.is_none(),
            "ERR_TOKEN_CONTRACT_ALREADY_REGISTERED"
        );
        self.token_contracts.insert(token_contract, asset_denom);
    }

    #[payable]
    fn do_transfer(
        &mut self,
        trace_path: String,
        base_denom: String,
        receiver_id: AccountId,
        amount: U128,
    ) {
        self.assert_near_ibc_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        let maybe_existed_token_contract = self.get_token_contract_by_asset_denom(&asset_denom);
        assert!(
            maybe_existed_token_contract.is_some(),
            "ERR_INVALID_TOKEN_DENOM"
        );
        near_sdk::assert_one_yocto();
        let token_contract = maybe_existed_token_contract.unwrap();
        ext_ft_core::ext(token_contract.clone())
            .with_attached_deposit(1)
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 2)
            .with_unused_gas_weight(0)
            .ft_transfer(receiver_id, amount.into(), None);
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
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        let maybe_existed_token_contract = self.get_token_contract_by_asset_denom(&asset_denom);
        assert!(
            maybe_existed_token_contract.is_some(),
            "ERR_INVALID_TOKEN_DENOM"
        );
        self.checked_remove_pending_transfer_request(
            &asset_denom.trace_path,
            &asset_denom.base_denom,
            &sender_id,
            amount,
        );
    }

    fn cancel_transfer_request(
        &mut self,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    ) {
        self.assert_near_ibc_account();
        let asset_denom = AssetDenom {
            trace_path,
            base_denom,
        };
        let maybe_existed_token_contract = self.get_token_contract_by_asset_denom(&asset_denom);
        assert!(
            maybe_existed_token_contract.is_some(),
            "ERR_INVALID_TOKEN_DENOM"
        );
        self.checked_remove_pending_transfer_request(
            &asset_denom.trace_path,
            &asset_denom.base_denom,
            &sender_id,
            amount,
        );
        let token_contract = maybe_existed_token_contract.unwrap();
        ext_ft_core::ext(token_contract.clone())
            .with_attached_deposit(1)
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 2)
            .with_unused_gas_weight(0)
            .ft_transfer(sender_id, amount.into(), None);
    }
}

impl NearIbcAccountAssertion for Contract {
    fn near_ibc_account(&self) -> AccountId {
        self.near_ibc_account.clone()
    }
}

/// View functions.
#[near_bindgen]
impl Contract {
    ///
    pub fn get_registered_assets(&self) -> Vec<RegisteredAsset> {
        self.token_contracts
            .iter()
            .map(|(token_contract, asset_denom)| RegisteredAsset {
                token_contract: token_contract.clone(),
                asset_denom: asset_denom.clone(),
            })
            .collect()
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
