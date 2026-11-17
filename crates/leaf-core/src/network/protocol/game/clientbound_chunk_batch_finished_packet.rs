//! Перенос `ClientboundChunkBatchFinishedPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundChunkBatchFinishedPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundChunkBatchFinishedPacket {
    /// `batchSize`.
    pub batch_size: i32,
}

impl ClientboundChunkBatchFinishedPacket {
    pub fn new(batch_size: i32) -> Self {
        Self { batch_size }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundChunkBatchFinishedPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundChunkBatchFinishedPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ClientboundChunkBatchFinishedPacket::new(buf.read_var_int()),
        )
    }
}

impl Packet for ClientboundChunkBatchFinishedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_chunk_batch_finished::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.batch_size);
    }
}
