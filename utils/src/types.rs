use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct AssetDenom {
    pub trace_path: String,
    pub base_denom: String,
}
