//! Перенос `ClientboundPingPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static PING_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("ping").unwrap())
});

/// `ClientboundPingPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundPingPacket {
    id: i32,
}

impl ClientboundPingPacket {
    pub fn new(id: i32) -> Self {
        Self { id }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPingPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPingPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ClientboundPingPacket::new(buf.read_int()),
        )
    }
}

impl Packet for ClientboundPingPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        PING_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.id);
    }
}
