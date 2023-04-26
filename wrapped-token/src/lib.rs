use near_contract_standards::fungible_token::{
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
    FungibleToken,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::U128,
    near_bindgen, AccountId, PanicOnDefault, Promise, PromiseOrValue,
};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct WrappedToken {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

#[near_bindgen]
impl WrappedToken {
    #[init]
    pub fn new(metadata: FungibleTokenMetadata) -> Self {
        assert!(!env::state_exists(), "ERR_ALREADY_INITIALIZED");
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "ERR_CONTRACT_MUST_BE_DEPLOYED_IN_SUB_ACCOUNT",
        );
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token
            .internal_register_account(&env::current_account_id());
        this
    }
    /// Mint tokens to the given account.
    /// Only the parent account can call this method.
    #[payable]
    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        assert_parent();
        self.storage_deposit(Some(account_id.clone()), None);
        self.token.internal_deposit(&account_id, amount.into());
    }
    /// Burn tokens from the given account.
    /// Only the parent account can call this method.
    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        assert_parent();
        self.token.internal_withdraw(&account_id, amount.into());
    }
    /// Set the icon to the token's metadata.
    /// Only the parent account can call this method.
    #[payable]
    pub fn set_icon(&mut self, icon: String) {
        assert_parent();
        assert!(
            env::attached_deposit()
                >= env::storage_byte_cost() * icon.clone().into_bytes().len() as u128,
            "ERR_NOT_ENOUGH_DEPOSIT"
        );
        let used_bytes = env::storage_usage();
        let mut metadata = self.metadata.get().unwrap();
        metadata.icon = Some(icon);
        self.metadata.set(&metadata);
        refund_deposit(used_bytes);
    }
    /// Set the name, symbol and decimals to the token's metadata.
    /// Only the parent account can call this method.
    #[payable]
    pub fn set_basic_metadata(&mut self, name: String, symbol: String, decimals: u8) {
        assert_parent();
        assert!(
            env::attached_deposit()
                >= env::storage_byte_cost()
                    * (name.clone().into_bytes().len() + symbol.clone().into_bytes().len()) as u128,
            "ERR_NOT_ENOUGH_DEPOSIT"
        );
        let used_bytes = env::storage_usage();
        let mut metadata = self.metadata.get().unwrap();
        metadata.name = name;
        metadata.symbol = symbol;
        metadata.decimals = decimals;
        self.metadata.set(&metadata);
        refund_deposit(used_bytes);
    }
}

// Asserts that the predecessor is the parent account.
fn assert_parent() {
    let account_id = String::from(env::current_account_id().as_str());
    let (_first, parent) = account_id.split_once(".").unwrap();
    assert_eq!(
        env::predecessor_account_id().as_str(),
        parent,
        "ERR_ONLY_PARENT_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}

fn refund_deposit(previously_used_bytes: u64) {
    if env::storage_usage() > previously_used_bytes {
        let newly_used_bytes = env::storage_usage() - previously_used_bytes;
        let refund_amount =
            env::attached_deposit() - env::storage_byte_cost() * newly_used_bytes as u128;
        Promise::new(env::predecessor_account_id()).transfer(refund_amount);
    }
}

near_contract_standards::impl_fungible_token_core!(WrappedToken, token);
near_contract_standards::impl_fungible_token_storage!(WrappedToken, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for WrappedToken {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
