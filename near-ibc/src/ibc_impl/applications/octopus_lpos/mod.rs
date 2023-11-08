use crate::{prelude::*, StorageKey};
use core::fmt::Debug;
use ibc::{
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
use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    log, serde_json,
    store::UnorderedMap,
    AccountId,
};
use octopus_lpos::{packet::consumer::ConsumerPacket, ConsumerChainId};

pub mod impls;

#[derive(BorshDeserialize, BorshSerialize, Debug)]
#[borsh(crate = "near_sdk::borsh")]
pub struct OctopusLposModule {
    pub chain_id_channel_map: UnorderedMap<ConsumerChainId, ChannelId>,
    pub appchain_registry_account: AccountId,
}

impl OctopusLposModule {
    pub fn new(appchain_registry_account: AccountId) -> Self {
        Self {
            chain_id_channel_map: UnorderedMap::new(StorageKey::ChainIdChannelMap),
            appchain_registry_account,
        }
    }
}

impl Module for OctopusLposModule {
    fn on_chan_open_init_validate(
        &self,
        order: Order,
        connection_hops: &[ConnectionId],
        port_id: &PortId,
        channel_id: &ChannelId,
        counterparty: &Counterparty,
        version: &Version,
    ) -> Result<Version, ChannelError> {
        octopus_lpos::context::on_chan_open_init_validate(
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
        octopus_lpos::context::on_chan_open_try_validate(
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
        octopus_lpos::context::on_chan_open_ack_validate(
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
        octopus_lpos::context::on_chan_open_confirm_validate(self, port_id, channel_id).map_err(
            |e| ChannelError::AppModule {
                description: e.to_string(),
            },
        )
    }

    fn on_chan_close_init_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        octopus_lpos::context::on_chan_close_init_validate(self, port_id, channel_id).map_err(|e| {
            ChannelError::AppModule {
                description: e.to_string(),
            }
        })
    }

    fn on_chan_close_confirm_validate(
        &self,
        port_id: &PortId,
        channel_id: &ChannelId,
    ) -> Result<(), ChannelError> {
        octopus_lpos::context::on_chan_close_confirm_validate(self, port_id, channel_id).map_err(
            |e| ChannelError::AppModule {
                description: e.to_string(),
            },
        )
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
        let consumer_packet =
            serde_json::from_slice::<ConsumerPacket>(&packet.data).expect("Invalid packet data");
        let maybe_consumer_packet = Packet {
            data: serde_json::to_string(&consumer_packet)
                .expect("Invalid packet data")
                .into_bytes(),
            ..packet.clone()
        };
        let (extras, ack) =
            octopus_lpos::context::on_recv_packet_execute(self, &maybe_consumer_packet);
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
        octopus_lpos::context::on_acknowledgement_packet_validate(
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
        octopus_lpos::context::on_timeout_packet_validate(self, packet, relayer).map_err(|e| {
            PacketError::AppModule {
                description: e.to_string(),
            }
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
        octopus_lpos::context::on_chan_open_init_execute(
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
        octopus_lpos::context::on_chan_open_try_execute(
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
        let result = octopus_lpos::context::on_acknowledgement_packet_execute(
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
        let result = octopus_lpos::context::on_timeout_packet_execute(self, packet, relayer);
        (
            result.0,
            result.1.map_err(|e| PacketError::AppModule {
                description: e.to_string(),
            }),
        )
    }
}
