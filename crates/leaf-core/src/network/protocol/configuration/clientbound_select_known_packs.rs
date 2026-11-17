//! Перенос `ClientboundSelectKnownPacks`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::server::packs::repository::known_pack::KnownPack;
use std::sync::LazyLock;

static SELECT_KNOWN_PACKS_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("select_known_packs").unwrap(),
    )
});

/// `ClientboundSelectKnownPacks`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundSelectKnownPacks {
    /// `knownPacks`.
    pub known_packs: Vec<KnownPack>,
}

impl ClientboundSelectKnownPacks {
    pub fn new(known_packs: Vec<KnownPack>) -> Self {
        Self { known_packs }
    }

    /// `STREAM_CODEC` — `KnownPack.STREAM_CODEC.list()`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSelectKnownPacks> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSelectKnownPacks| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let list = byte_buf_codecs::list(KnownPack::stream_codec());
                ClientboundSelectKnownPacks::new(list.decode(buf))
            },
        )
    }
}

impl Packet for ClientboundSelectKnownPacks {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        SELECT_KNOWN_PACKS_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        byte_buf_codecs::list(KnownPack::stream_codec()).encode(buf, &self.known_packs);
    }
}
