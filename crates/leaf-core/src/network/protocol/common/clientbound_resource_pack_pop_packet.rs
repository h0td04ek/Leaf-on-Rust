//! Перенос `ClientboundResourcePackPopPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::util::uuid::Uuid;
use std::sync::LazyLock;

static RESOURCE_PACK_POP_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("resource_pack_pop").unwrap())
});

/// `ClientboundResourcePackPopPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundResourcePackPopPacket {
    /// `id` — Optional (None = удалить все пакеты).
    pub id: Option<Uuid>,
}

impl ClientboundResourcePackPopPacket {
    pub fn new(id: Option<Uuid>) -> Self {
        Self { id }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundResourcePackPopPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundResourcePackPopPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_optional(|b| b.read_uuid());
                ClientboundResourcePackPopPacket::new(id)
            },
        )
    }
}

impl Packet for ClientboundResourcePackPopPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        RESOURCE_PACK_POP_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_optional(&self.id, |b, u| {
            b.write_uuid(u);
        });
    }
}
