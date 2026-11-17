//! Перенос `ServerboundPingRequestPacket`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static PING_REQUEST_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("ping_request").unwrap())
});

/// `ServerboundPingRequestPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPingRequestPacket {
    time: i64,
}

impl ServerboundPingRequestPacket {
    pub fn new(time: i64) -> Self {
        Self { time }
    }

    /// `getTime()`.
    pub fn get_time(&self) -> i64 {
        self.time
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, ServerboundPingRequestPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPingRequestPacket| {
                buf.write_long(p.time);
            },
            |buf: &mut FriendlyByteBuf| ServerboundPingRequestPacket::new(buf.read_long()),
        )
    }
}

impl Packet for ServerboundPingRequestPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        PING_REQUEST_TYPE.clone()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_long(self.time);
    }
}
