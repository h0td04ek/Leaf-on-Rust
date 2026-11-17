//! Перенос `ServerboundPickItemFromEntityPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ServerboundPickItemFromEntityPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPickItemFromEntityPacket {
    /// `id`.
    pub id: i32,
    /// `includeData`.
    pub include_data: bool,
}

impl ServerboundPickItemFromEntityPacket {
    /// `ServerboundPickItemFromEntityPacket::new`.
    pub fn new(id: i32, include_data: bool) -> Self {
        Self { id, include_data }
    }
    /// `id`.
    pub fn id(&self) -> i32 { self.id }
    /// `includeData`.
    pub fn include_data(&self) -> bool { self.include_data }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPickItemFromEntityPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPickItemFromEntityPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let include_data = buf.read_bool();
                ServerboundPickItemFromEntityPacket::new(id, include_data)
            },
        )
    }
}

impl Packet for ServerboundPickItemFromEntityPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_pick_item_from_entity::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_bool(self.include_data);    }
}
