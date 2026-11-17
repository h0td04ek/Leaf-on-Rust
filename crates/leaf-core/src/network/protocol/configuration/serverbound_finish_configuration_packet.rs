//! Перенос `ServerboundFinishConfigurationPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static SERVERBOUND_FINISH_CONFIGURATION_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::SERVERBOUND,
        Identifier::with_default_namespace("finish_configuration").unwrap(),
    )
});

/// `ServerboundFinishConfigurationPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundFinishConfigurationPacket;

/// `INSTANCE`.
pub const SERVERBOUND_FINISH_CONFIGURATION_PACKET: ServerboundFinishConfigurationPacket =
    ServerboundFinishConfigurationPacket;

impl ServerboundFinishConfigurationPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundFinishConfigurationPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ServerboundFinishConfigurationPacket| {},
            |_buf: &mut FriendlyByteBuf| ServerboundFinishConfigurationPacket,
        )
    }
}

impl Packet for ServerboundFinishConfigurationPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        SERVERBOUND_FINISH_CONFIGURATION_TYPE.clone()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}

    fn is_terminal(&self) -> bool {
        true
    }
}
