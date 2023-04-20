use core::fmt::Debug;

use ibc::{
    applications::transfer::{
        context::{BankKeeper, TokenTransferContext, TokenTransferKeeper, TokenTransferReader},
        error::TokenTransferError,
        PrefixedCoin, PORT_ID_STR,
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
    signer::Signer,
    Height,
};

use crate::context::NearIbcStoreHost;

use super::{AccountIdConversion, TransferModule};

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
        todo!()
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
        todo!()
    }

    fn is_send_enabled(&self) -> bool {
        todo!()
    }

    fn is_receive_enabled(&self) -> bool {
        todo!()
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
