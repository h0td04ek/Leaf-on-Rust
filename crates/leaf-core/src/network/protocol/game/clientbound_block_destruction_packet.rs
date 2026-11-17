//! Перенос `ClientboundBlockDestructionPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundBlockDestructionPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundBlockDestructionPacket {
    /// `id`.
    pub id: i32,
    /// `pos`.
    pub pos: crate::core::block_pos::BlockPos,
    /// `progress`.
    pub progress: u8,
}

impl ClientboundBlockDestructionPacket {
    /// `ClientboundBlockDestructionPacket::new`.
    pub fn new(id: i32, pos: crate::core::block_pos::BlockPos, progress: u8) -> Self {
        Self { id, pos, progress }
    }
    /// `id` getter.
    pub fn get_id(&self) -> i32 {
        self.id
    }
    /// `pos` getter.
    pub fn get_pos(&self) -> crate::core::block_pos::BlockPos {
        self.pos
    }
    /// `progress` getter.
    pub fn get_progress(&self) -> u8 {
        self.progress
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundBlockDestructionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundBlockDestructionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let pos = buf.read_block_pos();
                let progress = buf.read_byte() as u8;

                ClientboundBlockDestructionPacket::new(id, pos, progress)
            },
        )
    }
}

impl Packet for ClientboundBlockDestructionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_block_destruction::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_block_pos(&self.pos);
        buf.write_byte(self.progress as i32);
    }
}
