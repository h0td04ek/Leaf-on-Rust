//! Перенос `ServerboundJigsawGeneratePacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundJigsawGeneratePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundJigsawGeneratePacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `levels`.
    pub levels: i32,
    /// `keepJigsaws`.
    pub keep_jigsaws: bool,
}

impl ServerboundJigsawGeneratePacket {
    /// `new`.
    pub fn new(pos: BlockPos, levels: i32, keep_jigsaws: bool) -> Self {
        Self { pos, levels, keep_jigsaws }
    }

    /// `getPos`/`levels`/`keepJigsaws`.
    pub fn pos(&self) -> BlockPos {
        self.pos
    }
    /// `levels`.
    pub fn levels(&self) -> i32 {
        self.levels
    }
    /// `keepJigsaws`.
    pub fn keep_jigsaws(&self) -> bool {
        self.keep_jigsaws
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundJigsawGeneratePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundJigsawGeneratePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let levels = buf.read_var_int();
                let keep_jigsaws = buf.read_bool();
                ServerboundJigsawGeneratePacket::new(pos, levels, keep_jigsaws)
            },
        )
    }
}

impl Packet for ServerboundJigsawGeneratePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_jigsaw_generate::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_var_int(self.levels);
        buf.write_bool(self.keep_jigsaws);
    }
}
