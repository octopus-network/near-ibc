use crate::prelude::*;
use near_contract_standards::fungible_token::metadata::FungibleTokenMetadata;
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    json_types::{U128, U64},
    serde::{Deserialize, Serialize},
};

#[derive(
    BorshDeserialize, BorshSerialize, Clone, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct AssetDenom {
    pub trace_path: String,
    pub base_denom: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct CrossChainAsset {
    pub asset_id: String,
    pub asset_denom: AssetDenom,
    pub metadata: FungibleTokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub struct Ics20TransferRequest {
    pub port_on_a: String,
    pub chan_on_a: String,
    pub token_trace_path: String,
    pub token_denom: String,
    pub amount: U128,
    pub sender: String,
    pub receiver: String,
    pub timeout_seconds: Option<U64>,
}
