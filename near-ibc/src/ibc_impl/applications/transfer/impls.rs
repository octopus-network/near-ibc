use super::{AccountIdConversion, TransferModule};
use crate::context::NearIbcStoreHost;
use ibc::{
    applications::transfer::{
        context::{BankKeeper, TokenTransferContext, TokenTransferKeeper, TokenTransferReader},
        error::TokenTransferError,
        PrefixedCoin,
    },
    core::{
        ics02_client::{client_state::ClientState, consensus_state::ConsensusState},
        ics03_connection::connection::ConnectionEnd,
        ics04_channel::{
            channel::ChannelEnd,
            commitment::PacketCommitment,
            context::{ChannelKeeper, ChannelReader, SendPacketReader},
            error::PacketError,
            packet::Sequence,
        },
        ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    },
    Height,
};
use near_sdk::{
    env,
    json_types::U128,
    log,
    serde::{Deserialize, Serialize},
    Promise,
};
use std::str::FromStr;

impl BankKeeper for TransferModule {
    type AccountId = AccountIdConversion;

    fn send_coins(
        &mut self,
        from: &Self::AccountId,
        to: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        todo!()
    }

    fn mint_coins(
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
        #[derive(Serialize, Deserialize, Clone)]
        #[serde(crate = "near_sdk::serde")]
        struct Input {
            pub trace_path: String,
            pub base_denom: String,
            pub token_owner: String,
            pub amount: U128,
        }
        let args = Input {
            trace_path: amt.denom.trace_path.to_string(),
            base_denom: amt.denom.base_denom.to_string(),
            token_owner: account.0.to_string(),
            amount: U128(u128::from_str(amt.amount.to_string().as_str()).unwrap()),
        };
        let args = near_sdk::serde_json::to_vec(&args).expect("ERR_SERIALIZE_ARGS_FOR_MINT_ASSET");
        Promise::new(utils::get_token_factory_contract_id()).function_call(
            "mint_asset".to_string(),
            args,
            env::attached_deposit(),
            utils::GAS_FOR_MINT_ASSET,
        );
        Ok(())
    }

    fn burn_coins(
        &mut self,
        account: &Self::AccountId,
        amt: &PrefixedCoin,
    ) -> Result<(), TokenTransferError> {
        todo!()
    }
}

impl TokenTransferReader for TransferModule {
    type AccountId = <Self as TokenTransferContext>::AccountId;

    fn get_port(&self) -> Result<PortId, TokenTransferError> {
        Ok(PortId::transfer())
    }

    fn get_channel_escrow_address(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<<Self as TokenTransferReader>::AccountId, TokenTransferError> {
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

    fn is_send_enabled(&self) -> bool {
        // TODO: check if this is correct
        true
    }

    fn is_receive_enabled(&self) -> bool {
        // TODO: check if this is correct
        true
    }
}

impl SendPacketReader for TransferModule {
    fn channel_end(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<ChannelEnd, PacketError> {
        ChannelReader::channel_end(&Self::get_near_ibc_store(), port_id, channel_id)
            .map_err(|err| PacketError::Channel(err))
    }

    fn connection_end(&self, connection_id: &ConnectionId) -> Result<ConnectionEnd, PacketError> {
        ChannelReader::connection_end(&Self::get_near_ibc_store(), connection_id)
            .map_err(|err| PacketError::Channel(err))
    }

    fn client_state(&self, client_id: &ClientId) -> Result<Box<dyn ClientState>, PacketError> {
        ChannelReader::client_state(&Self::get_near_ibc_store(), client_id)
            .map_err(|err| PacketError::Channel(err))
    }

    fn client_consensus_state(
        &self,
        client_id: &ClientId,
        height: &Height,
    ) -> Result<Box<dyn ConsensusState>, PacketError> {
        ChannelReader::client_consensus_state(&Self::get_near_ibc_store(), client_id, height)
            .map_err(|err| PacketError::Channel(err))
    }

    fn get_next_sequence_send(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<Sequence, PacketError> {
        ChannelReader::get_next_sequence_send(&Self::get_near_ibc_store(), port_id, channel_id)
    }

    fn hash(&self, value: &[u8]) -> Vec<u8> {
        ChannelReader::hash(&Self::get_near_ibc_store(), value)
    }
}

impl TokenTransferKeeper for TransferModule {
    fn store_packet_commitment(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        sequence: Sequence,
        commitment: PacketCommitment,
    ) -> Result<(), PacketError> {
        let mut store = Self::get_near_ibc_store();
        let result = store.store_packet_commitment(port_id, channel_id, sequence, commitment);
        Self::set_near_ibc_store(&store);
        result
    }

    fn store_next_sequence_send(
        &mut self,
        port_id: PortId,
        channel_id: ChannelId,
        seq: Sequence,
    ) -> Result<(), PacketError> {
        let mut store = Self::get_near_ibc_store();
        let result = store.store_next_sequence_send(port_id, channel_id, seq);
        Self::set_near_ibc_store(&store);
        result
    }
}

impl TokenTransferContext for TransferModule {
    type AccountId = AccountIdConversion;
}
