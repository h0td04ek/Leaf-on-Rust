//! Перенос `ServerboundPongPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static PONG_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("pong").unwrap())
});

/// `ServerboundPongPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPongPacket {
    id: i32,
}

impl ServerboundPongPacket {
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPongPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPongPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ServerboundPongPacket::new(buf.read_int()),
        )
    }
}

impl Packet for ServerboundPongPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        PONG_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.id);
    }
}
