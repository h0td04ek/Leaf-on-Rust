//! Перенос `ClientboundCookieRequestPacket`.

use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

const MAX_KEY_LENGTH: usize = 256;

static COOKIE_REQUEST_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("cookie_request").unwrap())
});

/// `ClientboundCookieRequestPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundCookieRequestPacket {
    pub key: Identifier,
}

impl ClientboundCookieRequestPacket {
    pub fn new(key: Identifier) -> Self {
        Self { key }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl crate::network::codec::stream_codec::StreamCodec<FriendlyByteBuf, ClientboundCookieRequestPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCookieRequestPacket| {
                p.write_packet(buf);
            },
            |buf: &mut FriendlyByteBuf| {
                let key = buf.read_identifier().expect("invalid identifier");
                ClientboundCookieRequestPacket::new(key)
            },
        )
    }
}

impl Packet for ClientboundCookieRequestPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        COOKIE_REQUEST_TYPE.clone()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        // FriendlyByteBuf.writeIdentifier не ограничивает длину;
        // Java пишет Identifier как utf (max 32767 через ResourceLocation codec).
        buf.write_utf(&self.key.to_string()).expect("identifier too long");
    }
}
