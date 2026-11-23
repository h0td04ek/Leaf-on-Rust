use std::collections::HashMap;
use std::sync::Mutex;

use leaf_core::core::block_pos::BlockPos;
use std::io;

use leaf_core::util::uuid::Uuid;

use crate::blocks::{Block, Blocks, GLOBAL_BITS};

use leaf_core::network::friendly_byte_buf::FriendlyByteBuf;
use leaf_core::network::protocol::game::{
    clientbound_block_update_packet::ClientboundBlockUpdatePacket,
    clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData,
    clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket,
    clientbound_light_update_packet_data::ClientboundLightUpdatePacketData,
};

use crate::audience::Audience;
use crate::maps::MAP_SIZE;
use crate::wire::{PacketIds, Wire};

const MIN_Y: i32 = -64;
const SECTIONS: usize = 24;
pub const AIR: i32 = 0;
const STONE: i32 = 1;
pub const SPAWN_Y: i32 = 16;

type Changes = HashMap<BlockPos, i32>;
type WorldEdits = HashMap<(i32, i32), Changes>;

pub struct FlatWorld {
    flat_chunk_body: Vec<u8>,
    light: ClientboundLightUpdatePacketData,
    biome: i32,
    edits: Mutex<WorldEdits>,
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
        state_at(&self.edits.lock().unwrap(), pos)
    }

    pub fn editable(pos: BlockPos) -> bool {
        (MIN_Y..MIN_Y + SECTIONS as i32 * 16).contains(&pos.y)
    }

    pub fn refresh_block(&self, pos: BlockPos, audience: &Audience, player: Uuid) {
        let edits = self.edits.lock().unwrap();
        audience.send(
            player,
            ClientboundBlockUpdatePacket::new(pos, state_at(&edits, pos)),
        );
    }

    pub fn break_block(&self, pos: BlockPos, expected: i32, audience: &Audience) -> bool {
        let mut edits = self.edits.lock().unwrap();
        if expected == AIR || state_at(&edits, pos) != expected {
            return false;
        }
        let block = Blocks::vanilla().get(expected);
        if let Some((other, property, value)) = block.paired(pos, expected) {
            let second = state_at(&edits, other);
            if matching_half(block, second, property, value) {
                update(&mut edits, other, AIR, audience);
            }
        }
        update(&mut edits, pos, AIR, audience);
        true
    }

    pub fn place_block(
        &self,
        clicked: BlockPos,
        pos: BlockPos,
        state: i32,
        intersects_player: impl Fn(BlockPos) -> bool,
        audience: &Audience,
    ) -> bool {
        let mut edits = self.edits.lock().unwrap();
        let blocks = Blocks::vanilla();
        let vacant = |p| {
            let previous = blocks.get(state_at(&edits, p));
            Self::editable(p)
                && previous.replaceable
                && previous.default != blocks.get(state).default
                && !intersects_player(p)
        };
        if state_at(&edits, clicked) == AIR || !vacant(pos) {
            return false;
        }
        let block = Blocks::vanilla().get(state);
        let pair = block.paired(pos, state);
        if pair.is_some_and(|(other, _, _)| !vacant(other)) {
            return false;
        }
        if block.door() || block.name.ends_with("_bed") {
            if state_at(&edits, pos.offset(0, -1, 0)) == AIR {
                return false;
            }
            if block.name.ends_with("_bed")
                && pair.is_some_and(|(other, _, _)| state_at(&edits, other.offset(0, -1, 0)) == AIR)
            {
                return false;
            }
        }
        update(&mut edits, pos, state, audience);
        if let Some((other, property, value)) = pair {
            update(
                &mut edits,
                other,
                block.state(state, &[(property, value)]),
                audience,
            );
        }
        true
    }

    pub fn toggle_door(&self, pos: BlockPos, audience: &Audience) -> bool {
        let mut edits = self.edits.lock().unwrap();
        let state = state_at(&edits, pos);
        let block = Blocks::vanilla().get(state);
        if !(block.door()
            || block.name.ends_with("_trapdoor")
            || block.name.ends_with("_fence_gate"))
            || matches!(block.name, "iron_door" | "iron_trapdoor")
        {
            return false;
        }
        let open = if block.property(state, "open") == Some("true") {
            "false"
        } else {
            "true"
        };
        if let Some((other, property, value)) = block.paired(pos, state) {
            let second = state_at(&edits, other);
            if matching_half(block, second, property, value) {
                update(
                    &mut edits,
                    other,
                    block.state(second, &[("open", open)]),
                    audience,
                );
            }
        }
        update(
            &mut edits,
            pos,
            block.state(state, &[("open", open)]),
            audience,
        );
        true
    }

    pub fn map_colors(&self, x: i32, z: i32, step: i32) -> Vec<u8> {
        let side = MAP_SIZE as usize;
        let mut pixels = vec![46; side * side]; // stone, normal brightness
        let mut heights = vec![SPAWN_Y - 1; pixels.len()];
        let world = self.edits.lock().unwrap();
        let end_x = x + MAP_SIZE * step;
        let end_z = z + MAP_SIZE * step;
        for chunk_x in (x >> 4)..=((end_x - 1) >> 4) {
            for chunk_z in (z >> 4)..=((end_z - 1) >> 4) {
                let Some(changes) = world.get(&(chunk_x, chunk_z)) else {
                    continue;
                };
                for (pos, &state) in changes {
                    if !(x..end_x).contains(&pos.x) || !(z..end_z).contains(&pos.z) || state == AIR
                    {
                        continue;
                    }
                    let pixel =
                        ((pos.z - z) / step) as usize * side + ((pos.x - x) / step) as usize;
                    let color = Blocks::vanilla().get(state).map_color;
                    if color != 0 && pos.y >= heights[pixel] {
                        heights[pixel] = pos.y;
                        pixels[pixel] = color * 4 + 2;
                    }
                }
            }
        }
        pixels
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
            let mut lookup = HashMap::from([(AIR, 0)]);
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
                *palette_index = *lookup.entry(state).or_insert_with(|| {
                    palette.push(state);
                    palette.len() - 1
                });
            }
            sections.write_short(solid);
            if solid == 0 || (solid == 4096 && palette.len() == 2) {
                sections
                    .write_byte(0)
                    .write_var_int(palette[usize::from(solid != 0)]);
            } else {
                let local_bits =
                    (usize::BITS - (palette.len() - 1).leading_zeros()).max(4) as usize;
                let global = local_bits > 8;
                let bits = if global { GLOBAL_BITS } else { local_bits };
                sections.write_byte(bits as i32);
                if !global {
                    sections.write_var_int(palette.len() as i32);
                    for &id in &palette {
                        sections.write_var_int(id);
                    }
                }
                for group in indices.chunks(64 / bits) {
                    let mut word = 0_i64;
                    for (position, &index) in group.iter().enumerate() {
                        let index = if global {
                            palette[index] as usize
                        } else {
                            index
                        };
                        word |= (index as i64) << (position * bits);
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

fn state_at(edits: &WorldEdits, pos: BlockPos) -> i32 {
    edits
        .get(&chunk(pos))
        .and_then(|changes| changes.get(&pos).copied())
        .unwrap_or_else(|| base(pos))
}

fn update(edits: &mut WorldEdits, pos: BlockPos, state: i32, audience: &Audience) {
    let coord = chunk(pos);
    let changes = edits.entry(coord).or_default();
    if state == base(pos) {
        changes.remove(&pos);
    } else {
        changes.insert(pos, state);
    }
    if changes.is_empty() {
        edits.remove(&coord);
    }
    audience.broadcast(ClientboundBlockUpdatePacket::new(pos, state), Some(coord));
}

fn matching_half(block: &Block, state: i32, property: &str, value: &str) -> bool {
    let actual = Blocks::vanilla().get(state);
    block.name == actual.name && actual.property(state, property) == Some(value)
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
