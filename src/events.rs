use ibc::events::IbcEvent;
use near_sdk::log;
use near_sdk::serde::Serialize;
use near_sdk::serde_json::{json, Value};

const EVENT_STANDARD: &str = "near-ibc";
const EVENT_STANDARD_VERSION: &str = "1.0.0";

pub trait EventEmit {
    fn emit(&self)
    where
        Self: Sized + Serialize;
}

pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let mut result = json!(data);
    let map = result.as_object_mut().unwrap();
    map.insert(
        "standard".to_string(),
        Value::String(EVENT_STANDARD.to_string()),
    );
    map.insert(
        "version".to_string(),
        Value::String(EVENT_STANDARD_VERSION.to_string()),
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
