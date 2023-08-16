use crate::{context::NearIbcStoreHost, prelude::*};
use core::{fmt::Debug, str::FromStr};
use ibc::{
    applications::transfer::packet::PacketData,
    core::{
        ics04_channel::{
            acknowledgement::Acknowledgement,
            channel::{Counterparty, Order},
            error::{ChannelError, PacketError},
            packet::Packet,
            Version,
        },
        ics24_host::identifier::{ChannelId, ConnectionId, PortId},
        router::{Module, ModuleExtras},
    },
    Signer,
};
use ibc_proto::ibc::apps::transfer::v2::FungibleTokenPacketData;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    log,
    serde::{Deserialize, Serialize},
    serde_json, AccountId,
};

pub mod impls;

pub struct AccountIdConversion(AccountId);

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
        _relayer: &Signer,
    ) -> (ModuleExtras, Acknowledgement) {
        log!(
            "Received packet: {:?}",
            String::from_utf8(packet.data.to_vec()).expect("Invalid packet data")
        );
        let ft_packet_data =
            serde_json::from_slice::<FtPacketData>(&packet.data).expect("Invalid packet data");
        let maybe_ft_packet = Packet {
            data: serde_json::to_string(
                &PacketData::try_from(FungibleTokenPacketData::from(ft_packet_data))
                    .expect("Invalid packet data"),
            )
            .expect("Invalid packet data")
            .into_bytes(),
            ..packet.clone()
        };
        let (extras, ack) =
            ibc::applications::transfer::context::on_recv_packet_execute(self, &maybe_ft_packet);
        let ack_status =
            String::from_utf8(ack.as_bytes().to_vec()).expect("Invalid acknowledgement string");
        log!("Packet acknowledgement: {}", ack_status);
        (extras, ack)
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
        ibc::applications::transfer::context::on_chan_open_init_execute(
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
        })
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
        ibc::applications::transfer::context::on_chan_open_try_execute(
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
        })
    }

    fn on_acknowledgement_packet_execute(
        &mut self,
        packet: &Packet,
        acknowledgement: &Acknowledgement,
        relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        let result = ibc::applications::transfer::context::on_acknowledgement_packet_execute(
            self,
            packet,
            acknowledgement,
            relayer,
        );
        (
            result.0,
            result.1.map_err(|e| PacketError::AppModule {
                description: e.to_string(),
            }),
        )
    }

    fn on_timeout_packet_execute(
        &mut self,
        packet: &Packet,
        relayer: &Signer,
    ) -> (ModuleExtras, Result<(), PacketError>) {
        let result =
            ibc::applications::transfer::context::on_timeout_packet_execute(self, packet, relayer);
        (
            result.0,
            result.1.map_err(|e| PacketError::AppModule {
                description: e.to_string(),
            }),
        )
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct FtPacketData {
    /// the token denomination to be transferred
    pub denom: String,
    /// the token amount to be transferred
    pub amount: String,
    /// the sender address
    pub sender: String,
    /// the recipient address on the destination chain
    pub receiver: String,
    /// optional memo
    pub memo: String,
}

impl From<FtPacketData> for FungibleTokenPacketData {
    fn from(value: FtPacketData) -> Self {
        FungibleTokenPacketData {
            denom: value.denom,
            amount: value.amount,
            sender: value.sender,
            receiver: value.receiver,
            memo: value.memo,
        }
    }
}
