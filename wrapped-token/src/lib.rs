use near_contract_standards::fungible_token::{
    events::{FtBurn, FtMint},
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
    FungibleToken,
};
use near_sdk::{
    assert_self,
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::{U128, U64},
    log, near_bindgen,
    serde::{Deserialize, Serialize},
    store::UnorderedMap,
    AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue,
};
use utils::{types::MsgTransferPlan, BALANCE_FOR_TOKEN_CONTRACT_MINT, GAS_FOR_DO_SEND_TRANSFER};

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
pub struct WrappedToken {
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
    /// Accounting for the pending burning requests.
    pending_burnings: UnorderedMap<AccountId, Vec<MsgTransferPlan>>,
}

#[near_bindgen]
impl WrappedToken {
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
            pending_burnings: UnorderedMap::new(StorageKey::PendingBurnings),
        };
        this.token
            .internal_register_account(&env::current_account_id());
        this
    }
    /// Asserts that the predecessor account is `near_ibc_account`.
    fn assert_near_ibc_account(&self) {
        assert_eq!(
            env::predecessor_account_id(),
            self.near_ibc_account,
            "ERR_ONLY_NEAR_IBC_ACCOUNT_CAN_CALL_THIS_METHOD"
        );
    }
    /// Mint tokens to the given account.
    ///
    /// Only the parent account can call this method.
    #[payable]
    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        utils::assert_parent_account();
        assert!(
            env::attached_deposit() >= BALANCE_FOR_TOKEN_CONTRACT_MINT,
            "ERR_NOT_ENOUGH_DEPOSIT"
        );
        self.storage_deposit(Some(account_id.clone()), None);
        self.token.internal_deposit(&account_id, amount.into());
        FtMint {
            owner_id: &account_id,
            amount: &amount,
            memo: None,
        }
        .emit();
    }
    /// Burn tokens from the given account.
    ///
    /// Only the parent account can call this method.
    ///
    /// The calling is triggered by the IBC/TAO implementation, when all checkings
    /// are passed for a `send_transfer` request from this contract.
    /// The account id and amount must match a certain pending burning.
    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.assert_near_ibc_account();
        let remained_pending_burnings = self.assert_pending_burning(&account_id, amount);
        self.token
            .internal_withdraw(&env::current_account_id(), amount.into());
        self.update_pending_burnings_for(&account_id, remained_pending_burnings);
        FtBurn {
            owner_id: &env::current_account_id(),
            amount: &amount,
            memo: None,
        }
        .emit()
    }
    /// Assert that the given account has a pending burning request with the given amount.
    fn assert_pending_burning(&self, account_id: &AccountId, amount: U128) -> Vec<MsgTransferPlan> {
        assert!(
            self.pending_burnings.contains_key(&account_id),
            "ERR_NO_PENDING_BURNING"
        );
        let pending_burnings = self.pending_burnings.get(&account_id).unwrap();
        let mut remained_pending_burnings: Vec<MsgTransferPlan> = vec![];
        let mut matched = false;
        for msg in pending_burnings {
            if !matched && msg.amount == amount {
                matched = true;
            } else {
                remained_pending_burnings.push(msg.clone());
            }
        }
        assert!(matched, "ERR_NO_MATCHED_PENDING_BURNING");
        remained_pending_burnings
    }
    /// Update the pending burnings for the given account.
    fn update_pending_burnings_for(
        &mut self,
        account_id: &AccountId,
        remained_pending_burnings: Vec<MsgTransferPlan>,
    ) {
        if remained_pending_burnings.len() > 0 {
            self.pending_burnings
                .insert(account_id.clone(), remained_pending_burnings);
        } else {
            self.pending_burnings.remove(&account_id);
        }
    }
    /// Request to burn a certain amount of tokens, for sending them to another chain.
    ///
    /// This function is called by a certain token holder, when he/she wants to redeem
    /// the token on NEAR protocol back to the source chain. It will send
    /// a transfer plan to the IBC/TAO implementation.
    pub fn request_burning(&mut self, receiver_id: String, amount: U128) {
        assert!(amount.0 > 0, "ERR_AMOUNT_MUST_BE_GREATER_THAN_ZERO");
        let sender = env::predecessor_account_id();
        assert!(
            self.token.ft_balance_of(sender.clone()) >= amount,
            "ERR_NOT_ENOUGH_BALANCE"
        );
        let msg = MsgTransferPlan {
            port_on_a: self.port_id.clone(),
            chan_on_a: self.channel_id.clone(),
            token_trace_path: self.trace_path.clone(),
            token_denom: self.base_denom.clone(),
            amount,
            sender: sender.to_string(),
            receiver: receiver_id,
        };
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            msg_transfer_plan: MsgTransferPlan,
        }
        let args = Input {
            msg_transfer_plan: msg.clone(),
        };
        let args =
            near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_DO_SEND_TRANSFER");
        Promise::new(self.near_ibc_account.clone()).function_call(
            "do_send_transfer".to_string(),
            args,
            0,
            GAS_FOR_DO_SEND_TRANSFER,
        );
        if self.pending_burnings.contains_key(&sender) {
            self.pending_burnings
                .get_mut(&env::predecessor_account_id())
                .unwrap()
                .push(msg.clone());
        } else {
            self.pending_burnings
                .insert(sender.clone(), vec![msg.clone()]);
        }
        self.token.internal_withdraw(&sender, amount.into());
        self.token
            .internal_deposit(&env::current_account_id(), amount.into());
        FtBurn {
            owner_id: &sender,
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
    /// Cancel a pending burning.
    ///
    /// Only the parent account can call this method.
    ///
    /// The calling is triggered by the IBC/TAO implementation, when error happens
    /// in processing a `send_transfer` request from this contract.
    /// The account id and amount must match a certain pending burning.
    pub fn cancel_burning(&mut self, account_id: AccountId, amount: U128) {
        self.assert_near_ibc_account();
        let remained_pending_burnings = self.assert_pending_burning(&account_id, amount);
        self.token
            .internal_withdraw(&env::current_account_id(), amount.into());
        self.token.internal_deposit(&account_id, amount.into());
        self.update_pending_burnings_for(&account_id, remained_pending_burnings);
        FtBurn {
            owner_id: &env::current_account_id(),
            amount: &amount,
            memo: None,
        }
        .emit();
        FtMint {
            owner_id: &account_id,
            amount: &amount,
            memo: None,
        }
        .emit();
    }
    /// Set the icon to the token's metadata.
    ///
    /// Only the parent account can call this method.
    #[payable]
    pub fn set_icon(&mut self, icon: String) {
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
    /// Set the name, symbol and decimals to the token's metadata.
    ///
    /// Only the parent account can call this method.
    #[payable]
    pub fn set_basic_metadata(&mut self, name: String, symbol: String, decimals: u8) {
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

near_contract_standards::impl_fungible_token_core!(WrappedToken, token);
near_contract_standards::impl_fungible_token_storage!(WrappedToken, token);
utils::impl_storage_check_and_refund!(WrappedToken);

#[near_bindgen]
impl FungibleTokenMetadataProvider for WrappedToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

/// View functions for the wrapped token.
#[near_bindgen]
impl WrappedToken {
    ///
    pub fn get_pending_accounts(&self) -> Vec<AccountId> {
        self.pending_burnings
            .keys()
            .map(|account_id| account_id.clone())
            .collect()
    }
    ///
    pub fn get_pending_burnings(&self, account_id: AccountId) -> Vec<MsgTransferPlan> {
        self.pending_burnings
            .get(&account_id)
            .map_or_else(|| vec![], |msgs| msgs.clone())
    }
}

/// Re-deploy the contract code.
/// Implemented to avoid loading the data into WASM for optimal gas usage.
#[no_mangle]
pub extern "C" fn update_contract_code() {
    env::setup_panic_hook();
    let _contract: WrappedToken = env::state_read().expect("ERR_CONTRACT_IS_NOT_INITIALIZED");
    utils::assert_parent_account();

    let input = env::input().expect("ERR_NO_INPUT");
    Promise::new(env::current_account_id()).deploy_contract(input);
}
