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
use ibc::apps::transfer::types::PORT_ID_STR;
use near_contract_standards::fungible_token::core::ext_ft_core;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    env, ext_contract,
    json_types::{U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    serde_json,
    store::{LookupMap, UnorderedMap},
    AccountId, BorshStorageKey, NearToken, PanicOnDefault, Promise, PromiseOrValue, PromiseResult,
};
use utils::{
    interfaces::{
        ext_transfer_request_handler, ChannelEscrow, NearIbcAccountAssertion,
        ProcessTransferRequestCallback,
    },
    types::{AssetDenom, Ics20TransferRequest},
};

mod migration;

#[derive(BorshSerialize, BorshStorageKey)]
#[borsh(crate = "near_sdk::borsh")]
pub enum StorageKey {
    TokenContracts,
    PendingTransferRequests,
    DenomToTokenContractMap,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FtOnTransferMsg {
    pub receiver: String,
    #[serde(default)]
    pub timeout_seconds: Option<U64>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RegisteredAsset {
    pub token_contract: AccountId,
    pub asset_denom: AssetDenom,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Contract {
    /// The account id of IBC/TAO implementation.
    near_ibc_account: AccountId,
    /// The token accounts that this contract is allowed to send tokens to.
    token_contracts: UnorderedMap<AccountId, AssetDenom>,
    /// Accounting for the pending transfer requests.
    pending_transfer_requests: UnorderedMap<AccountId, Ics20TransferRequest>,
    /// The mapping from the asset denom to the token contract account id.
    denom_to_token_contract_map: LookupMap<AssetDenom, AccountId>,
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
            denom_to_token_contract_map: LookupMap::new(StorageKey::DenomToTokenContractMap),
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
            timeout_seconds: msg.timeout_seconds,
        };
        ext_transfer_request_handler::ext(self.near_ibc_account())
            .with_attached_deposit(NearToken::from_yoctonear(0))
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
}

#[ext_contract(ext_ft_transfer_callback)]
pub trait FtTransferCallback {
    fn ft_transfer_callback(
        &mut self,
        token_contract: AccountId,
        receiver_id: AccountId,
        amount: U128,
    );
}

#[near_bindgen]
impl FtTransferCallback for Contract {
    #[private]
    fn ft_transfer_callback(
        &mut self,
        token_contract: AccountId,
        receiver_id: AccountId,
        amount: U128,
    ) {
        match env::promise_result(0) {
            PromiseResult::Successful(_bytes) => {
                log!(
                    r#"EVENT_JSON:{{"standard":"nep297","version":"1.0.0","event":"FT_TRANSFER_SUCCEEDED","token_contract":"{}","receiver_id":"{}","amount":"{}"}}"#,
                    token_contract,
                    receiver_id,
                    amount.0,
                );
            }
            PromiseResult::Failed => {
                log!(
                    r#"EVENT_JSON:{{"standard":"nep297","version":"1.0.0","event":"ERR_FT_TRANSFER","token_contract":"{}","receiver_id":"{}","amount":"{}"}}"#,
                    token_contract,
                    receiver_id,
                    amount.0,
                );
            }
        }
    }
}

#[near_bindgen]
impl ChannelEscrow for Contract {
    #[payable]
    fn register_asset(&mut self, base_denom: String, token_contract: AccountId) {
        self.assert_near_ibc_account();
        let asset_denom = AssetDenom {
            trace_path: String::new(),
            base_denom,
        };
        let maybe_existed_token_contract = self.denom_to_token_contract_map.get(&asset_denom);
        assert!(
            maybe_existed_token_contract.is_none(),
            "ERR_TOKEN_CONTRACT_ALREADY_REGISTERED"
        );
        self.token_contracts
            .insert(token_contract.clone(), asset_denom.clone());
        self.denom_to_token_contract_map
            .insert(asset_denom, token_contract);
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
        let maybe_existed_token_contract = self.denom_to_token_contract_map.get(&asset_denom);
        assert!(
            maybe_existed_token_contract.is_some(),
            "ERR_INVALID_TOKEN_DENOM"
        );
        near_sdk::assert_one_yocto();
        let token_contract = maybe_existed_token_contract.unwrap();
        ext_ft_core::ext(token_contract.clone())
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(2))
            .with_unused_gas_weight(0)
            .ft_transfer(receiver_id.clone(), amount, None)
            .then(
                ext_ft_transfer_callback::ext(env::current_account_id())
                    .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                    .with_unused_gas_weight(0)
                    .ft_transfer_callback(token_contract.clone(), receiver_id, amount),
            );
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
        let maybe_existed_token_contract = self
            .denom_to_token_contract_map
            .get(&asset_denom)
            .map(|v| v.clone());
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
        let maybe_existed_token_contract = self
            .denom_to_token_contract_map
            .get(&asset_denom)
            .map(|v| v.clone());
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
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(2))
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
pub trait Viewer {
    /// Get all registered assets.
    fn get_registered_assets(&self) -> Vec<RegisteredAsset>;
    /// Get all pending accounts.
    fn get_pending_accounts(&self) -> Vec<AccountId>;
    /// Get pending transfer request of the given account.
    fn get_pending_transfer_request_of(
        &self,
        account_id: AccountId,
    ) -> Option<Ics20TransferRequest>;
}

#[near_bindgen]
impl Viewer for Contract {
    ///
    fn get_registered_assets(&self) -> Vec<RegisteredAsset> {
        self.token_contracts
            .iter()
            .map(|(token_contract, asset_denom)| RegisteredAsset {
                token_contract: token_contract.clone(),
                asset_denom: asset_denom.clone(),
            })
            .collect()
    }
    ///
    fn get_pending_accounts(&self) -> Vec<AccountId> {
        self.pending_transfer_requests
            .keys()
            .map(|account_id| account_id.clone())
            .collect()
    }
    ///
    fn get_pending_transfer_request_of(
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
