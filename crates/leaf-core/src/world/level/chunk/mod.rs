//! `net.minecraft.world.level.chunk` — хранилище блоков/биомов чанка.

pub mod chunk_access;
pub mod chunk_pyramid;
pub mod storage;
pub mod data_layer;
pub mod level_chunk;
pub mod level_chunk_section;
pub mod light_chunk;
pub mod light_chunk_getter;
pub mod packed_ticks;
pub mod palette;
pub mod paletted_container;
pub mod proto_chunk;

pub use level_chunk_section::LevelChunkSection;
pub use light_chunk::LightChunk;
pub use light_chunk_getter::LightChunkGetter;
pub use paletted_container::PalettedContainer;

pub enum RuntimeChunk {
    Full(level_chunk::LevelChunk),
    Proto(proto_chunk::ProtoChunk),
}

impl RuntimeChunk {
    pub fn access(&self) -> &chunk_access::ChunkAccess {
        match self {
            Self::Full(chunk) => &chunk.chunk_access,
            Self::Proto(chunk) => &chunk.chunk_access,
        }
    }

    pub fn access_mut(&mut self) -> &mut chunk_access::ChunkAccess {
        match self {
            Self::Full(chunk) => &mut chunk.chunk_access,
            Self::Proto(chunk) => &mut chunk.chunk_access,
        }
    }
}
