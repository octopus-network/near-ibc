use ibc::core::events::IbcEvent;
use near_sdk::serde::Serialize;
use near_sdk::serde_json::{json, Value};
use near_sdk::{env, log};

const EVENT_STANDARD: &str = "near-ibc";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

pub trait EventEmit {
    fn emit(&self)
    where
        Self: Sized + Serialize;
}

pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let mut result = json!({ "raw-ibc-event": data });
    let map = result.as_object_mut().unwrap();

    map.insert(
        "standard".to_string(),
        Value::String(EVENT_STANDARD.to_string()),
    );
    map.insert(
        "version".to_string(),
        Value::String(EVENT_STANDARD_VERSION.to_string()),
    );
    map.insert(
        "block_height".to_string(),
        Value::String(env::block_height().to_string()),
    );
    map.insert(
        "epoch_height".to_string(),
        Value::String(env::epoch_height().to_string()),
    );

    log!(format!("EVENT_JSON:{}", result.to_string()));
}

impl EventEmit for IbcEvent {
    fn emit(&self)
    where
        Self: Sized + Serialize,
    {
        emit_event(&self)
    }
}

#[test]
fn event_test() {
    use std::str::FromStr;

    let event_json = "{\"raw-ibc-event\":{\"CreateClient\":{\"client_id\":{\"client_id\":\"07-tendermint-0\"},\"client_type\":{\"client_type\":\"07-tendermint\"},\"consensus_height\":{\"consensus_height\":{\"revision_height\":53650,\"revision_number\":0}}}},\"standard\":\"near-ibc\",\"version\":\"1.0.0\"}";
    let event_value = near_sdk::serde_json::value::Value::from_str(event_json).unwrap();
    let raw_ibc = event_value["raw-ibc-event"].clone();
    let ibc_event: IbcEvent = near_sdk::serde_json::from_value(raw_ibc).unwrap();
    dbg!(&ibc_event);
}
