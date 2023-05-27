use near_contract_standards::fungible_token::{
    events::{FtBurn, FtMint},
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
    FungibleToken,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::U128,
    near_bindgen,
    store::UnorderedMap,
    AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use utils::{
    interfaces::{
        ext_transfer_request_handler, NearIbcAccountAssertion, ProcessTransferRequestCallback,
        WrappedToken,
    },
    types::Ics20TransferRequest,
};

#[derive(BorshSerialize, BorshStorageKey)]
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
pub struct Contract {
    /// The NEP-141 fungible token implementation.
    token: FungibleToken,
    /// The metadata of the token.
    metadata: LazyOption<FungibleTokenMetadata>,
    /// The port id of the token, in ICS-20 of IBC protocol.
    port_id: String,
    /// The channel id of the token, in ICS-20 of IBC protocol.
    channel_id: String,
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
        port_id: String,
        channel_id: String,
        trace_path: String,
        base_denom: String,
        near_ibc_account: AccountId,
    ) -> Self {
        assert!(!env::state_exists(), "ERR_ALREADY_INITIALIZED");
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
        let mut this = Self {
            token: FungibleToken::new(StorageKey::Token),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            port_id,
            channel_id,
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
    pub fn request_transfer(&mut self, receiver_id: String, amount: U128) {
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
        // Schedule a call to `process_transfer_request` on `near-ibc` contract.
        let transfer_request = Ics20TransferRequest {
            port_on_a: self.port_id.clone(),
            chan_on_a: self.channel_id.clone(),
            token_trace_path: self.trace_path.clone(),
            token_denom: self.base_denom.clone(),
            amount,
            sender: sender_id.to_string(),
            receiver: receiver_id,
        };
        ext_transfer_request_handler::ext(self.near_ibc_account.clone())
            .with_attached_deposit(0)
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
            amount: &amount,
            memo: None,
        }
        .emit();
        FtMint {
            owner_id: &env::current_account_id(),
            amount: &amount,
            memo: None,
        }
        .emit();
    }
    /// Assert that the given account has a pending transfer request with the given amount.
    fn checked_remove_pending_transfer_request(
        &mut self,
        base_denom: &String,
        account_id: &AccountId,
        amount: U128,
    ) {
        assert!(
            self.pending_transfer_requests.contains_key(&account_id),
            "ERR_NO_PENDING_TRANSFER_REQUEST"
        );
        let req = self.pending_transfer_requests.get(&account_id).unwrap();
        if !self.base_denom.eq(base_denom) || req.amount != amount {
            panic!("ERR_PENDING_TRANSFER_REQUEST_NOT_MATCHED")
        }
        self.pending_transfer_requests.remove(&account_id);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);
utils::impl_storage_check_and_refund!(Contract);

#[near_bindgen]
impl WrappedToken for Contract {
    #[payable]
    fn mint(&mut self, account_id: AccountId, amount: U128) {
        utils::assert_parent_account();
        let used_bytes = env::storage_usage();
        self.storage_deposit(Some(account_id.clone()), None);
        self.token.internal_deposit(&account_id, amount.into());
        FtMint {
            owner_id: &account_id,
            amount: &amount,
            memo: None,
        }
        .emit();
        utils::refund_deposit(used_bytes, env::attached_deposit());
    }

    #[payable]
    fn set_icon(&mut self, icon: String) {
        utils::assert_parent_account();
        assert!(
            env::attached_deposit()
                >= env::storage_byte_cost() * icon.clone().into_bytes().len() as u128,
            "ERR_NOT_ENOUGH_DEPOSIT"
        );
        let used_bytes = env::storage_usage();
        let mut metadata = self.metadata.get().unwrap();
        metadata.icon = Some(icon);
        self.metadata.set(&metadata);
        // Refund the unused attached deposit.
        utils::refund_deposit(used_bytes, env::attached_deposit());
    }

    #[payable]
    fn set_basic_metadata(&mut self, name: String, symbol: String, decimals: u8) {
        utils::assert_parent_account();
        assert!(
            env::attached_deposit()
                >= env::storage_byte_cost()
                    * (name.clone().into_bytes().len() + symbol.clone().into_bytes().len() + 1)
                        as u128,
            "ERR_NOT_ENOUGH_DEPOSIT"
        );
        let used_bytes = env::storage_usage();
        let mut metadata = self.metadata.get().unwrap();
        metadata.name = name;
        metadata.symbol = symbol;
        metadata.decimals = decimals;
        self.metadata.set(&metadata);
        // Refund the unused attached deposit.
        utils::refund_deposit(used_bytes, env::attached_deposit());
    }
}

#[near_bindgen]
impl ProcessTransferRequestCallback for Contract {
    fn apply_transfer_request(&mut self, base_denom: String, sender_id: AccountId, amount: U128) {
        self.assert_near_ibc_account();
        self.checked_remove_pending_transfer_request(&base_denom, &sender_id, amount);
        self.token
            .internal_withdraw(&env::current_account_id(), amount.into());
        FtBurn {
            owner_id: &env::current_account_id(),
            amount: &amount,
            memo: None,
        }
        .emit()
    }

    fn cancel_transfer_request(&mut self, base_denom: String, sender_id: AccountId, amount: U128) {
        self.assert_near_ibc_account();
        self.checked_remove_pending_transfer_request(&base_denom, &sender_id, amount);
        self.token
            .internal_withdraw(&env::current_account_id(), amount.into());
        self.token.internal_deposit(&sender_id, amount.into());
        FtBurn {
            owner_id: &env::current_account_id(),
            amount: &amount,
            memo: None,
        }
        .emit();
        FtMint {
            owner_id: &sender_id,
            amount: &amount,
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

/// View functions for the wrapped token.
#[near_bindgen]
impl Contract {
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
