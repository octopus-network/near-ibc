use crate::prelude::*;
use ibc::{
    core::{
        ics04_channel::packet::Sequence,
        ics24_host::identifier::{ChannelId, PortId},
    },
    Height,
};
use near_sdk::{
    json_types::U64,
    serde::{Deserialize, Serialize},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum QueryHeight {
    Latest,
    Specific(Height),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Qualified<T> {
    SmallerEqual(T),
    Equal(T),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct QueryPacketEventDataRequest {
    pub event_type: String,
    pub source_channel_id: ChannelId,
    pub source_port_id: PortId,
    pub destination_channel_id: ChannelId,
    pub destination_port_id: PortId,
    pub sequences: Vec<Sequence>,
    pub height: Qualified<QueryHeight>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum ProcessingResult {
    NeedMoreGas,
    Ok,
    Error(String),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ValidatorKeyAndPower {
    pub public_key: Vec<u8>,
    pub power: U64,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct VscPacketData {
    pub validator_pubkeys: Vec<ValidatorKeyAndPower>,
    pub validator_set_id: U64,
    pub slash_acks: Vec<Vec<u8>>,
}
