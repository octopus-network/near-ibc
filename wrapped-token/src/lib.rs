use near_contract_standards::fungible_token::{
    metadata::{FungibleTokenMetadata, FungibleTokenMetadataProvider},
    FungibleToken,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::LazyOption,
    env,
    json_types::U128,
    near_bindgen, AccountId, PanicOnDefault, PromiseOrValue,
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
        assert!(!env::state_exists(), "Already initialized.");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token
            .internal_register_account(&env::current_account_id());
        this
    }
    // Asserts that the predecessor is the parent account.
    fn assert_parent(&self) {
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "This contract must be deployed as a sub-account.",
        );
        let (_first, parent) = account_id.split_once(".").unwrap();
        assert_eq!(
            env::predecessor_account_id().as_str(),
            parent,
            "Only parent account can call this method."
        );
    }
    /// Mint tokens to the given account.
    /// Only the parent account can call this method.
    #[payable]
    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        self.assert_parent();
        self.storage_deposit(Some(account_id.clone()), None);
        self.token.internal_deposit(&account_id, amount.into());
    }
    /// Burn tokens from the given account.
    /// Only the parent account can call this method.
    #[payable]
    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.assert_parent();
        self.token.internal_withdraw(&account_id, amount.into());
    }
    /// Set the icon to the token's metadata.
    /// Only the parent account can call this method.
    pub fn set_icon(&mut self, icon: String) {
        self.assert_parent();
        let mut metadata = self.metadata.get().unwrap();
        metadata.icon = Some(icon);
        self.metadata.set(&metadata);
    }
    /// Set the name, symbol and decimals to the token's metadata.
    /// Only the parent account can call this method.
    pub fn set_basic_metadata(&mut self, name: String, symbol: String, decimals: u8) {
        self.assert_parent();
        let mut metadata = self.metadata.get().unwrap();
        metadata.name = name;
        metadata.symbol = symbol;
        metadata.decimals = decimals;
        self.metadata.set(&metadata);
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
