//! Перенос `ClientboundLoginCompressionPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static LOGIN_COMPRESSION_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("login_compression").unwrap(),
    )
});

/// `ClientboundLoginCompressionPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundLoginCompressionPacket {
    /// `compressionThreshold`.
    pub compression_threshold: i32,
}

impl ClientboundLoginCompressionPacket {
    pub fn new(compression_threshold: i32) -> Self {
        Self { compression_threshold }
    }

    /// `getCompressionThreshold`.
    pub fn get_compression_threshold(&self) -> i32 {
        self.compression_threshold
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLoginCompressionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundLoginCompressionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundLoginCompressionPacket::new(buf.read_var_int())
            },
        )
    }
}

impl Packet for ClientboundLoginCompressionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        LOGIN_COMPRESSION_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.compression_threshold);
    }
}
