//! Перенос `ClientboundForgetLevelChunkPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::level::chunk_pos::ChunkPos;

/// `ClientboundForgetLevelChunkPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundForgetLevelChunkPacket {
    /// `pos`.
    pub pos: ChunkPos,
}

impl ClientboundForgetLevelChunkPacket {
    pub fn new(pos: ChunkPos) -> Self {
        Self { pos }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundForgetLevelChunkPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundForgetLevelChunkPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundForgetLevelChunkPacket::new(buf.read_chunk_pos())
            },
        )
    }
}

impl Packet for ClientboundForgetLevelChunkPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_forget_level_chunk::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_chunk_pos(&self.pos);
    }
}
