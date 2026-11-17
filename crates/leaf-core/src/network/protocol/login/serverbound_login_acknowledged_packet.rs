//! Перенос `ServerboundLoginAcknowledgedPacket` — singleton, пустой пакет.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static LOGIN_ACK_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::SERVERBOUND,
        Identifier::with_default_namespace("login_acknowledged").unwrap(),
    )
});

/// `ServerboundLoginAcknowledgedPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundLoginAcknowledgedPacket;

/// `INSTANCE`.
pub const SERVERBOUND_LOGIN_ACKNOWLEDGED_PACKET: ServerboundLoginAcknowledgedPacket =
    ServerboundLoginAcknowledgedPacket;

impl ServerboundLoginAcknowledgedPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundLoginAcknowledgedPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ServerboundLoginAcknowledgedPacket| {},
            |_buf: &mut FriendlyByteBuf| ServerboundLoginAcknowledgedPacket,
        )
    }
}

impl Packet for ServerboundLoginAcknowledgedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        LOGIN_ACK_TYPE.clone()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}

    fn is_terminal(&self) -> bool {
        true
    }
}
