//! Перенос `ServerboundCookieResponsePacket`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

static COOKIE_RESPONSE_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("cookie_response").unwrap())
});

/// `ServerboundCookieResponsePacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundCookieResponsePacket {
    pub key: Identifier,
    /// `payload` — до 5120 байт (`ClientboundStoreCookiePacket.PAYLOAD_STREAM_CODEC`).
    pub payload: Option<Vec<u8>>,
}

impl ServerboundCookieResponsePacket {
    pub fn new(key: Identifier, payload: Option<Vec<u8>>) -> Self {
        Self { key, payload }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, ServerboundCookieResponsePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundCookieResponsePacket| {
                p.write_packet(buf);
            },
            |buf: &mut FriendlyByteBuf| {
                let key = buf.read_identifier().expect("invalid identifier");
                let payload = read_nullable_payload(buf);
                ServerboundCookieResponsePacket::new(key, payload)
            },
        )
    }
}

fn read_nullable_payload(buf: &mut FriendlyByteBuf) -> Option<Vec<u8>> {
    let present = buf.read_byte() != 0;
    if !present {
        return None;
    }
    Some(byte_buf_codecs::byte_array(5120).decode(buf))
}

fn write_nullable_payload(buf: &mut FriendlyByteBuf, payload: &Option<Vec<u8>>) {
    match payload {
        None => {
            buf.write_byte(0);
        }
        Some(bytes) => {
            buf.write_byte(1);
            byte_buf_codecs::byte_array(5120).encode(buf, bytes);
        }
    }
}

impl Packet for ServerboundCookieResponsePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        COOKIE_RESPONSE_TYPE.clone()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.key.to_string()).expect("identifier too long");
        write_nullable_payload(buf, &self.payload);
    }
}
