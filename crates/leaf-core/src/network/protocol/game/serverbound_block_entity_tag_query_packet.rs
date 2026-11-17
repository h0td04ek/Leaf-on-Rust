//! Перенос `ServerboundBlockEntityTagQueryPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ServerboundBlockEntityTagQueryPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundBlockEntityTagQueryPacket {
    /// `transactionId`.
    pub transaction_id: i32,
    /// `pos`.
    pub pos: crate::core::block_pos::BlockPos,
}

impl ServerboundBlockEntityTagQueryPacket {
    /// `ServerboundBlockEntityTagQueryPacket::new`.
    pub fn new(transaction_id: i32, pos: crate::core::block_pos::BlockPos) -> Self {
        Self { transaction_id, pos }
    }
    /// `getTransactionId`.
    pub fn get_transaction_id(&self) -> i32 { self.transaction_id }
    /// `getPos`.
    pub fn get_pos(&self) -> crate::core::block_pos::BlockPos { self.pos }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundBlockEntityTagQueryPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundBlockEntityTagQueryPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let transaction_id = buf.read_var_int();
                let pos = buf.read_block_pos();
                ServerboundBlockEntityTagQueryPacket::new(transaction_id, pos)
            },
        )
    }
}

impl Packet for ServerboundBlockEntityTagQueryPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_block_entity_tag_query::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        buf.write_block_pos(&self.pos);    }
}
