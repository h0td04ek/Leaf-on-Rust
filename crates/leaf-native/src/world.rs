#[path = "blast.rs"]
mod blast;
#[path = "fill.rs"]
mod fill;
#[path = "fluids.rs"]
mod fluids;
#[path = "world_storage.rs"]
mod storage;
#[path = "terrain.rs"]
mod terrain;

pub use terrain::{Biome, Generation, WorldType, TERRAIN_VERSION};

pub use fill::FillMode;
pub use fluids::{bucket_hit, BucketHit, Fluid, FluidRules};

use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use leaf_core::core::{block_pos::BlockPos, direction::Direction, section_pos::SectionPos};
use leaf_core::world::phys::vec3::Vec3;
use std::io;

use leaf_core::util::uuid::Uuid;

use crate::blocks::{Block, Blocks, GLOBAL_BITS};

use leaf_core::network::friendly_byte_buf::FriendlyByteBuf;
use leaf_core::network::protocol::game::{
    clientbound_block_update_packet::ClientboundBlockUpdatePacket,
    clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData,
    clientbound_light_update_packet_data::ClientboundLightUpdatePacketData,
    clientbound_section_blocks_update_packet::ClientboundSectionBlocksUpdatePacket,
};

use crate::audience::Audience;
use crate::maps::MAP_SIZE;
use crate::wire::{PacketIds, Wire};

use fluids::FluidUpdates;
use storage::{LoadedWorld, WorldStorage};
use terrain::{block_index, GeneratedChunk, Section, Terrain};

const MIN_Y: i32 = -64;
const SECTIONS: usize = 24;
pub const AIR: i32 = 0;
const STONE: i32 = 1;
pub const SPAWN_Y: i32 = 16;

type Changes = HashMap<BlockPos, i32>;
type Chunk = (i32, i32);
type WorldEdits = HashMap<Chunk, Arc<Changes>>;

struct Edits {
    terrain: Arc<Terrain>,
    chunks: Arc<WorldEdits>,
    dirty: HashSet<Chunk>,
    fluids: FluidUpdates,
}

impl Deref for Edits {
    type Target = WorldEdits;

    fn deref(&self) -> &Self::Target {
        &self.chunks
    }
}

#[derive(Clone)]
pub struct BlockView(Arc<WorldEdits>, Arc<Terrain>);

pub struct NativeWorld {
    terrain: Arc<Terrain>,
    biomes: [i32; 10],
    spawn_column: (i32, i32),
    edits: Mutex<Edits>,
    storage: Option<Mutex<WorldStorage>>,
}

pub fn chunk(pos: BlockPos) -> (i32, i32) {
    (pos.x >> 4, pos.z >> 4)
}

impl NativeWorld {
    #[cfg(test)]
    pub fn new(biome: i32) -> Self {
        Self::from_terrain([biome; 10], Arc::new(Terrain::new(Generation::FLAT)), None).unwrap()
    }

    pub fn open(
        biomes: [i32; 10],
        generation: Generation,
        region_path: PathBuf,
    ) -> io::Result<Self> {
        Self::from_terrain(
            biomes,
            Arc::new(Terrain::new(generation)),
            Some(region_path),
        )
    }

    fn from_terrain(
        biomes: [i32; 10],
        terrain: Arc<Terrain>,
        path: Option<PathBuf>,
    ) -> io::Result<Self> {
        let storage = path.map(|path| WorldStorage::new(path, Arc::clone(&terrain)));
        let LoadedWorld {
            chunks,
            pending_fluids,
            legacy_notifications,
        } = storage
            .as_ref()
            .map(WorldStorage::load)
            .transpose()?
            .unwrap_or_default();
        let mut edits = Edits {
            terrain: Arc::clone(&terrain),
            chunks: Arc::new(chunks),
            dirty: HashSet::new(),
            fluids: FluidUpdates::default(),
        };
        for (coord, ticks) in pending_fluids {
            edits.fluids.restore(coord, ticks);
        }
        for pos in legacy_notifications {
            fluids::reschedule(&mut edits, pos);
        }
        Ok(Self {
            spawn_column: terrain.spawn_column(),
            terrain: Arc::clone(&terrain),
            biomes,
            edits: Mutex::new(edits),
            storage: storage.map(Mutex::new),
        })
    }

    pub fn generation(&self) -> Generation {
        self.terrain.generation()
    }

    pub fn sea_level(&self) -> i32 {
        self.terrain.sea_level()
    }

    pub fn spawn(&self) -> BlockPos {
        let (x, z) = self.spawn_column;
        if let Some(position) = self.open_surface(x, z) {
            BlockPos::containing_vec3(&position)
        } else {
            BlockPos::new(x, self.sea_level() + 1, z)
        }
    }

    pub fn save(&self, game_time: i64) -> io::Result<usize> {
        let Some(storage) = &self.storage else {
            return Ok(0);
        };
        let storage = storage.lock().unwrap();
        let (snapshot, dirty) = {
            let mut edits = self.edits.lock().unwrap();
            let dirty = std::mem::take(&mut edits.dirty)
                .into_iter()
                .map(|coord| (coord, edits.fluids.pending(coord, game_time)))
                .collect::<Vec<_>>();
            (Arc::clone(&edits.chunks), dirty)
        };
        let count = dirty.len();
        let mut pending = dirty.into_iter();
        while let Some((coord, fluid_ticks)) = pending.next() {
            if let Err(error) = storage.write(
                coord,
                snapshot.get(&coord).map(Arc::as_ref),
                game_time,
                &fluid_ticks,
            ) {
                let mut edits = self.edits.lock().unwrap();
                // Edits made while the snapshot was saving remain dirty as well.
                edits.dirty.insert(coord);
                edits.dirty.extend(pending.map(|(coord, _)| coord));
                return Err(error);
            }
        }
        Ok(count)
    }

    pub fn snapshot(&self) -> BlockView {
        BlockView(
            Arc::clone(&self.edits.lock().unwrap().chunks),
            Arc::clone(&self.terrain),
        )
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

    pub fn has_room(&self, position: Vec3, height: f64, aquatic: bool) -> bool {
        has_room(
            &self.edits.lock().unwrap(),
            &self.terrain,
            position,
            height,
            aquatic,
        )
    }

    pub fn open_surface(&self, x: i32, z: i32) -> Option<Vec3> {
        let edits = self.edits.lock().unwrap();
        let coord = (x >> 4, z >> 4);
        let terrain = self.terrain.chunk(coord);
        let column = edits.get(&coord).map(Arc::as_ref);
        let top_edit = column
            .into_iter()
            .flat_map(|changes| changes.iter())
            .filter(|(pos, state)| {
                pos.x == x && pos.z == z && !Blocks::vanilla().get(**state).replaceable
            })
            .map(|(pos, _)| pos.y)
            .max();
        let generated_top = terrain.column_top(x, z).saturating_add(1);
        let top = top_edit.unwrap_or(generated_top).max(generated_top);
        for y in (MIN_Y..=top).rev() {
            let pos = BlockPos::new(x, y, z);
            if !Blocks::vanilla()
                .get(block_at(column, &terrain, pos))
                .replaceable
            {
                return NativeWorld::editable(pos.offset(0, 1, 0))
                    .then(|| Vec3::new(f64::from(x) + 0.5, f64::from(y + 1), f64::from(z) + 0.5));
            }
        }
        None
    }

    pub fn toggle_lever(&self, pos: BlockPos, audience: &Audience) -> bool {
        let mut edits = self.edits.lock().unwrap();
        let state = state_at(&edits, pos);
        let block = Blocks::vanilla().get(state);
        if block.name != "lever" {
            return false;
        }
        let powered = if block.property(state, "powered") == Some("true") {
            "false"
        } else {
            "true"
        };
        update(
            &mut edits,
            pos,
            block.state(state, &[("powered", powered)]),
            audience,
        );
        true
    }

    pub fn powered_tnt_near(&self, changed: BlockPos) -> Vec<BlockPos> {
        let edits = self.edits.lock().unwrap();
        let blocks = Blocks::vanilla();
        let power = |pos| {
            let state = state_at(&edits, pos);
            let block = blocks.get(state);
            matches!(
                block.name,
                "redstone_block" | "redstone_torch" | "redstone_wall_torch"
            ) || block.name == "lever" && block.property(state, "powered") == Some("true")
        };
        let is_tnt = |pos| blocks.get(state_at(&edits, pos)).name == "tnt";
        let mut powered = Vec::new();
        if is_tnt(changed)
            && Direction::VALUES
                .into_iter()
                .any(|side| power(changed.relative(side)))
        {
            powered.push(changed);
        }
        if power(changed) {
            powered.extend(
                Direction::VALUES
                    .into_iter()
                    .map(|side| changed.relative(side))
                    .filter(|&p| is_tnt(p)),
            );
        }
        powered
    }

    pub fn map_colors(&self, x: i32, z: i32, step: i32) -> Vec<u8> {
        let side = MAP_SIZE as usize;
        let mut pixels = vec![46; side * side]; // stone, normal brightness
        let mut heights = vec![SPAWN_Y - 1; pixels.len()];
        if self.generation().world_type == WorldType::Normal {
            let blocks = Blocks::vanilla();
            for index in 0..pixels.len() {
                let px = x + index as i32 % MAP_SIZE * step + step / 2;
                let pz = z + index as i32 / MAP_SIZE * step + step / 2;
                let (height, state) = self.terrain.surface(px, pz);
                heights[index] = height;
                pixels[index] = blocks.get(state).map_color * 4 + 2;
            }
        }
        let world = self.edits.lock().unwrap();
        let end_x = x + MAP_SIZE * step;
        let end_z = z + MAP_SIZE * step;
        for chunk_x in (x >> 4)..=((end_x - 1) >> 4) {
            for chunk_z in (z >> 4)..=((end_z - 1) >> 4) {
                let Some(changes) = world.get(&(chunk_x, chunk_z)) else {
                    continue;
                };
                for (pos, &state) in changes.iter() {
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
        let changes = self.edits.lock().unwrap().get(&(x, z)).cloned();
        let terrain = self.terrain.chunk((x, z));
        let mut prefix = FriendlyByteBuf::new();
        prefix
            .write_var_int(ids.id("level_chunk_with_light")?)
            .write_int(x)
            .write_int(z);
        if let Some(changes) = changes {
            let encoded = encode_chunk(&terrain, Some(&changes), &self.biomes);
            wire.send_parts(&prefix.copy_bytes(), &encoded)
        } else {
            let encoded = terrain
                .encoded
                .get_or_init(|| encode_chunk(&terrain, None, &self.biomes));
            wire.send_parts(&prefix.copy_bytes(), encoded)
        }
    }
}

fn encode_chunk(
    chunk: &GeneratedChunk,
    changes: Option<&Changes>,
    biome_ids: &[i32; 10],
) -> Vec<u8> {
    let mut buffer = FriendlyByteBuf::new();
    let mut heights = [MIN_Y - 1; 256];
    let mut sky_heights = heights;
    let blocks = Blocks::vanilla();
    let mut palette_indices = vec![0_u16; blocks.state_count()];
    let mut edited: [Vec<(usize, u16)>; SECTIONS] = std::array::from_fn(|_| Vec::new());
    for (&pos, &state) in changes.into_iter().flatten() {
        let (section, index) = block_index(pos).expect("a block edit is within the dimension");
        edited[section].push((index, state as u16));
    }
    let biomes: [u16; 64] =
        std::array::from_fn(|index| biome_ids[chunk.biomes[index % 16] as usize] as u16);
    let mut encoded_biomes = FriendlyByteBuf::new();
    write_palette(&mut encoded_biomes, &biomes, &mut palette_indices, 1, 3, 7);
    let encoded_biomes = encoded_biomes.copy_bytes();
    for (number, original) in chunk.sections.iter().enumerate() {
        let bottom = MIN_Y + number as i32 * 16;
        match (original, edited[number].is_empty()) {
            (Section::Uniform(state), true) => {
                let solid = if *state == AIR as u16 { 0 } else { 4096 };
                buffer
                    .write_short(solid)
                    .write_byte(0)
                    .write_var_int(i32::from(*state));
                if solid != 0 {
                    heights.fill(bottom + 15);
                    if !blocks.get(i32::from(*state)).transmits_sky {
                        sky_heights.fill(bottom + 15);
                    }
                }
            }
            _ => {
                let mut values = match original {
                    Section::Uniform(state) => [*state; 4096],
                    Section::Mixed(states) => **states,
                };
                for &(index, state) in &edited[number] {
                    values[index] = state;
                }
                let mut count = 0;
                for (index, &state) in values.iter().enumerate() {
                    if state != AIR as u16 {
                        count += 1;
                        let y = bottom + (index / 256) as i32;
                        heights[index % 256] = y;
                        if !blocks.get(i32::from(state)).transmits_sky {
                            sky_heights[index % 256] = y;
                        }
                    }
                }
                buffer.write_short(count);
                write_palette(
                    &mut buffer,
                    &values,
                    &mut palette_indices,
                    4,
                    8,
                    GLOBAL_BITS,
                );
            }
        }
        buffer.write_bytes(&encoded_biomes);
    }
    let mut packed = vec![0_i64; 256usize.div_ceil(7)];
    for (column, height) in heights.iter().enumerate() {
        packed[column / 7] |= i64::from(height - MIN_Y + 1) << (9 * (column % 7));
    }
    let data = ClientboundLevelChunkPacketData::new(
        [(1, packed.clone()), (4, packed.clone()), (5, packed)]
            .into_iter()
            .collect(),
        buffer.copy_bytes(),
        Vec::new(),
    );
    let mut encoded = FriendlyByteBuf::new();
    data.write(&mut encoded);
    sky_light(&sky_heights).write(&mut encoded);
    encoded.copy_bytes()
}

fn write_palette(
    buffer: &mut FriendlyByteBuf,
    states: &[u16],
    palette_indices: &mut [u16],
    min_bits: usize,
    max_local: usize,
    global_bits: usize,
) {
    let mut palette = Vec::new();
    for &state in states {
        let index = &mut palette_indices[usize::from(state)];
        if *index == 0 {
            palette.push(state);
            *index = palette.len() as u16;
        }
    }
    if palette.len() == 1 {
        buffer.write_byte(0).write_var_int(i32::from(palette[0]));
    } else {
        let local_bits = (usize::BITS - (palette.len() - 1).leading_zeros()) as usize;
        let global = local_bits > max_local;
        let bits = if global {
            global_bits
        } else {
            local_bits.max(min_bits)
        };
        buffer.write_byte(bits as i32);
        if !global {
            buffer.write_var_int(palette.len() as i32);
            for &id in &palette {
                buffer.write_var_int(i32::from(id));
            }
        }
        for group in states.chunks(64 / bits) {
            let mut word = 0_i64;
            for (position, &state) in group.iter().enumerate() {
                let value = if global {
                    state
                } else {
                    palette_indices[usize::from(state)] - 1
                };
                word |= i64::from(value) << (position * bits);
            }
            buffer.write_long(word);
        }
    }
    for id in palette {
        palette_indices[usize::from(id)] = 0;
    }
}

fn sky_light(heights: &[i32; 256]) -> ClientboundLightUpdatePacketData {
    let mut present = Vec::new();
    let mut empty = Vec::new();
    let mut layers = Vec::new();
    let minimum = *heights.iter().min().unwrap();
    let maximum = *heights.iter().max().unwrap();
    for section in 0..SECTIONS + 2 {
        let bottom = MIN_Y + section as i32 * 16 - 16;
        let has_sky = bottom + 15 > minimum;
        present.push(has_sky);
        empty.push(!has_sky);
        if !has_sky {
            continue;
        }
        let mut light = vec![0xff; 2048];
        if bottom <= maximum {
            for y in 0..16 {
                for (column, pair) in heights.chunks_exact(2).enumerate() {
                    let low = if bottom + y > pair[0] { 0x0f } else { 0 };
                    let high = if bottom + y > pair[1] { 0xf0 } else { 0 };
                    light[y as usize * 128 + column] = low | high;
                }
            }
        }
        layers.push(light);
    }
    ClientboundLightUpdatePacketData::new(
        present,
        Vec::new(),
        empty,
        vec![true; SECTIONS + 2],
        layers,
        Vec::new(),
    )
}

impl BlockView {
    pub fn block(&self, pos: BlockPos) -> i32 {
        self.0
            .get(&chunk(pos))
            .and_then(|changes| changes.get(&pos).copied())
            .unwrap_or_else(|| self.1.block(pos))
    }

    pub fn sees_sky(&self, pos: BlockPos) -> bool {
        let generated = self.1.chunk(chunk(pos));
        let changes = self.0.get(&chunk(pos)).map(Arc::as_ref);
        let last_edit = changes
            .into_iter()
            .flat_map(|changes| changes.keys())
            .filter(|key| key.x == pos.x && key.z == pos.z)
            .map(|key| key.y)
            .max()
            .unwrap_or(pos.y);
        let top = generated.column_top(pos.x, pos.z).max(last_edit);
        let blocks = Blocks::vanilla();
        (pos.y..=top).all(|y| {
            let state = block_at(changes, &generated, BlockPos::new(pos.x, y, pos.z));
            let block = blocks.get(state);
            block.replaceable && block.name != "water" && block.name != "lava"
        })
    }

    pub fn has_room(&self, position: Vec3, height: f64, aquatic: bool) -> bool {
        has_room(&self.0, &self.1, position, height, aquatic)
    }
}

fn has_room(
    edits: &WorldEdits,
    terrain: &Terrain,
    position: Vec3,
    height: f64,
    aquatic: bool,
) -> bool {
    let floor = BlockPos::containing_vec3(&position);
    if !NativeWorld::editable(floor) {
        return false;
    }
    let roof = BlockPos::containing_vec3(&position.add(0.0, height - 0.01, 0.0));
    let coord = chunk(floor);
    let chunk_edits = edits.get(&coord).map(Arc::as_ref);
    let generated = terrain.chunk(coord);
    let blocks = Blocks::vanilla();
    (floor.y..=roof.y).all(|y| {
        let pos = BlockPos::new(floor.x, y, floor.z);
        let state = block_at(chunk_edits, &generated, pos);
        if state == AIR {
            return !aquatic;
        }
        let block = blocks.get(state);
        if aquatic {
            block.name == "water"
        } else {
            block.replaceable && block.name != "water" && block.name != "lava"
        }
    })
}

fn block_at(edits: Option<&Changes>, generated: &GeneratedChunk, pos: BlockPos) -> i32 {
    edits
        .and_then(|changes| changes.get(&pos).copied())
        .unwrap_or_else(|| generated.block(pos))
}

fn state_at(edits: &Edits, pos: BlockPos) -> i32 {
    edits
        .get(&chunk(pos))
        .and_then(|changes| changes.get(&pos).copied())
        .unwrap_or_else(|| edits.terrain.block(pos))
}

fn update(edits: &mut Edits, pos: BlockPos, state: i32, audience: &Audience) {
    set_state(edits, pos, state);
    audience.broadcast(
        ClientboundBlockUpdatePacket::new(pos, state),
        Some(chunk(pos)),
    );
}

fn set_state(edits: &mut Edits, pos: BlockPos, state: i32) {
    let coord = chunk(pos);
    let previous = state_at(edits, pos);
    if previous == state {
        return;
    }
    edits.dirty.insert(coord);
    let base_state = edits.terrain.block(pos);
    let chunks = Arc::make_mut(&mut edits.chunks);
    let changes = Arc::make_mut(chunks.entry(coord).or_default());
    if state == base_state {
        changes.remove(&pos);
    } else {
        changes.insert(pos, state);
    }
    if changes.is_empty() {
        chunks.remove(&coord);
    }
    fluids::notify(edits, pos, previous, state);
}

pub(super) fn broadcast_blocks(
    audience: &Audience,
    updates: impl IntoIterator<Item = (BlockPos, i32)>,
) {
    let mut sections = HashMap::<SectionPos, (Vec<i16>, Vec<i32>)>::new();
    for (pos, state) in updates {
        let (positions, states) = sections.entry(SectionPos::of(&pos)).or_default();
        positions.push(SectionPos::section_relative_pos(&pos));
        states.push(state);
    }
    audience.broadcast_batch(sections.into_iter().map(|(section, (positions, states))| {
        (
            (section.x, section.z),
            ClientboundSectionBlocksUpdatePacket::new(section, positions, states),
        )
    }));
}

fn matching_half(block: &Block, state: i32, property: &str, value: &str) -> bool {
    let actual = Blocks::vanilla().get(state);
    block.name == actual.name && actual.property(state, property) == Some(value)
}
