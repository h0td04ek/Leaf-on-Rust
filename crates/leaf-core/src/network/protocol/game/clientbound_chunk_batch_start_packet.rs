//! Перенос `ClientboundChunkBatchStartPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundChunkBatchStartPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundChunkBatchStartPacket;

/// `INSTANCE`.
pub const CLIENTBOUND_CHUNK_BATCH_START_PACKET: ClientboundChunkBatchStartPacket =
    ClientboundChunkBatchStartPacket;

impl ClientboundChunkBatchStartPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundChunkBatchStartPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ClientboundChunkBatchStartPacket| {},
            |_buf: &mut FriendlyByteBuf| ClientboundChunkBatchStartPacket,
        )
    }
}

impl Packet for ClientboundChunkBatchStartPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_chunk_batch_start::<Self>().erase()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
