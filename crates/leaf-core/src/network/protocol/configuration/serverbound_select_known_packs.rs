//! Перенос `ServerboundSelectKnownPacks` — list max 64.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::server::packs::repository::known_pack::KnownPack;
use std::sync::LazyLock;

static SERVERBOUND_SELECT_KNOWN_PACKS_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::SERVERBOUND,
        Identifier::with_default_namespace("select_known_packs").unwrap(),
    )
});

/// `ServerboundSelectKnownPacks`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSelectKnownPacks {
    /// `knownPacks`.
    pub known_packs: Vec<KnownPack>,
}

impl ServerboundSelectKnownPacks {
    pub fn new(known_packs: Vec<KnownPack>) -> Self {
        Self { known_packs }
    }

    /// `STREAM_CODEC` — `KnownPack.STREAM_CODEC.list(64)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSelectKnownPacks> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSelectKnownPacks| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let list = byte_buf_codecs::list(KnownPack::stream_codec());
                ServerboundSelectKnownPacks::new(list.decode(buf))
            },
        )
    }
}

impl Packet for ServerboundSelectKnownPacks {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        SERVERBOUND_SELECT_KNOWN_PACKS_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        byte_buf_codecs::list(KnownPack::stream_codec()).encode(buf, &self.known_packs);
    }
}
