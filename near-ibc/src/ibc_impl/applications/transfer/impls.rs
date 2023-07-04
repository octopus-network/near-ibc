use super::{AccountIdConversion, TransferModule};
use crate::{context::NearIbcStoreHost, prelude::*};
use core::str::FromStr;
use ibc::{
    applications::transfer::{
        context::{TokenTransferExecutionContext, TokenTransferValidationContext},
        error::TokenTransferError,
        PrefixedCoin,
    },
    core::{
        ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
        ics03_connection::connection::ConnectionEnd,
        ics04_channel::{
            channel::ChannelEnd,
            commitment::PacketCommitment,
            context::{SendPacketExecutionContext, SendPacketValidationContext},
            packet::Sequence,
        },
        ics24_host::{
            identifier::{ChannelId, ClientId, ConnectionId, PortId},
            path::{ChannelEndPath, ClientConsensusStatePath, CommitmentPath, SeqSendPath},
        },
        ContextError,
    },
};
use near_sdk::{env, json_types::U128, log};
use utils::interfaces::{
    ext_channel_escrow, ext_process_transfer_request_callback, ext_token_factory,
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
        let base_denom = amt.denom.base_denom.to_string();
        if receiver_id.ends_with(env::current_account_id().as_str()) {
            ext_process_transfer_request_callback::ext(to.0.clone())
                .with_attached_deposit(0)
                .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                .with_unused_gas_weight(0)
                .apply_transfer_request(
                    base_denom,
                    from.0.clone(),
                    U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
                );
        } else if sender_id.ends_with(env::current_account_id().as_str()) {
            ext_channel_escrow::ext(from.0.clone())
                .with_attached_deposit(1)
                .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL)
                .with_unused_gas_weight(0)
                .do_transfer(
                    base_denom,
                    to.0.clone(),
                    U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
                );
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
            .with_attached_deposit(env::attached_deposit())
            .with_static_gas(utils::GAS_FOR_SIMPLE_FUNCTION_CALL * 8)
            .with_unused_gas_weight(0)
            .mint_asset(
                amt.denom.trace_path.to_string(),
                amt.denom.base_denom.to_string(),
                account.0.clone(),
                U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
            );
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
        from_account: &Self::AccountId,
        to_account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        todo!()
    }

    fn mint_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        todo!()
    }

    fn burn_coins_validate(
        &self,
        account: &Self::AccountId,
        coin: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        todo!()
    }
}

impl SendPacketValidationContext for TransferModule {
    fn channel_end(&self, channel_end_path: &ChannelEndPath) -> Result<ChannelEnd, ContextError> {
        todo!()
    }

    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, ContextError> {
        todo!()
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, ContextError> {
        todo!()
    }

    fn client_consensus_state(
        &self,
        client_cons_state_path: &ClientConsensusStatePath,
    ) -> Result<Box<dyn ConsensusState>, ContextError> {
        todo!()
    }

    fn get_next_sequence_send(
        &self,
        seq_send_path: &SeqSendPath,
    ) -> Result<Sequence, ContextError> {
        todo!()
    }
}

impl SendPacketExecutionContext for TransferModule {
    fn store_packet_commitment(
        &mut self,
        commitment_path: &CommitmentPath,
        commitment: PacketCommitment,
    ) -> Result<(), ContextError> {
        let mut store = Self::get_near_ibc_store();
        let result = store.store_packet_commitment(commitment_path, commitment);
        Self::set_near_ibc_store(&store);
        result
    }

    fn store_next_sequence_send(
        &mut self,
        seq_send_path: &SeqSendPath,
        seq: Sequence,
    ) -> Result<(), ContextError> {
        let mut store = Self::get_near_ibc_store();
        let result = store.store_next_sequence_send(seq_send_path, seq);
        Self::set_near_ibc_store(&store);
        result
    }

    fn emit_ibc_event(&mut self, event: ibc::core::events::IbcEvent) {
        todo!()
    }

    fn log_message(&mut self, message: String) {
        todo!()
    }
}
