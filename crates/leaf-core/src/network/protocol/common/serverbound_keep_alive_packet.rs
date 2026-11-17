//! Перенос `ServerboundKeepAlivePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static KEEP_ALIVE_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("keep_alive").unwrap())
});

/// `ServerboundKeepAlivePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundKeepAlivePacket {
    id: i64,
}

impl ServerboundKeepAlivePacket {
    pub fn new(id: i64) -> Self {
        Self { id }
    }

    /// `getId`.
    pub fn get_id(&self) -> i64 {
        self.id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundKeepAlivePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundKeepAlivePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ServerboundKeepAlivePacket::new(buf.read_long()),
        )
    }
}

impl Packet for ServerboundKeepAlivePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        KEEP_ALIVE_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_long(self.id);
    }
}
