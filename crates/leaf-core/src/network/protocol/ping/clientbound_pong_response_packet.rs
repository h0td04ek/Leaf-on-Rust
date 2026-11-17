//! Перенос `ClientboundPongResponsePacket`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::network::protocol::ping::ping_packet_types;
use crate::network::protocol::packet::PacketTypePtr;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static PONG_RESPONSE_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("pong_response").unwrap())
});

/// `ClientboundPongResponsePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundPongResponsePacket {
    pub time: i64,
}

impl ClientboundPongResponsePacket {
    pub fn new(time: i64) -> Self {
        Self { time }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, ClientboundPongResponsePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPongResponsePacket| {
                buf.write_long(p.time);
            },
            |buf: &mut FriendlyByteBuf| ClientboundPongResponsePacket::new(buf.read_long()),
        )
    }
}

impl Packet for ClientboundPongResponsePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        PONG_RESPONSE_TYPE.clone()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_long(self.time);
    }
}
