use super::{AccountIdConversion, TransferModule};
use crate::prelude::*;
use core::str::FromStr;
use ibc::{
    applications::transfer::{
        context::{TokenTransferExecutionContext, TokenTransferValidationContext},
        error::TokenTransferError,
        PrefixedCoin,
    },
    core::ics24_host::identifier::{ChannelId, PortId},
};
use near_sdk::{env, json_types::U128, log};
use utils::{
    interfaces::{ext_channel_escrow, ext_process_transfer_request_callback, ext_token_factory},
    ExtraDepositCost,
};

impl TokenTransferExecutionContext for TransferModule {
    fn send_coins_execute(
        &mut self,
        from: &Self::AccountId,
        to: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        let sender_id = from.0.to_string();
        let receiver_id = to.0.to_string();
        let trace_path = amt.denom.trace_path.to_string();
        let base_denom = amt.denom.base_denom.to_string();
        let prefixed_ef = format!(".ef.transfer.{}", env::current_account_id());
        if receiver_id.ends_with(prefixed_ef.as_str()) {
            ext_process_transfer_request_callback::ext(to.0.clone())
                .with_attached_deposit(0)
                .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                .with_unused_gas_weight(0)
                .apply_transfer_request(
                    trace_path,
                    base_denom,
                    from.0.clone(),
                    U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
                );
        } else if sender_id.ends_with(prefixed_ef.as_str()) {
            ext_channel_escrow::ext(from.0.clone())
                .with_attached_deposit(1)
                .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(4))
                .with_unused_gas_weight(0)
                .do_transfer(
                    trace_path,
                    base_denom,
                    to.0.clone(),
                    U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
                );
            ExtraDepositCost::add(1);
        } else {
            panic!("Neither sender nor receiver is an escrow account. This should not happen.");
        }
        Ok(())
    }

    fn mint_coins_execute(
        &mut self,
        account: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        log!(
            "Minting coins for account {}, trace path {}, base denom {}",
            account.0,
            amt.denom.trace_path,
            amt.denom.base_denom
        );
        ext_token_factory::ext(utils::get_token_factory_contract_id())
            .with_attached_deposit(utils::STORAGE_DEPOSIT_FOR_MINT_TOKEN)
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL.saturating_mul(8))
            .with_unused_gas_weight(0)
            .mint_asset(
                amt.denom.trace_path.to_string(),
                amt.denom.base_denom.to_string(),
                account.0.clone(),
                U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
            );
        ExtraDepositCost::add(utils::STORAGE_DEPOSIT_FOR_MINT_TOKEN);
        Ok(())
    }

    fn burn_coins_execute(
        &mut self,
        account: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        log!(
            "Burning coins for account {}, trace path {}, base denom {}",
            account.0,
            amt.denom.trace_path,
            amt.denom.base_denom
        );
        ext_process_transfer_request_callback::ext(env::predecessor_account_id())
            .with_attached_deposit(0)
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
            .with_unused_gas_weight(0)
            .apply_transfer_request(
                amt.denom.trace_path.to_string(),
                amt.denom.base_denom.to_string(),
                account.0.clone(),
                U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
            );
        Ok(())
    }
}

impl TokenTransferValidationContext for TransferModule {
    type AccountId = AccountIdConversion;

    fn get_port(&self) -> Result<PortId, TokenTransferError> {
        Ok(PortId::transfer())
    }

    fn get_escrow_account(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Self::AccountId, TokenTransferError> {
        let escrow_account = format!(
            "{}.ef.{}.{}",
            channel_id.as_str(),
            port_id.as_str(),
            env::current_account_id()
        );
        Ok(AccountIdConversion(
            near_sdk::AccountId::from_str(escrow_account.as_str()).unwrap(),
        ))
    }

    fn can_send_coins(&self) -> Result<(), TokenTransferError> {
        // TODO: check if this is correct
        Ok(())
    }

    fn can_receive_coins(&self) -> Result<(), TokenTransferError> {
        // TODO: check if this is correct
        Ok(())
    }

    fn send_coins_validate(
        &self,
        _from_account: &Self::AccountId,
        _to_account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn mint_coins_validate(
        &self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }

    fn burn_coins_validate(
        &self,
        _account: &Self::AccountId,
        _coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        Ok(())
    }
}
