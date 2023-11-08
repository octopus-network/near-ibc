use crate::*;

#[near_bindgen]
impl TransferRequestHandler for NearIbcContract {
    //
    fn process_transfer_request(&mut self, transfer_request: Ics20TransferRequest) {
        utils::assert_sub_account();
        let mut near_ibc_store = self.near_ibc_store.get().unwrap();
        let timeout_seconds = transfer_request
            .timeout_seconds
            .map_or_else(|| DEFAULT_TIMEOUT_SECONDS, |value| value.0);
        if let Err(e) = ibc::applications::transfer::send_transfer(
            &mut near_ibc_store,
            &mut TransferModule(),
            MsgTransfer {
                port_id_on_a: PortId::from_str(transfer_request.port_on_a.as_str()).unwrap(),
                chan_id_on_a: ChannelId::from_str(transfer_request.chan_on_a.as_str()).unwrap(),
                packet_data: PacketData {
                    token: PrefixedCoin {
                        denom: PrefixedDenom {
                            trace_path: TracePath::from_str(
                                transfer_request.token_trace_path.as_str(),
                            )
                            .unwrap(),
                            base_denom: BaseDenom::from_str(transfer_request.token_denom.as_str())
                                .unwrap(),
                        },
                        amount: Amount::from_str(transfer_request.amount.0.to_string().as_str())
                            .unwrap(),
                    },
                    sender: Signer::from(transfer_request.sender.clone()),
                    receiver: Signer::from(transfer_request.receiver.clone()),
                    memo: Memo::from_str("").unwrap(),
                },
                timeout_height_on_b: TimeoutHeight::Never {},
                timeout_timestamp_on_b: Timestamp::from_nanoseconds(
                    env::block_timestamp() + timeout_seconds * 1000000000,
                )
                .unwrap(),
            },
        ) {
            log!("ERR_SEND_TRANSFER: {:?}", e);
            log!(
                "Cancelling transfer request for account {}, trace path {}, base denom {} with amount {}",
                transfer_request.sender,
                transfer_request.token_trace_path,
                transfer_request.token_denom,
                transfer_request.amount.0
            );
            ext_process_transfer_request_callback::ext(env::predecessor_account_id())
                .with_attached_deposit(0)
                .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(4))
                .with_unused_gas_weight(0)
                .cancel_transfer_request(
                    transfer_request.token_trace_path,
                    transfer_request.token_denom,
                    AccountId::from_str(transfer_request.sender.as_str()).unwrap(),
                    transfer_request.amount,
                );
        }
        self.near_ibc_store.set(&near_ibc_store);
    }
}
