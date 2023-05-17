#[macro_export]
macro_rules! impl_storage_check_and_refund {
    ($contract: ident) => {
        use $crate::CheckStorageAndRefund;

        #[near_bindgen]
        impl CheckStorageAndRefund for $contract {
            fn check_storage_and_refund(
                &mut self,
                caller: AccountId,
                max_refundable_amount: U128,
                previously_used_bytes: U64,
            ) {
                assert_self();
                let mut refund_amount = max_refundable_amount.0;
                if env::storage_usage() > previously_used_bytes.0 {
                    log!(
                        "Storage usage in previously function call: {}",
                        env::storage_usage() - previously_used_bytes.0
                    );
                    let cost = env::storage_byte_cost()
                        * (env::storage_usage() - previously_used_bytes.0) as u128;
                    if cost >= refund_amount {
                        return;
                    } else {
                        refund_amount -= cost;
                    }
                }
                Promise::new(caller).transfer(refund_amount);
            }
        }
    };
}
