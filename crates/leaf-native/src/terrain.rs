use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, OnceLock};

use leaf_core::core::block_pos::BlockPos;
use leaf_core::random::{LegacyRandomSource, RandomSource};
use leaf_core::world::level::levelgen::synth::improved_noise::ImprovedNoise;

use crate::blocks::Blocks;

use super::{Chunk, AIR, MIN_Y, SECTIONS, SPAWN_Y, STONE};

pub const SEA_LEVEL: i32 = 63;
pub const TERRAIN_VERSION: u8 = 2;
const CACHE_SHARDS: usize = 16;
const CHUNKS_PER_SHARD: usize = 32;
const SECTION_VOLUME: usize = 4096;
const TREE_GRID: i32 = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldType {
    Normal,
    Flat,
}

impl WorldType {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "normal" => Some(Self::Normal),
            "flat" => Some(Self::Flat),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Flat => "flat",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Generation {
    pub world_type: WorldType,
    pub seed: i64,
    pub version: u8,
}

impl Generation {
    pub fn biome_seed(self) -> i64 {
        hash(self.seed, 0x68120, 0x746e0) as i64
    }

    pub const FLAT: Self = Self {
        world_type: WorldType::Flat,
        seed: 0,
        version: 1,
    };
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum Biome {
    Plains,
    Forest,
    BirchForest,
    Taiga,
    Desert,
    SnowyPlains,
    Ocean,
    Beach,
    River,
    Hills,
}

impl Biome {
    pub const NAMES: [&str; 10] = [
        "plains",
        "forest",
        "birch_forest",
        "taiga",
        "desert",
        "snowy_plains",
        "ocean",
        "beach",
        "river",
        "windswept_hills",
    ];
}

pub(super) struct Terrain {
    generation: Generation,
    flat: Arc<GeneratedChunk>,
    overworld: Option<Overworld>,
    cache: [Mutex<ChunkCache>; CACHE_SHARDS],
}

#[derive(Default)]
struct ChunkCache {
    chunks: HashMap<Chunk, Arc<GeneratedChunk>>,
    order: VecDeque<Chunk>,
}

pub(super) struct GeneratedChunk {
    pub sections: [Section; SECTIONS],
    pub biomes: [Biome; 16],
    pub height: [i16; 256],
    pub encoded: OnceLock<Vec<u8>>,
}

pub(super) enum Section {
    Uniform(u16),
    Mixed(Box<[u16; SECTION_VOLUME]>),
}

impl Section {
    pub fn state(&self, index: usize) -> u16 {
        match self {
            Self::Uniform(state) => *state,
            Self::Mixed(states) => states[index],
        }
    }

    fn set(&mut self, index: usize, state: u16) {
        if let Self::Uniform(old) = *self {
            if old == state {
                return;
            }
            *self = Self::Mixed(Box::new([old; SECTION_VOLUME]));
        }
        let Self::Mixed(states) = self else {
            unreachable!()
        };
        states[index] = state;
    }
}

impl GeneratedChunk {
    fn new(biomes: [Biome; 16], height: [i16; 256], sections: [Section; SECTIONS]) -> Self {
        Self {
            sections,
            biomes,
            height,
            encoded: OnceLock::new(),
        }
    }

    pub fn column_top(&self, x: i32, z: i32) -> i32 {
        i32::from(self.height[column(x, z)])
    }

    pub fn block(&self, pos: BlockPos) -> i32 {
        if let Some((section, index)) = block_index(pos) {
            i32::from(self.sections[section].state(index))
        } else {
            AIR
        }
    }

    fn set(&mut self, pos: BlockPos, state: u16) {
        if let Some((section, index)) = block_index(pos) {
            self.sections[section].set(index, state);
        }
    }
}

fn column(x: i32, z: i32) -> usize {
    ((z & 15) * 16 + (x & 15)) as usize
}

pub(super) fn block_index(pos: BlockPos) -> Option<(usize, usize)> {
    let y = pos.y - MIN_Y;
    (0..SECTIONS as i32 * 16).contains(&y).then(|| {
        (
            (y >> 4) as usize,
            ((y & 15) as usize * 256) + column(pos.x, pos.z),
        )
    })
}

impl Terrain {
    pub fn new(generation: Generation) -> Self {
        let flat = Arc::new(GeneratedChunk::new(
            [Biome::Plains; 16],
            [(SPAWN_Y - 1) as i16; 256],
            std::array::from_fn(|section| {
                Section::Uniform(if MIN_Y + section as i32 * 16 < SPAWN_Y {
                    STONE as u16
                } else {
                    AIR as u16
                })
            }),
        ));
        Self {
            generation,
            flat,
            overworld: (generation.world_type == WorldType::Normal)
                .then(|| Overworld::new(generation.seed, generation.version)),
            cache: std::array::from_fn(|_| Mutex::default()),
        }
    }

    pub fn generation(&self) -> Generation {
        self.generation
    }

    pub fn sea_level(&self) -> i32 {
        if self.overworld.is_some() {
            SEA_LEVEL
        } else {
            SPAWN_Y - 1
        }
    }

    pub fn spawn_column(&self) -> (i32, i32) {
        let Some(generator) = &self.overworld else {
            return (0, 0);
        };
        let mut fallback = (0, 0);
        let mut best_height = i32::MIN;
        for radius in 0_i32..=80 {
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    if dx.abs().max(dz.abs()) != radius {
                        continue;
                    }
                    let (x, z) = (dx * 12, dz * 12);
                    let height = generator.column(x, z).height;
                    if height > best_height {
                        fallback = (x, z);
                        best_height = height;
                    }
                    if height > SEA_LEVEL + 2 && height < 105 {
                        let ground = self.chunk((x >> 4, z >> 4));
                        if ground.column_top(x, z) == height {
                            return (x, z);
                        }
                    }
                }
            }
        }
        fallback
    }

    pub fn chunk(&self, coord: Chunk) -> Arc<GeneratedChunk> {
        let Some(generator) = &self.overworld else {
            return Arc::clone(&self.flat);
        };
        let shard = hash(0, coord.0, coord.1) as usize % CACHE_SHARDS;
        {
            let cache = self.cache[shard].lock().unwrap();
            if let Some(chunk) = cache.chunks.get(&coord) {
                return Arc::clone(chunk);
            }
        }
        let chunk = Arc::new(generator.generate(coord));
        let mut cache = self.cache[shard].lock().unwrap();
        if let Some(existing) = cache.chunks.get(&coord) {
            return Arc::clone(existing);
        }
        if cache.chunks.len() == CHUNKS_PER_SHARD {
            let oldest = cache.order.pop_front().unwrap();
            cache.chunks.remove(&oldest);
        }
        cache.order.push_back(coord);
        cache.chunks.insert(coord, Arc::clone(&chunk));
        chunk
    }

    pub fn surface(&self, x: i32, z: i32) -> (i32, i32) {
        let Some(generator) = &self.overworld else {
            return (SPAWN_Y - 1, STONE);
        };
        let column = generator.column(x, z);
        let blocks = &generator.blocks;
        let state = if column.height < SEA_LEVEL {
            blocks.water
        } else {
            match column.biome {
                Biome::Desert | Biome::Beach => blocks.sand,
                Biome::SnowyPlains => blocks.snow,
                Biome::Hills if column.height > 120 => blocks.snow,
                _ => blocks.grass,
            }
        };
        (column.height.max(SEA_LEVEL), i32::from(state))
    }

    pub fn block(&self, pos: BlockPos) -> i32 {
        if block_index(pos).is_none() {
            return AIR;
        }
        if self.overworld.is_none() {
            return self.flat.block(pos);
        }
        self.chunk((pos.x >> 4, pos.z >> 4)).block(pos)
    }
}

#[derive(Clone, Copy)]
struct Column {
    height: i32,
    biome: Biome,
}

struct Overworld {
    seed: i64,
    version: u8,
    land: ImprovedNoise,
    hills: ImprovedNoise,
    detail: ImprovedNoise,
    climate: ImprovedNoise,
    moisture: ImprovedNoise,
    rivers: ImprovedNoise,
    caves: ImprovedNoise,
    blocks: Materials,
}

struct Materials {
    stone: u16,
    deepslate: u16,
    bedrock: u16,
    dirt: u16,
    grass: u16,
    sand: u16,
    sandstone: u16,
    gravel: u16,
    water: u16,
    snow: u16,
    short_grass: u16,
    flower: u16,
    logs: [u16; 3],
    leaves: [u16; 3],
    ores: [(u16, u16); 5],
}

impl Materials {
    fn new() -> Self {
        let blocks = Blocks::vanilla();
        let id = |name| blocks.named(name).unwrap().default as u16;
        Self {
            stone: id("stone"),
            deepslate: id("deepslate"),
            bedrock: id("bedrock"),
            dirt: id("dirt"),
            grass: id("grass_block"),
            sand: id("sand"),
            sandstone: id("sandstone"),
            gravel: id("gravel"),
            water: id("water"),
            snow: id("snow"),
            short_grass: id("short_grass"),
            flower: id("dandelion"),
            logs: [id("oak_log"), id("birch_log"), id("spruce_log")],
            leaves: [id("oak_leaves"), id("birch_leaves"), id("spruce_leaves")],
            ores: [
                (id("coal_ore"), id("deepslate_coal_ore")),
                (id("iron_ore"), id("deepslate_iron_ore")),
                (id("copper_ore"), id("deepslate_copper_ore")),
                (id("gold_ore"), id("deepslate_gold_ore")),
                (id("diamond_ore"), id("deepslate_diamond_ore")),
            ],
        }
    }
}

impl Overworld {
    fn new(seed: i64, version: u8) -> Self {
        let mut random = LegacyRandomSource::new(hash(seed, 0, 0) as i64);
        Self {
            seed,
            version,
            land: ImprovedNoise::new(&mut random),
            hills: ImprovedNoise::new(&mut random),
            detail: ImprovedNoise::new(&mut random),
            climate: ImprovedNoise::new(&mut random),
            moisture: ImprovedNoise::new(&mut random),
            rivers: ImprovedNoise::new(&mut random),
            caves: ImprovedNoise::new(&mut random),
            blocks: Materials::new(),
        }
    }

    fn column(&self, x: i32, z: i32) -> Column {
        let sample = |noise: &ImprovedNoise, scale: f64| {
            noise.noise(f64::from(x) / scale, 0.0, f64::from(z) / scale)
        };
        let (height, river) = if self.version == 1 {
            let land = sample(&self.land, 410.0) + sample(&self.detail, 150.0) * 0.18;
            let relief = sample(&self.hills, 116.0);
            let detail = sample(&self.detail, 32.0);
            let mountain = ((relief - 0.12) * 2.8).clamp(0.0, 1.15);
            let mut height = (69.0 + land * 51.0 + (9.0 + mountain * 67.0) * relief + 5.0 * detail)
                .round() as i32;
            // A smooth valley around a contour of the river field.
            let river = sample(&self.rivers, 210.0).abs();
            if land > -0.11 && height < 95 && river < 0.08 {
                let bank = (river / 0.08).clamp(0.0, 1.0);
                height =
                    lerp(59.0, f64::from(height), bank * bank * (3.0 - 2.0 * bank)).round() as i32;
                if height < SEA_LEVEL - 1 {
                    return Column {
                        height,
                        biome: Biome::River,
                    };
                }
            }
            (height, false)
        } else {
            let warp = sample(&self.detail, 150.0);
            let continents = self.land.noise(
                (f64::from(x) + warp * 55.0) / 465.0,
                0.0,
                (f64::from(z) - warp * 55.0) / 465.0,
            );
            let relief = sample(&self.hills, 125.0);
            let detail = sample(&self.detail, 35.0);
            let ocean_depth = 42.0 * smooth((-continents - 0.08) / 0.47);
            let uplands = 21.0 * smooth((continents + 0.02) / 0.45);
            let mountain = smooth((relief - 0.08) / 0.52) * smooth((continents + 0.24) / 0.36);
            let surface =
                66.0 + uplands - ocean_depth + 12.0 * relief + 41.0 * mountain + 5.0 * detail;
            let river = sample(&self.rivers, 210.0).abs();
            let bank = 1.0 - smooth(river / 0.083);
            let slope = 1.0 - smooth((surface - 76.0) / 30.0);
            let inland = smooth((continents + 0.16) / 0.16);
            let bed = lerp(surface, surface.min(58.0), bank * slope * inland);
            (
                bed.round() as i32,
                bed < 62.0 && surface >= 62.0 && bank > 0.2,
            )
        };
        if river {
            return Column {
                height,
                biome: Biome::River,
            };
        }
        let climate = sample(&self.climate, 247.0);
        let moisture = sample(&self.moisture, 190.0);
        let biome = if height < SEA_LEVEL - 2 {
            Biome::Ocean
        } else if height <= SEA_LEVEL + 1 {
            Biome::Beach
        } else if height > 105 {
            Biome::Hills
        } else if climate < -0.26 {
            Biome::SnowyPlains
        } else if climate < -0.09 && moisture > -0.06 {
            Biome::Taiga
        } else if climate > 0.25 && moisture < -0.10 {
            Biome::Desert
        } else if moisture > 0.16 {
            Biome::Forest
        } else if moisture > 0.02 && climate > 0.02 {
            Biome::BirchForest
        } else {
            Biome::Plains
        };
        Column { height, biome }
    }

    fn generate(&self, (cx, cz): Chunk) -> GeneratedChunk {
        let (start_x, start_z) = (cx * 16, cz * 16);
        let columns: [Column; 256] = std::array::from_fn(|index| {
            self.column(start_x + (index & 15) as i32, start_z + (index >> 4) as i32)
        });
        let floor = columns.iter().map(|column| column.height).min().unwrap() - 5;
        let mut chunk = GeneratedChunk::new(
            std::array::from_fn(|index| {
                columns[(index / 4 * 4 + 2) * 16 + index % 4 * 4 + 2].biome
            }),
            std::array::from_fn(|index| columns[index].height.max(SEA_LEVEL) as i16),
            std::array::from_fn(|index| {
                let bottom = MIN_Y + index as i32 * 16;
                Section::Uniform(if bottom < 0 {
                    self.blocks.deepslate
                } else if bottom + 15 < floor {
                    self.blocks.stone
                } else {
                    AIR as u16
                })
            }),
        );
        let variable_start = (floor.max(0) >> 4) << 4;
        for (index, column) in columns.iter().enumerate() {
            let x = start_x + (index & 15) as i32;
            let z = start_z + (index >> 4) as i32;
            let random = hash(self.seed, x, z);
            for y in MIN_Y..MIN_Y + 1 + (random % 4) as i32 {
                chunk.set(BlockPos::new(x, y, z), self.blocks.bedrock);
            }
            let (top, below, depth) = match column.biome {
                Biome::Desert | Biome::Beach => (self.blocks.sand, self.blocks.sandstone, 4),
                Biome::Ocean | Biome::River => (self.blocks.gravel, self.blocks.gravel, 3),
                Biome::Hills if column.height > 122 => (self.blocks.stone, self.blocks.stone, 2),
                _ => (self.blocks.grass, self.blocks.dirt, 3),
            };
            for y in variable_start..=column.height {
                let depth_from_top = column.height - y;
                let state = match depth_from_top {
                    0 => top,
                    1 if matches!(column.biome, Biome::Desert | Biome::Beach) => top,
                    d if d <= depth => below,
                    _ => self.blocks.stone,
                };
                chunk.set(BlockPos::new(x, y, z), state);
            }
            for y in column.height + 1..=SEA_LEVEL {
                chunk.set(BlockPos::new(x, y, z), self.blocks.water);
            }
            if column.height > SEA_LEVEL {
                let plant = match column.biome {
                    Biome::SnowyPlains => self.blocks.snow,
                    Biome::Hills if column.height > 120 => self.blocks.snow,
                    Biome::Plains | Biome::Forest | Biome::BirchForest if random % 41 == 0 => {
                        self.blocks.flower
                    }
                    Biome::Plains | Biome::Forest | Biome::BirchForest | Biome::Taiga
                        if random % 9 == 0 =>
                    {
                        self.blocks.short_grass
                    }
                    _ => AIR as u16,
                };
                if plant != AIR as u16 {
                    chunk.set(BlockPos::new(x, column.height + 1, z), plant);
                }
            }
        }
        self.carve(&mut chunk, &columns, start_x, start_z);
        self.ore(&mut chunk, start_x, start_z);
        self.trees(&mut chunk, start_x, start_z);
        chunk
    }

    fn carve(&self, chunk: &mut GeneratedChunk, columns: &[Column; 256], x: i32, z: i32) {
        const STEP_Y: i32 = 8;
        const CAVE_MIN: i32 = -56;
        const LAYERS: usize = 18;
        let mut values = [[[0.0; 5]; 5]; LAYERS];
        for (dy, layer) in values.iter_mut().enumerate() {
            for (dz, row) in layer.iter_mut().enumerate() {
                for (dx, value) in row.iter_mut().enumerate() {
                    let px = x + dx as i32 * 4;
                    let pz = z + dz as i32 * 4;
                    let py = CAVE_MIN + dy as i32 * STEP_Y;
                    *value = self.caves.noise(
                        f64::from(px) / 33.0,
                        f64::from(py) / 24.0,
                        f64::from(pz) / 33.0,
                    );
                }
            }
        }
        for dy in 0..LAYERS - 1 {
            for dz in 0..4 {
                for dx in 0..4 {
                    let low = &values[dy];
                    let high = &values[dy + 1];
                    let samples = |layer: &[[f64; 5]; 5]| {
                        [
                            layer[dz][dx],
                            layer[dz][dx + 1],
                            layer[dz + 1][dx],
                            layer[dz + 1][dx + 1],
                        ]
                    };
                    let low = samples(low);
                    let high = samples(high);
                    if low.into_iter().chain(high).all(|value| value <= 0.35) {
                        continue;
                    }
                    for y in 0..STEP_Y {
                        let py = CAVE_MIN + dy as i32 * STEP_Y + y;
                        let row: [f64; 4] = std::array::from_fn(|i| {
                            lerp(low[i], high[i], f64::from(y) / f64::from(STEP_Y))
                        });
                        for z_local in 0..4 {
                            for x_local in 0..4 {
                                let px = dx * 4 + x_local;
                                let pz = dz * 4 + z_local;
                                let ground = columns[pz * 16 + px].height;
                                if py > ground - if ground > SEA_LEVEL + 8 { 2 } else { 6 } {
                                    continue;
                                }
                                let left = lerp(row[0], row[2], z_local as f64 / 4.0);
                                let right = lerp(row[1], row[3], z_local as f64 / 4.0);
                                if lerp(left, right, x_local as f64 / 4.0) > 0.35 {
                                    chunk.set(
                                        BlockPos::new(x + px as i32, py, z + pz as i32),
                                        AIR as u16,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn ore(&self, chunk: &mut GeneratedChunk, x: i32, z: i32) {
        let mut random = LegacyRandomSource::new(hash(self.seed ^ 0x513289, x, z) as i64);
        for (material, (attempts, min, max, length)) in [
            (6, 8, 118, 9),
            (8, -49, 80, 8),
            (6, -12, 72, 10),
            (3, -55, 24, 6),
            (2, -56, 9, 5),
        ]
        .into_iter()
        .enumerate()
        {
            for _ in 0..attempts {
                let mut pos = BlockPos::new(
                    random.next_int_bound(16),
                    min + random.next_int_bound(max - min),
                    random.next_int_bound(16),
                );
                for _ in 0..length {
                    if !(0..16).contains(&pos.x) || !(0..16).contains(&pos.z) {
                        break;
                    }
                    let block = BlockPos::new(x + pos.x, pos.y, z + pos.z);
                    let state = chunk.block(block) as u16;
                    let (stone, deep) = self.blocks.ores[material];
                    if state == self.blocks.stone {
                        chunk.set(block, stone);
                    } else if state == self.blocks.deepslate {
                        chunk.set(block, deep);
                    }
                    pos = pos.offset(
                        random.next_int_bound(3) - 1,
                        random.next_int_bound(3) - 1,
                        random.next_int_bound(3) - 1,
                    );
                }
            }
        }
    }

    fn trees(&self, chunk: &mut GeneratedChunk, x: i32, z: i32) {
        let contains = |px, pz| (x..x + 16).contains(&px) && (z..z + 16).contains(&pz);
        for gx in (x - 2).div_euclid(TREE_GRID)..=(x + 17).div_euclid(TREE_GRID) {
            for gz in (z - 2).div_euclid(TREE_GRID)..=(z + 17).div_euclid(TREE_GRID) {
                let random = hash(self.seed ^ 0x38cad, gx, gz);
                let px = gx * TREE_GRID + (random % TREE_GRID as u64) as i32;
                let pz = gz * TREE_GRID + ((random >> 8) % TREE_GRID as u64) as i32;
                let Column { height, biome } = self.column(px, pz);
                let roll = (random >> 16) % 12;
                if self.version >= 2
                    && self
                        .detail
                        .noise(f64::from(px) / 49.0, 0.0, f64::from(pz) / 49.0)
                        < -0.21
                {
                    continue;
                }
                let species = match biome {
                    Biome::Forest if roll < 10 => 0,
                    Biome::BirchForest if roll < 9 => 1,
                    Biome::Taiga if roll < 9 => 2,
                    Biome::SnowyPlains if roll < 2 => 2,
                    Biome::Plains if roll == 0 => 0,
                    _ => continue,
                };
                let trunk = 4 + ((random >> 24) & 3) as i32;
                let crown = height + trunk + 1;
                for y in crown - 2..=crown + 1 {
                    let radius: i32 = if y >= crown { 1 } else { 2 };
                    for dx in -radius..=radius {
                        for dz in -radius..=radius {
                            if !contains(px + dx, pz + dz)
                                || (y > crown - 2 && dx.abs() + dz.abs() > radius)
                            {
                                continue;
                            }
                            let pos = BlockPos::new(px + dx, y, pz + dz);
                            if chunk.block(pos) == AIR {
                                chunk.set(pos, self.blocks.leaves[species]);
                                let index = column(pos.x, pos.z);
                                chunk.height[index] = chunk.height[index].max(y as i16);
                            }
                        }
                    }
                }
                if contains(px, pz) {
                    for y in height + 1..=crown - 1 {
                        chunk.set(BlockPos::new(px, y, pz), self.blocks.logs[species]);
                    }
                    let index = column(px, pz);
                    chunk.height[index] = chunk.height[index].max((crown - 1) as i16);
                }
            }
        }
    }
}

fn smooth(value: f64) -> f64 {
    let fraction = value.clamp(0.0, 1.0);
    fraction * fraction * (3.0 - 2.0 * fraction)
}

fn lerp(a: f64, b: f64, fraction: f64) -> f64 {
    a + (b - a) * fraction
}

fn hash(seed: i64, x: i32, z: i32) -> u64 {
    let key = seed as u64
        ^ (x as u64).wrapping_mul(0x9e3779b97f4a7c15)
        ^ (z as u64).wrapping_mul(0xd1b54a32d192ed03);
    let key = (key ^ (key >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    let key = (key ^ (key >> 27)).wrapping_mul(0x94d049bb133111eb);
    key ^ (key >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn seeded_overworld_has_varied_land_water_biomes_caves_and_ores() {
        let generator = Overworld::new(322, TERRAIN_VERSION);
        let same = Overworld::new(322, TERRAIN_VERSION);
        let other = Overworld::new(323, TERRAIN_VERSION);
        let mut biomes = HashSet::new();
        let mut heights = HashSet::new();
        let mut differs = false;
        for x in (-1024..=1024).step_by(35) {
            for z in (-1024..=1024).step_by(35) {
                let col = generator.column(x, z);
                let repeated = same.column(x, z);
                assert_eq!((col.height, col.biome), (repeated.height, repeated.biome));
                let next = generator.column(x + 1, z);
                assert!((col.height - next.height).abs() <= 6);
                differs |= col.height != other.column(x, z).height;
                heights.insert(col.height);
                biomes.insert(col.biome as usize);
            }
        }
        assert!(differs);
        assert!(heights.len() > 45);
        assert!(biomes.len() >= 7);
        assert!(biomes.contains(&(Biome::Ocean as usize)));
        assert!(biomes.contains(&(Biome::River as usize)));

        let mut has_cave = false;
        let mut has_ore = false;
        for (x, z) in [(0, 0), (-1, 0), (-11, 6), (-4000, -8500)] {
            let terrain = generator.generate((x, z));
            for y in -54..=0 {
                for dx in 0..16 {
                    for dz in 0..16 {
                        let block =
                            terrain.block(BlockPos::new(x * 16 + dx, y, z * 16 + dz)) as u16;
                        has_cave |= block == AIR as u16;
                        has_ore |= generator
                            .blocks
                            .ores
                            .iter()
                            .any(|&(stone, deep)| block == stone || block == deep);
                    }
                }
            }
        }
        assert!(has_cave && has_ore);
    }
}
