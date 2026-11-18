//! `ClientboundLevelChunkPacketData` — данные чанка для клиента.

use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ClientboundLevelChunkPacketData`.
#[derive(Clone, Debug)]
pub struct ClientboundLevelChunkPacketData {
    /// `heightmaps` — Map<Heightmap.Types, long[]>.
    pub heightmaps: std::collections::HashMap<i32, Vec<i64>>,
    /// `buffer` — сериализованные данные секций.
    pub buffer: Vec<u8>,
    /// `blockEntitiesData` — данные block entities.
    pub block_entities_data: Vec<BlockEntityInfo>,
}

/// `BlockEntityInfo` — информация о block entity.
#[derive(Clone, Debug)]
pub struct BlockEntityInfo {
    /// `pos` — BlockPos packed.
    pub pos: i64,
    /// `type` — BlockEntityType id.
    pub type_id: i32,
    /// `tag` — NBT data.
    pub tag: Option<crate::nbt::compound::CompoundTag>,
}

impl BlockEntityInfo {
    /// `create(blockEntity)`.
    pub fn create(pos: i64, type_id: i32, tag: Option<crate::nbt::compound::CompoundTag>) -> Self {
        Self { pos, type_id, tag }
    }

    /// `encode`.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_long(self.pos);
        buf.write_var_int(self.type_id);
        match &self.tag {
            Some(t) => {
                buf.write_bool(true);
                let _ = buf.write_nbt_compound(Some(t));
            }
            None => { buf.write_bool(false); }
        }
    }

    /// `decode`.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let pos = buf.read_var_long();
        let type_id = buf.read_var_int();
        let tag = if buf.read_bool() {
            buf.read_nbt_compound().ok().flatten()
        } else {
            None
        };
        Self { pos, type_id, tag }
    }
}

impl ClientboundLevelChunkPacketData {
    /// `new(...)` — создаёт из chunk data (упрощённо).
    pub fn new(heightmaps: std::collections::HashMap<i32, Vec<i64>>, buffer: Vec<u8>, block_entities: Vec<BlockEntityInfo>) -> Self {
        Self { heightmaps, buffer, block_entities_data: block_entities }
    }

    /// `write` — сериализация.
    pub fn write(&self, buf: &mut FriendlyByteBuf) {
        // Heightmaps — Map<Heightmap.Types, long[]>
        buf.write_var_int(self.heightmaps.len() as i32);
        for (ty, data) in &self.heightmaps {
            buf.write_var_int(*ty);
            buf.write_var_int(data.len() as i32);
            for v in data {
                buf.source.write_i64(*v);
            }
        }
        // Buffer — byte array with VarInt length
        buf.write_var_int(self.buffer.len() as i32);
        buf.write_bytes(&self.buffer);
        // Block entities
        buf.write_var_int(self.block_entities_data.len() as i32);
        for be in &self.block_entities_data {
            be.encode(buf);
        }
    }

    /// `read` — десериализация.
    pub fn read(buf: &mut FriendlyByteBuf, _x: i32, _z: i32) -> Self {
        // Heightmaps
        let heightmap_count = buf.read_var_int();
        let mut heightmaps = std::collections::HashMap::new();
        for _ in 0..heightmap_count {
            let ty = buf.read_var_int();
            let len = buf.read_var_int();
            let mut data = Vec::with_capacity(len as usize);
            for _ in 0..len {
                data.push(buf.source.read_i64());
            }
            heightmaps.insert(ty, data);
        }
        // Buffer
        let buffer_len = buf.read_var_int();
        if buffer_len > 2097152 {
            panic!("Chunk Packet trying to allocate too much memory on read.");
        }
        let buffer = buf.read_bytes_vec(buffer_len as usize);
        // Block entities
        let be_count = buf.read_var_int();
        let mut block_entities = Vec::with_capacity(be_count as usize);
        for _ in 0..be_count {
            block_entities.push(BlockEntityInfo::decode(buf));
        }
        Self::new(heightmaps, buffer, block_entities)
    }
}
