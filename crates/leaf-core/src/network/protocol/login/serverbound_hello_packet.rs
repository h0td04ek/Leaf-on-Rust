//! Перенос `ServerboundHelloPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use std::sync::LazyLock;

/// `MAX_NAME_LENGTH`.
const MAX_NAME_LENGTH: usize = 16;

static HELLO_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::SERVERBOUND, Identifier::with_default_namespace("hello").unwrap())
});

/// `ServerboundHelloPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundHelloPacket {
    /// `name` — ник игрока (≤16 символов).
    pub name: String,
    /// `profileId` — uuid аккаунта.
    pub profile_id: Uuid,
}

impl ServerboundHelloPacket {
    pub fn new(name: String, profile_id: Uuid) -> Self {
        Self { name, profile_id }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundHelloPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundHelloPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let name = buf.read_utf_n(MAX_NAME_LENGTH).expect("invalid player name");
                let profile_id = buf.read_uuid();
                ServerboundHelloPacket::new(name, profile_id)
            },
        )
    }
}

impl Packet for ServerboundHelloPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        HELLO_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf_n(&self.name, MAX_NAME_LENGTH).expect("player name too long");
        buf.write_uuid(&self.profile_id);
    }
}
