use crate::*;
use ibc::core::ics04_channel::packet::Receipt as IbcReceipt;
use ibc::mock::client_state::MockClientState;
use near_sdk::json_types::U64;

// pub struct Near
#[derive(Clone, Debug, Serialize)]
#[serde(crate = "near_sdk::serde")]
pub enum Receipt {
    Ok,
}

impl From<IbcReceipt> for Receipt {
    fn from(ibc_receipt: IbcReceipt) -> Self {
        match ibc_receipt {
            IbcReceipt::Ok => Receipt::Ok,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Height {
    /// Previously known as "epoch"
    pub revision_number: U64,

    /// The height of a block
    pub revision_height: U64,
}

impl From<Height> for ibc::core::ics02_client::height::Height {
    fn from(height: Height) -> Self {
        ibc::core::ics02_client::height::Height::new(
            height.revision_number.0,
            height.revision_height.0,
        )
        .unwrap()
    }
}
