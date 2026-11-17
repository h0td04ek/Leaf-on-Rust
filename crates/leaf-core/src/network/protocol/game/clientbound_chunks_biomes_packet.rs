//! `ClientboundChunksBiomesPacket` — отправка биомов чанков.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::chunk::level_chunk::LevelChunk;
use crate::world::level::chunk::level_chunk_section::LevelChunkSection;

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
    /// `ChunkBiomeData(chunk)` — из `LevelChunk`.
    pub fn from_chunk(chunk: &LevelChunk) -> Self {
        let pos = chunk.get_pos();
        let size = Self::calculate_chunk_size(chunk);
        let mut buffer = vec![0u8; size];
        let mut buf = FriendlyByteBuf::new(&mut buffer);
        Self::extract_chunk_data(&mut buf, chunk);
        Self { pos, buffer }
    }

    /// `ChunkBiomeData(buffer)` — десериализация.
    pub fn read(buf: &mut FriendlyByteBuf) -> Self {
        let pos = buf.read_chunk_pos();
        let buffer = buf.read_byte_array(2097152);
        Self { pos, buffer }
    }

    /// `calculateChunkSize`.
    fn calculate_chunk_size(chunk: &LevelChunk) -> usize {
        let mut i = 0usize;
        for section in chunk.get_sections() {
            i += section.biomes.get_serialized_size(section.biomes.global_map());
        }
        i
    }

    /// `extractChunkData` — запись биомов всех секций в буфер.
    pub fn extract_chunk_data(buf: &mut FriendlyByteBuf, chunk: &LevelChunk) {
        for section in chunk.get_sections() {
            section.biomes.write(buf, section.biomes.global_map());
        }
    }

    /// `getReadBuffer`.
    pub fn get_read_buffer(&self) -> FriendlyByteBuf<'static> {
        FriendlyByteBuf::new_owned(self.buffer.clone())
    }

    /// `write` — запись `ChunkBiomeData`.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        buf.write_chunk_pos(&self.pos);
        buf.write_byte_array(&self.buffer);
    }
}

impl ClientboundChunksBiomesPacket {
    /// `forChunks`.
    pub fn for_chunks(chunks: &[LevelChunk]) -> Self {
        Self {
            chunk_biome_data: chunks.iter().map(ChunkBiomeData::from_chunk).collect(),
        }
    }

    /// `new`.
    pub fn new(chunk_biome_data: Vec<ChunkBiomeData>) -> Self {
        Self { chunk_biome_data }
    }

    /// `STREAM_CODEC` — `Packet.codec(write, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundChunksBiomesPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundChunksBiomesPacket| {
                buf.write_var_int(pkt.chunk_biome_data.len() as i32);
                for data in &pkt.chunk_biome_data {
                    data.write(buf);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let len = buf.read_var_int();
                let mut data = Vec::new();
                for _ in 0..len {
                    data.push(ChunkBiomeData::read(buf));
                }
                Self::new(data)
            },
        )
    }
}

impl Packet for ClientboundChunksBiomesPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_chunks_biomes::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.chunk_biome_data.len() as i32);
        for data in &self.chunk_biome_data {
            data.write(buf);
        }
    }
}
