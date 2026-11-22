use std::collections::HashMap;
use std::sync::Mutex;

use leaf_core::core::block_pos::BlockPos;
use std::io;

use leaf_core::network::friendly_byte_buf::FriendlyByteBuf;
use leaf_core::network::protocol::game::{
    clientbound_block_update_packet::ClientboundBlockUpdatePacket,
    clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData,
    clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket,
    clientbound_light_update_packet_data::ClientboundLightUpdatePacketData,
};

use crate::audience::Audience;
use crate::wire::{PacketIds, Wire};

const MIN_Y: i32 = -64;
const SECTIONS: usize = 24;
pub const AIR: i32 = 0;
const STONE: i32 = 1;
pub const SPAWN_Y: i32 = 16;

type Changes = HashMap<BlockPos, i32>;

pub struct FlatWorld {
    flat_chunk_body: Vec<u8>,
    light: ClientboundLightUpdatePacketData,
    biome: i32,
    edits: Mutex<HashMap<(i32, i32), Changes>>,
}

pub fn chunk(pos: BlockPos) -> (i32, i32) {
    (pos.x >> 4, pos.z >> 4)
}

fn base(pos: BlockPos) -> i32 {
    if (MIN_Y..SPAWN_Y).contains(&pos.y) {
        STONE
    } else {
        AIR
    }
}

impl FlatWorld {
    pub fn new(biome: i32) -> Self {
        let mut sections = FriendlyByteBuf::new();
        let solid = ((SPAWN_Y - MIN_Y) / 16) as usize;
        for section in 0..SECTIONS {
            let stone = section < solid;
            sections.write_short(if stone { 4096 } else { 0 });
            // Single-value palettes in 1.21.11 have no storage longs.
            sections
                .write_byte(0)
                .write_var_int(if stone { STONE } else { AIR });
            sections.write_byte(0).write_var_int(biome);
        }
        let data = ClientboundLevelChunkPacketData::new(
            [(1, heightmap()), (4, heightmap()), (5, heightmap())]
                .into_iter()
                .collect(),
            sections.copy_bytes(),
            Vec::new(),
        );

        let layers = SECTIONS + 2;
        let sky = solid + 1;
        let light = ClientboundLightUpdatePacketData::new(
            (0..layers).map(|index| index >= sky).collect(),
            Vec::new(),
            (0..layers).map(|index| index < sky).collect(),
            vec![true; layers],
            vec![vec![0xff; 2048]; layers - sky],
            Vec::new(),
        );
        let mut body = FriendlyByteBuf::new();
        data.write(&mut body);
        light.write(&mut body);
        Self {
            flat_chunk_body: body.copy_bytes(),
            light,
            biome,
            edits: Mutex::new(HashMap::new()),
        }
    }

    pub fn block(&self, pos: BlockPos) -> i32 {
        self.edits
            .lock()
            .unwrap()
            .get(&chunk(pos))
            .and_then(|edits| edits.get(&pos).copied())
            .unwrap_or_else(|| base(pos))
    }

    pub fn editable(pos: BlockPos) -> bool {
        (MIN_Y..MIN_Y + SECTIONS as i32 * 16).contains(&pos.y)
    }

    pub fn change(&self, pos: BlockPos, state: i32, audience: &Audience) -> i32 {
        let coord = chunk(pos);
        let mut world = self.edits.lock().unwrap();
        let original = base(pos);
        let mut previous = world
            .get(&coord)
            .and_then(|edits| edits.get(&pos).copied())
            .unwrap_or(original);
        if Self::editable(pos) && (previous == AIR) != (state == AIR) {
            let changes = world.entry(coord).or_default();
            if state == original {
                changes.remove(&pos);
            } else {
                changes.insert(pos, state);
            }
            if changes.is_empty() {
                world.remove(&coord);
            }
        } else if state != AIR {
            previous = AIR; // No replacement occurred.
        }
        let current = world
            .get(&coord)
            .and_then(|edits| edits.get(&pos).copied())
            .unwrap_or(original);
        audience.broadcast(ClientboundBlockUpdatePacket::new(pos, current), Some(coord));
        previous
    }

    pub fn send_chunk(&self, x: i32, z: i32, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        let changed = self
            .edits
            .lock()
            .unwrap()
            .get(&(x, z))
            .map(|changes| self.changed_chunk(x, z, changes));
        if let Some(data) = changed {
            let packet = ClientboundLevelChunkWithLightPacket::new(x, z, data, self.light.clone());
            wire.send(ids, &packet)
        } else {
            // The unedited terrain and light body are the same in every chunk.
            let mut prefix = FriendlyByteBuf::new();
            prefix
                .write_var_int(ids.id("level_chunk_with_light")?)
                .write_int(x)
                .write_int(z);
            wire.send_parts(&prefix.copy_bytes(), &self.flat_chunk_body)
        }
    }
    fn changed_chunk(&self, x: i32, z: i32, changes: &Changes) -> ClientboundLevelChunkPacketData {
        let mut sections = FriendlyByteBuf::new();
        let mut heights = vec![0_i64; 256usize.div_ceil(7)];
        for section in 0..SECTIONS {
            let mut palette = vec![AIR];
            let mut indices = [0usize; 4096];
            let mut solid = 0;
            for (index, palette_index) in indices.iter_mut().enumerate() {
                let lx = index & 15;
                let lz = (index >> 4) & 15;
                let local_y = 16 * section + (index >> 8);
                let pos = BlockPos::new(
                    x * 16 + lx as i32,
                    MIN_Y + local_y as i32,
                    z * 16 + lz as i32,
                );
                let state = changes.get(&pos).copied().unwrap_or_else(|| base(pos));
                if state != AIR {
                    solid += 1;
                    let column = lx + lz * 16;
                    let shift = 9 * (column % 7);
                    let packed = &mut heights[column / 7];
                    *packed = (*packed & !(511 << shift)) | ((local_y as i64 + 1) << shift);
                }
                *palette_index = match palette.iter().position(|&id| id == state) {
                    Some(index) => index,
                    None => {
                        palette.push(state);
                        palette.len() - 1
                    }
                };
            }
            sections.write_short(solid);
            if solid == 0 || (solid == 4096 && palette.len() == 2) {
                sections
                    .write_byte(0)
                    .write_var_int(palette[usize::from(solid != 0)]);
            } else {
                let bits = (usize::BITS - (palette.len() - 1).leading_zeros()).max(4);
                sections
                    .write_byte(bits as i32)
                    .write_var_int(palette.len() as i32);
                for id in palette {
                    sections.write_var_int(id);
                }
                for group in indices.chunks(64 / bits as usize) {
                    let mut word = 0_i64;
                    for (position, &index) in group.iter().enumerate() {
                        word |= (index as i64) << (position as u32 * bits);
                    }
                    sections.write_long(word);
                }
            }
            sections.write_byte(0).write_var_int(self.biome);
        }
        ClientboundLevelChunkPacketData::new(
            [(1, heights.clone()), (4, heights.clone()), (5, heights)]
                .into_iter()
                .collect(),
            sections.copy_bytes(),
            Vec::new(),
        )
    }
}

fn heightmap() -> Vec<i64> {
    // The 384-block dimension uses nine bits per entry, seven entries per long.
    let mut data = vec![0; 256usize.div_ceil(7)];
    let height = i64::from(SPAWN_Y - MIN_Y);
    for column in 0..256 {
        data[column / 7] |= height << (9 * (column % 7));
    }
    data
}
