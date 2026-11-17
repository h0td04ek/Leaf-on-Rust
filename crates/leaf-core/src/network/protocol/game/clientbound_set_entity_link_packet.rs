//! Перенос `ClientboundSetEntityLinkPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetEntityLinkPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundSetEntityLinkPacket {
    /// `sourceId`.
    pub source_id: i32,
    /// `destId` (0 = нет связи).
    pub dest_id: i32,
}

impl ClientboundSetEntityLinkPacket {
    /// `ClientboundSetEntityLinkPacket::new` (поля пакета).
    pub fn new(source_id: i32, dest_id: i32) -> Self {
        Self { source_id, dest_id }
    }

    /// `getSourceId`.
    pub fn get_source_id(&self) -> i32 {
        self.source_id
    }

    /// `getDestId`.
    pub fn get_dest_id(&self) -> i32 {
        self.dest_id
    }

    /// `STREAM_CODEC` — INT×2 (не VarInt!).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetEntityLinkPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetEntityLinkPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let source_id = buf.read_int();
                let dest_id = buf.read_int();
                ClientboundSetEntityLinkPacket::new(source_id, dest_id)
            },
        )
    }
}

impl Packet for ClientboundSetEntityLinkPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_entity_link::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.source_id);
        buf.write_int(self.dest_id);
    }
}
