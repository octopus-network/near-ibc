use crate::context::NearIbcStoreHost;
use core::{fmt::Debug, str::FromStr};
use ibc::{
    core::{
        ics04_channel::{
            channel::{Counterparty, Order},
            error::{ChannelError, PacketError},
            packet::Acknowledgement,
            packet::Packet,
            Version,
        },
        ics24_host::identifier::{ChannelId, ConnectionId, PortId},
        router::{Module, ModuleExtras},
    },
    Signer,
};
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    AccountId,
};

pub mod impls;

pub struct AccountIdConversion(near_sdk::AccountId);

#[derive(BorshDeserialize, BorshSerialize, Debug)]
pub struct TransferModule();

impl NearIbcStoreHost for TransferModule {}

impl Module for TransferModule {
    fn on_chan_open_init_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<Version, ChannelError> {
        ibc::applications::transfer::context::on_chan_open_init_validate(
            self,
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
            version,
        )
        .map_err(|e| ChannelError::AppModule {
            description: e.to_string(),
        })?;
        Ok(version.clone())
    }

    fn on_chan_open_try_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<Version, ChannelError> {
        ibc::applications::transfer::context::on_chan_open_try_validate(
            self,
            order,
            connection_hops,
            port_id,
            channel_id,
            counterparty,
            counterparty_version,
        )
        .map_err(|e| ChannelError::AppModule {
            description: e.to_string(),
        })?;
        Ok(counterparty_version.clone())
    }

    fn on_chan_open_ack_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty_version: &Version,
    ) -> Result<(), ChannelError> {
        ibc::applications::transfer::context::on_chan_open_ack_validate(
            self,
            port_id,
            channel_id,
            counterparty_version,
        )
        .map_err(|e| ChannelError::AppModule {
            description: e.to_string(),
        })
    }

    fn on_chan_open_confirm_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        // Create and initialize the escrow sub-account for this channel.

        // Call default implementation.
        ibc::applications::transfer::context::on_chan_open_confirm_validate(
            self, port_id, channel_id,
        )
        .map_err(|e| ChannelError::AppModule {
            description: e.to_string(),
        })
    }

    fn on_chan_close_init_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        ibc::applications::transfer::context::on_chan_close_init_validate(self, port_id, channel_id)
            .map_err(|e| ChannelError::AppModule {
                description: e.to_string(),
            })
    }

    fn on_chan_close_confirm_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        ibc::applications::transfer::context::on_chan_close_confirm_validate(
            self, port_id, channel_id,
        )
        .map_err(|e| ChannelError::AppModule {
            description: e.to_string(),
        })
    }

    fn on_recv_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Acknowledgement) {
        ibc::applications::transfer::context::on_recv_packet_execute(self, packet)
    }

    fn on_acknowledgement_packet_validate(
        &self,
        packet: &Packet,
        acknowledgement: &Acknowledgement,
        relayer: &Signer,
    ) -> Result<(), PacketError> {
        ibc::applications::transfer::context::on_acknowledgement_packet_validate(
            self,
            packet,
            acknowledgement,
            relayer,
        )
        .map_err(|e| PacketError::AppModule {
            description: e.to_string(),
        })
    }

    fn on_timeout_packet_validate(
        &self,
        packet: &Packet,
        relayer: &Signer,
    ) -> Result<(), PacketError> {
        ibc::applications::transfer::context::on_timeout_packet_validate(self, packet, relayer)
            .map_err(|e| PacketError::AppModule {
                description: e.to_string(),
            })
    }

    fn on_chan_open_init_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError> {
        todo!()
    }

    fn on_chan_open_try_execute(
        &mut self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        counterparty_version: &Version,
    ) -> Result<(ModuleExtras, Version), ChannelError> {
        todo!()
    }

    fn on_acknowledgement_packet_execute(
        &mut self,
        _packet: &Packet,
        _acknowledgement: &Acknowledgement,
        _relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        todo!()
    }

    fn on_timeout_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        todo!()
    }
}

impl TryFrom<Signer> for AccountIdConversion {
    type Error = &'static str;

    fn try_from(value: Signer) -> Result<Self, Self::Error> {
        Ok(AccountIdConversion(
            AccountId::from_str(value.as_ref()).map_err(|_| "invalid signer")?,
        ))
    }
}
