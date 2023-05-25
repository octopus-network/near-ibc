use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    json_types::U128,
    serde::{Deserialize, Serialize},
};

#[derive(BorshDeserialize, BorshSerialize, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub struct AssetDenom {
    pub trace_path: String,
    pub base_denom: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MsgTransferPlan {
    pub port_on_a: String,
    pub chan_on_a: String,
    pub token_trace_path: String,
    pub token_denom: String,
    pub amount: U128,
    pub sender: String,
    pub receiver: String,
}
