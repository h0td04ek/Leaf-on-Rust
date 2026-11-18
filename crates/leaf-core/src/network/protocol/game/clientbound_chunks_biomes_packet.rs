//! `ClientboundChunksBiomesPacket` — отправка биомов чанков.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr, PacketListenerObj};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::world::level::chunk_pos::ChunkPos;

/// `ClientboundChunksBiomesPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundChunksBiomesPacket {
    /// `chunkBiomeData`.
    pub chunk_biome_data: Vec<ChunkBiomeData>,
}

/// `ChunkBiomeData` — данные биомов одного чанка.
#[derive(Clone, Debug)]
pub struct ChunkBiomeData {
    /// `pos`.
    pub pos: ChunkPos,
    /// `buffer` — сериализованные биомы всех секций.
    pub buffer: Vec<u8>,
}

impl ChunkBiomeData {
    /// `ChunkBiomeData(buffer)` — десериализация.
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let pos = buf.read_chunk_pos();
        let buffer = buf.read_byte_array_n(2097152).unwrap_or_default();
        Self { pos, buffer }
    }

    /// `getReadBuffer`.
    pub fn get_read_buffer(&self) -> FriendlyByteBuf {
        FriendlyByteBuf::from_bytes(&self.buffer)
    }

    /// `write` — запись `ChunkBiomeData`.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_chunk_pos(&self.pos);
        buf.write_byte_array(&self.buffer);
    }
}

impl ClientboundChunksBiomesPacket {
    /// `forChunks` — создаёт пакет из списка чанков.
    /// Требует `LevelChunk` — будет добавлено после порта `world::level::chunk`.
    //  будет добавлен после порта .
    // pub fn for_chunks(chunks: &[LevelChunk]) -> Self { ... }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundChunksBiomesPacket> {
        crate::network::protocol::packet::packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundChunksBiomesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let data = buf.read_collection(|n| Vec::with_capacity(n), |b| ChunkBiomeData::read(b));
                ClientboundChunksBiomesPacket { chunk_biome_data: data }
            },
        )
    }
}

impl Packet for ClientboundChunksBiomesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_chunks_biomes::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.chunk_biome_data, |b, v| v.write(b));
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
