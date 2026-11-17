//! Перенос `ServerboundPickItemFromBlockPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ServerboundPickItemFromBlockPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPickItemFromBlockPacket {
    /// `pos`.
    pub pos: crate::core::block_pos::BlockPos,
    /// `includeData`.
    pub include_data: bool,
}

impl ServerboundPickItemFromBlockPacket {
    /// `ServerboundPickItemFromBlockPacket::new`.
    pub fn new(pos: crate::core::block_pos::BlockPos, include_data: bool) -> Self {
        Self { pos, include_data }
    }
    /// `pos`.
    pub fn pos(&self) -> crate::core::block_pos::BlockPos { self.pos }
    /// `includeData`.
    pub fn include_data(&self) -> bool { self.include_data }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPickItemFromBlockPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPickItemFromBlockPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let include_data = buf.read_bool();
                ServerboundPickItemFromBlockPacket::new(pos, include_data)
            },
        )
    }
}

impl Packet for ServerboundPickItemFromBlockPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_pick_item_from_block::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_bool(self.include_data);    }
}
