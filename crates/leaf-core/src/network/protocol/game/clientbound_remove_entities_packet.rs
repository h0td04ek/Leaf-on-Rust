//! Перенос `ClientboundRemoveEntitiesPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundRemoveEntitiesPacket` — `IntList` entity ids.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundRemoveEntitiesPacket {
    /// `entityIds`.
    pub entity_ids: Vec<i32>,
}

impl ClientboundRemoveEntitiesPacket {
    /// `new ClientboundRemoveEntitiesPacket(IntList)`.
    pub fn new(entity_ids: Vec<i32>) -> Self {
        Self { entity_ids }
    }

    /// `getEntityIds`.
    pub fn get_entity_ids(&self) -> &[i32] {
        &self.entity_ids
    }

    /// `STREAM_CODEC` (`readIntIdList`/`writeIntIdList` — VarInt count + VarInt ids).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRemoveEntitiesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRemoveEntitiesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundRemoveEntitiesPacket::new(buf.read_int_id_list())
            },
        )
    }
}

impl Packet for ClientboundRemoveEntitiesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_remove_entities::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int_id_list(&self.entity_ids);
    }
}
