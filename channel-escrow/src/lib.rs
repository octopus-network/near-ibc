use ibc::core::ics24_host::identifier::ChannelId;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    env,
    json_types::U128,
    near_bindgen,
    store::UnorderedSet,
    AccountId, BorshStorageKey, PanicOnDefault, PromiseOrValue,
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
        todo!()
    }
}
