//! Перенос `ClientboundStoreCookiePacket`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

/// `MAX_PAYLOAD_SIZE`.
pub const MAX_PAYLOAD_SIZE: usize = 5120;

static STORE_COOKIE_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("store_cookie").unwrap())
});

/// `ClientboundStoreCookiePacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundStoreCookiePacket {
    pub key: Identifier,
    /// `payload` — до 5120 байт.
    pub payload: Vec<u8>,
}

impl ClientboundStoreCookiePacket {
    pub fn new(key: Identifier, payload: Vec<u8>) -> Self {
        Self { key, payload }
    }

    /// `PAYLOAD_STREAM_CODEC` (`ByteBufCodecs.byteArray(5120)`).
    pub fn payload_stream_codec() -> byte_buf_codecs::ByteArrayCodec {
        byte_buf_codecs::byte_array(MAX_PAYLOAD_SIZE)
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundStoreCookiePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundStoreCookiePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let key = buf.read_identifier().expect("invalid identifier");
                let payload = Self::payload_stream_codec().decode(buf);
                ClientboundStoreCookiePacket::new(key, payload)
            },
        )
    }
}

impl Packet for ClientboundStoreCookiePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        STORE_COOKIE_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_identifier(&self.key).expect("identifier");
        Self::payload_stream_codec().encode(buf, &self.payload);
    }
}
