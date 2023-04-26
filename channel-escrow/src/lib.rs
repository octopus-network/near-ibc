use ibc::core::ics24_host::identifier::ChannelId;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::{Base58CryptoHash, U128},
    near_bindgen,
    store::UnorderedSet,
    AccountId, Balance, BorshStorageKey, Gas, PanicOnDefault, PromiseOrValue,
};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    ChannelIdSet,
    EscrowContractWasm,
}

#[derive(BorshSerialize, BorshStorageKey)]
pub struct BaseDenom(String);

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct ChannelEscrow {
    channel_id_set: UnorderedSet<ChannelId>,
}

#[near_bindgen]
impl ChannelEscrow {
    #[init]
    pub fn new() -> Self {
        assert!(!env::state_exists(), "ERR_ALREADY_INITIALIZED");
        let account_id = String::from(env::current_account_id().as_str());
        let parts = account_id.split(".").collect::<Vec<&str>>();
        assert!(
            parts.len() > 2,
            "ERR_CONTRACT_MUST_BE_DEPLOYED_IN_SUB_ACCOUNT",
        );
        Self {
            channel_id_set: UnorderedSet::new(StorageKey::ChannelIdSet),
        }
    }
    /// Callback function for `ft_transfer_call` of NEP-141 compatible contracts
    pub fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        todo!()
    }
    ///
    pub fn transfer(&mut self, base_denom: String, receiver_id: AccountId, amount: U128) {
        assert_root_account();
        todo!()
    }
}

//
fn assert_root_account() {
    let account_id = String::from(env::current_account_id().as_str());
    let parts = account_id.split(".").collect::<Vec<&str>>();
    let root_account = format!("{}.{}", parts[parts.len() - 2], parts[parts.len() - 1]);
    assert_eq!(
        env::predecessor_account_id().to_string(),
        root_account,
        "ERR_ONLY_ROOT_ACCOUNT_CAN_CALL_THIS_METHOD"
    );
}
