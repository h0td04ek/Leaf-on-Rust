//! Перенос `ServerboundChunkBatchReceivedPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundChunkBatchReceivedPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ServerboundChunkBatchReceivedPacket {
    /// `desiredChunksPerTick`.
    pub desired_chunks_per_tick: f32,
}

impl ServerboundChunkBatchReceivedPacket {
    pub fn new(desired_chunks_per_tick: f32) -> Self {
        Self { desired_chunks_per_tick }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChunkBatchReceivedPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChunkBatchReceivedPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundChunkBatchReceivedPacket::new(buf.read_float())
            },
        )
    }
}

impl Packet for ServerboundChunkBatchReceivedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_chunk_batch_received::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.desired_chunks_per_tick);
    }
}
