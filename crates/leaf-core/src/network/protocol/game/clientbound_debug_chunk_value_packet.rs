//! `ClientboundDebugChunkValuePacket` — debug-значение чанка.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::world::level::chunk_pos::ChunkPos;
use crate::util::debug::debug_subscription::Update;

/// `ClientboundDebugChunkValuePacket`.
#[derive(Debug)]
pub struct ClientboundDebugChunkValuePacket {
    /// `chunkPos`.
    pub chunk_pos: ChunkPos,
    /// `update`.
    pub update: Update,
}

impl ClientboundDebugChunkValuePacket {
    /// `new`.
    pub fn new(chunk_pos: ChunkPos, update: Update) -> Self {
        Self { chunk_pos, update }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDebugChunkValuePacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundDebugChunkValuePacket| {
                buf.write_chunk_pos(&pkt.chunk_pos);
                Update::stream_codec().encode(buf, &pkt.update);
            },
            |buf: &mut FriendlyByteBuf| {
                let chunk_pos = buf.read_chunk_pos();
                let update = Update::stream_codec().decode(buf);
                Self::new(chunk_pos, update)
            },
        )
    }
}

impl Packet for ClientboundDebugChunkValuePacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_debug_chunk_value::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_chunk_pos(&self.chunk_pos);
        Update::stream_codec().encode(buf, &self.update);
    }
}
