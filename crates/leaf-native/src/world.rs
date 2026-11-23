#[path = "blast.rs"]
mod blast;
#[path = "fill.rs"]
mod fill;
#[path = "world_storage.rs"]
mod storage;

pub use fill::FillMode;

use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use leaf_core::core::{block_pos::BlockPos, direction::Direction};
use leaf_core::world::phys::vec3::Vec3;
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

use storage::WorldStorage;

const MIN_Y: i32 = -64;
const SECTIONS: usize = 24;
pub const AIR: i32 = 0;
const STONE: i32 = 1;
pub const SPAWN_Y: i32 = 16;

type Changes = HashMap<BlockPos, i32>;
type Chunk = (i32, i32);
type WorldEdits = HashMap<Chunk, Arc<Changes>>;

#[derive(Default)]
struct Edits {
    chunks: Arc<WorldEdits>,
    dirty: HashSet<Chunk>,
}

impl Deref for Edits {
    type Target = WorldEdits;

    fn deref(&self) -> &Self::Target {
        &self.chunks
    }
}

#[derive(Clone)]
pub struct BlockView(Arc<WorldEdits>);

pub struct FlatWorld {
    flat_chunk_body: Vec<u8>,
    light: ClientboundLightUpdatePacketData,
    biome: i32,
    edits: Mutex<Edits>,
    storage: Option<Mutex<WorldStorage>>,
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
            edits: Mutex::default(),
            storage: None,
        }
    }

    pub fn open(biome: i32, region_path: PathBuf) -> io::Result<Self> {
        let storage = WorldStorage::new(region_path);
        let chunks = Arc::new(storage.load()?);
        let mut world = Self::new(biome);
        world.edits = Mutex::new(Edits {
            chunks,
            dirty: HashSet::new(),
        });
        world.storage = Some(Mutex::new(storage));
        Ok(world)
    }

    pub fn save(&self, game_time: i64) -> io::Result<usize> {
        let Some(storage) = &self.storage else {
            return Ok(0);
        };
        let storage = storage.lock().unwrap();
        let (snapshot, dirty) = {
            let mut edits = self.edits.lock().unwrap();
            (Arc::clone(&edits.chunks), std::mem::take(&mut edits.dirty))
        };
        let count = dirty.len();
        let mut pending = dirty.into_iter();
        while let Some(coord) = pending.next() {
            if let Err(error) =
                storage.write(coord, snapshot.get(&coord).map(Arc::as_ref), game_time)
            {
                let mut edits = self.edits.lock().unwrap();
                // Edits made while the snapshot was saving remain dirty as well.
                edits.dirty.insert(coord);
                edits.dirty.extend(pending);
                return Err(error);
            }
        }
        Ok(count)
    }

    pub fn snapshot(&self) -> BlockView {
        BlockView(Arc::clone(&self.edits.lock().unwrap().chunks))
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
        has_room(&self.edits.lock().unwrap(), position, height, aquatic)
    }

    pub fn open_surface(&self, x: i32, z: i32) -> Option<Vec3> {
        let edits = self.edits.lock().unwrap();
        let column = edits.get(&(x >> 4, z >> 4));
        let top_edit = column
            .into_iter()
            .flat_map(|changes| changes.iter())
            .filter(|(pos, state)| {
                pos.x == x && pos.z == z && !Blocks::vanilla().get(**state).replaceable
            })
            .map(|(pos, _)| pos.y)
            .max();
        let top = top_edit.unwrap_or(SPAWN_Y - 1).max(SPAWN_Y - 1);
        for y in (MIN_Y..=top).rev() {
            let pos = BlockPos::new(x, y, z);
            if !Blocks::vanilla().get(state_at(&edits, pos)).replaceable {
                return FlatWorld::editable(pos.offset(0, 1, 0))
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
        let changed = self.edits.lock().unwrap().get(&(x, z)).cloned();
        if let Some(changes) = changed {
            let data = self.changed_chunk(x, z, &changes);
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

impl BlockView {
    pub fn block(&self, pos: BlockPos) -> i32 {
        state_at(&self.0, pos)
    }

    pub fn has_room(&self, position: Vec3, height: f64, aquatic: bool) -> bool {
        has_room(&self.0, position, height, aquatic)
    }
}

fn has_room(edits: &WorldEdits, position: Vec3, height: f64, aquatic: bool) -> bool {
    let floor = BlockPos::containing_vec3(&position);
    if !FlatWorld::editable(floor) {
        return false;
    }
    let roof = BlockPos::containing_vec3(&position.add(0.0, height - 0.01, 0.0));
    let chunk_edits = edits.get(&chunk(floor));
    let blocks = Blocks::vanilla();
    (floor.y..=roof.y).all(|y| {
        let pos = BlockPos::new(floor.x, y, floor.z);
        let state = chunk_edits
            .and_then(|edits| edits.get(&pos).copied())
            .unwrap_or_else(|| base(pos));
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

fn state_at(edits: &WorldEdits, pos: BlockPos) -> i32 {
    edits
        .get(&chunk(pos))
        .and_then(|changes| changes.get(&pos).copied())
        .unwrap_or_else(|| base(pos))
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
    if state_at(edits, pos) == state {
        return;
    }
    edits.dirty.insert(coord);
    let edits = Arc::make_mut(&mut edits.chunks);
    let changes = Arc::make_mut(edits.entry(coord).or_default());
    if state == base(pos) {
        changes.remove(&pos);
    } else {
        changes.insert(pos, state);
    }
    if changes.is_empty() {
        edits.remove(&coord);
    }
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
