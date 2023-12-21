use crate::*;

pub trait SudoFunctions {
    ///
    fn cancel_transfer_request_in_channel_escrow(
        &mut self,
        channel_id: String,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    );
}

#[near_bindgen]
impl SudoFunctions for NearIbcContract {
    ///
    fn cancel_transfer_request_in_channel_escrow(
        &mut self,
        channel_id: String,
        trace_path: String,
        base_denom: String,
        sender_id: AccountId,
        amount: U128,
    ) {
        self.assert_governance();
        let channel_escrow_id =
            format!("{}.{}", channel_id, utils::get_escrow_factory_contract_id());
        ext_process_transfer_request_callback::ext(
            AccountId::from_str(channel_escrow_id.as_str()).unwrap(),
        )
        .with_attached_deposit(NearToken::from_yoctonear(0))
        .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(4))
        .with_unused_gas_weight(0)
        .cancel_transfer_request(trace_path, base_denom, sender_id, amount);
    }
}
