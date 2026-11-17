//! `net.minecraft.world.level.levelgen.blending.BlendingData` —
//! данные блендинга старых и новых чанков (высоты, биомы, плотности).

use super::super::density_function::DensityFunction;
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction8;
use crate::core::holder::Holder;
use crate::core::quart_pos;
use crate::core::section_pos::SectionPos;
use crate::tags::{is_block_in_named_tag, BlockTags};
use crate::util::mth;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::biome::biome::Biome;
use crate::world::level::level_height_accessor::{LevelHeightAccessor, SimpleLevelHeightAccessor};
use crate::world::level::levelgen::heightmap::HeightmapType;
use std::collections::HashSet;
use std::sync::Arc;

/// `BLENDING_DENSITY_FACTOR`.
const BLENDING_DENSITY_FACTOR: f64 = 0.1;
/// `CELL_WIDTH`/`CELL_HEIGHT`/`CELL_RATIO`.
const CELL_WIDTH: i32 = 4;
const CELL_HEIGHT: i32 = 8;
const CELL_RATIO: i32 = 2;
const SOLID_DENSITY: f64 = 1.0;
const AIR_DENSITY: f64 = -1.0;
const CELLS_PER_SECTION_Y: i32 = 2;
const QUARTS_PER_SECTION: i32 = 4; // quart_pos::from_block(16)
const CELL_HORIZONTAL_MAX_INDEX_INSIDE: i32 = QUARTS_PER_SECTION - 1;
const CELL_HORIZONTAL_MAX_INDEX_OUTSIDE: i32 = QUARTS_PER_SECTION;
const CELL_COLUMN_INSIDE_COUNT: i32 = 2 * CELL_HORIZONTAL_MAX_INDEX_INSIDE + 1;
const CELL_COLUMN_OUTSIDE_COUNT: i32 = 2 * CELL_HORIZONTAL_MAX_INDEX_OUTSIDE + 1;
/// `CELL_COLUMN_COUNT`.
pub const CELL_COLUMN_COUNT: usize = (CELL_COLUMN_INSIDE_COUNT + CELL_COLUMN_OUTSIDE_COUNT) as usize;
/// `NO_VALUE`.
pub const NO_VALUE: f64 = f64::MAX;

/// `SURFACE_BLOCKS` — блоки поверхности для определения высоты.
fn is_surface_block(state: &BlockState) -> bool {
    const SURFACE: [&str; 12] = [
        "podzol", "gravel", "grass_block", "stone", "coarse_dirt", "sand",
        "red_sand", "mycelium", "snow_block", "terracotta", "dirt", "podzol",
    ];
    let path = state.block.id.get_path();
    SURFACE.iter().any(|s| path == *s)
}

/// `BlendingData` — данные блендинга для одной области старой генерации.
pub struct BlendingData {
    /// `areaWithOldGeneration`.
    pub area_with_old_generation: SimpleLevelHeightAccessor,
    has_calculated_data: bool,
    heights: [f64; CELL_COLUMN_COUNT],
    biomes: Vec<Option<Vec<Option<Holder<Biome>>>>>,
    densities: Vec<Option<Vec<f64>>>,
}

/// `BlendingData.Packed` — сериализованная форма (NBT/codec — при порте).
#[derive(Clone, Debug)]
pub struct PackedBlendingData {
    /// `minSection`.
    pub min_section: i32,
    /// `maxSection`.
    pub max_section: i32,
    /// `heights` (опционально).
    pub heights: Option<Vec<f64>>,
}

impl BlendingData {
    /// `new BlendingData(minSection, maxSection, heights)`.
    fn new(min_section: i32, max_section: i32, heights: Option<Vec<f64>>) -> Self {
        let block_pos_x = SectionPos::section_to_block_coord(min_section);
        let i = SectionPos::section_to_block_coord(max_section) - block_pos_x;
        let mut heights_arr = [NO_VALUE; CELL_COLUMN_COUNT];
        if let Some(h) = heights {
            for (dst, src) in heights_arr.iter_mut().zip(h.iter()) {
                *dst = *src;
            }
        }
        Self {
            area_with_old_generation: SimpleLevelHeightAccessor::create(block_pos_x, i),
            has_calculated_data: false,
            heights: heights_arr,
            biomes: vec![None; CELL_COLUMN_COUNT],
            densities: vec![None; CELL_COLUMN_COUNT],
        }
    }

    /// `unpack(packed)`.
    pub fn unpack(packed: Option<&PackedBlendingData>) -> Option<Self> {
        packed.map(|p| Self::new(p.min_section, p.max_section, p.heights.clone()))
    }

    /// `pack()`.
    pub fn pack(&self) -> PackedBlendingData {
        let has_values = self.heights.iter().any(|&d| d != NO_VALUE);
        PackedBlendingData {
            min_section: self.area_with_old_generation.get_min_section_y(),
            max_section: self.area_with_old_generation.get_max_section_y() + 1,
            heights: if has_values { Some(self.heights.to_vec()) } else { None },
        }
    }

    /// `getOrUpdateBlendingData(region, chunkX, chunkZ)`.
    pub fn get_or_update_blending_data(region: &dyn BlendingRegion, chunk_x: i32, chunk_z: i32) -> Option<Self> {
        let chunk = region.get_chunk(chunk_x, chunk_z);
        if let Some(blending_data) = chunk.get_blending_data() {
            if !chunk.get_highest_generated_status().is_before_biomes() {
                let mut data = blending_data.clone_shallow();
                data.calculate_data(&*chunk, &Self::side_by_generation_age(region, chunk_x, chunk_z, false));
                return Some(data);
            }
        }
        None
    }

    /// `sideByGenerationAge(level, chunkX, chunkZ, oldNoiseGeneration)`.
    pub fn side_by_generation_age(level: &dyn BlendingRegion, chunk_x: i32, chunk_z: i32, old_noise_generation: bool) -> HashSet<Direction8> {
        let mut set = HashSet::new();
        for direction8 in Direction8::VALUES {
            let i = chunk_x + direction8.get_step_x();
            let i1 = chunk_z + direction8.get_step_z();
            if level.get_chunk(i, i1).is_old_noise_generation() == old_noise_generation {
                set.insert(direction8);
            }
        }
        set
    }

    /// `calculateData(chunk, directions)` — заполнение высот/плотностей/биомов.
    pub fn calculate_data(&mut self, chunk: &dyn BlendingChunkAccess, directions: &HashSet<Direction8>) {
        if self.has_calculated_data {
            return;
        }
        if directions.contains(&Direction8::North) || directions.contains(&Direction8::West) || directions.contains(&Direction8::NorthWest) {
            self.add_values_for_column(Self::get_inside_index(0, 0), chunk, 0, 0);
        }
        if directions.contains(&Direction8::North) {
            for i in 1..QUARTS_PER_SECTION {
                self.add_values_for_column(Self::get_inside_index(i, 0), chunk, 4 * i, 0);
            }
        }
        if directions.contains(&Direction8::West) {
            for i in 1..QUARTS_PER_SECTION {
                self.add_values_for_column(Self::get_inside_index(0, i), chunk, 0, 4 * i);
            }
        }
        if directions.contains(&Direction8::East) {
            for i in 1..QUARTS_PER_SECTION {
                self.add_values_for_column(Self::get_outside_index(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE, i), chunk, 15, 4 * i);
            }
        }
        if directions.contains(&Direction8::South) {
            for i in 0..QUARTS_PER_SECTION {
                self.add_values_for_column(Self::get_outside_index(i, CELL_HORIZONTAL_MAX_INDEX_OUTSIDE), chunk, 4 * i, 15);
            }
        }
        if directions.contains(&Direction8::East) && directions.contains(&Direction8::NorthEast) {
            self.add_values_for_column(Self::get_outside_index(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE, 0), chunk, 15, 0);
        }
        if directions.contains(&Direction8::East)
            && directions.contains(&Direction8::South)
            && directions.contains(&Direction8::SouthEast)
        {
            self.add_values_for_column(
                Self::get_outside_index(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE, CELL_HORIZONTAL_MAX_INDEX_OUTSIDE),
                chunk,
                15,
                15,
            );
        }
        self.has_calculated_data = true;
    }

    /// `addValuesForColumn`.
    fn add_values_for_column(&mut self, index: usize, chunk: &dyn BlendingChunkAccess, x: i32, z: i32) {
        if self.heights[index] == NO_VALUE {
            self.heights[index] = self.get_height_at_xz(chunk, x, z) as f64;
        }
        self.densities[index] = Some(self.get_density_column(chunk, x, z, mth::floor(self.heights[index])));
        self.biomes[index] = Some(self.get_biome_column(chunk, x, z));
    }

    /// `getHeightAtXZ`.
    fn get_height_at_xz(&self, chunk: &dyn BlendingChunkAccess, x: i32, z: i32) -> i32 {
        let min;
        if chunk.has_primed_heightmap(HeightmapType::WorldSurfaceWg) {
            min = chunk.get_height(HeightmapType::WorldSurfaceWg, x, z).min(self.area_with_old_generation.get_max_y());
        } else {
            min = self.area_with_old_generation.get_max_y();
        }
        let min_y = self.area_with_old_generation.get_min_y();
        let mut y = min;
        while y > min_y {
            let pos = BlockPos::new(x, y, z);
            if let Some(state) = chunk.get_block_state(pos) {
                if is_surface_block(&state) {
                    return y;
                }
            }
            y -= 1;
        }
        min_y
    }

    /// `read1` — плотность одного блока.
    fn read1(chunk: &dyn BlendingChunkAccess, pos: BlockPos) -> f64 {
        if Self::is_ground(chunk, pos.below()) { SOLID_DENSITY } else { AIR_DENSITY }
    }

    /// `read7` — сумма 7 значений read1.
    fn read7(chunk: &dyn BlendingChunkAccess, mut pos: BlockPos) -> f64 {
        let mut d = 0.0;
        for _ in 0..7 {
            d += Self::read1(chunk, pos);
            pos = pos.below();
        }
        d
    }

    /// `getDensityColumn`.
    fn get_density_column(&self, chunk: &dyn BlendingChunkAccess, x: i32, z: i32, height: i32) -> Vec<f64> {
        let mut doubles = vec![AIR_DENSITY; self.cell_count_per_column() as usize];
        let mut pos = BlockPos::new(x, self.area_with_old_generation.get_max_y() + 1, z);
        let mut d = Self::read7(chunk, pos);
        for i in (0..doubles.len() - 1).rev() {
            let d1 = Self::read1(chunk, pos);
            let d2 = Self::read7(chunk, pos);
            doubles[i] = (d + d1 + d2) / 15.0;
            d = d2;
            pos = pos.below_n(CELL_HEIGHT);
        }
        let i = self.get_cell_y_index(mth::floor_div(height, CELL_HEIGHT));
        if i >= 0 && (i as usize) < doubles.len() - 1 {
            let d1 = (height as f64 + 0.5) % CELL_HEIGHT as f64 / CELL_HEIGHT as f64;
            let d2 = (1.0 - d1) / d1;
            let d3 = d2.max(1.0) * 0.25;
            doubles[i as usize + 1] = -d2 / d3;
            doubles[i as usize] = 1.0 / d3;
        }
        doubles
    }

    /// `getBiomeColumn`.
    fn get_biome_column(&self, chunk: &dyn BlendingChunkAccess, x: i32, z: i32) -> Vec<Option<Holder<Biome>>> {
        let count = self.quart_count_per_column() as usize;
        let mut list: Vec<Option<Holder<Biome>>> = vec![None; count];
        for i in 0..count {
            let i1 = i as i32 + quart_pos::from_block(self.area_with_old_generation.get_min_y());
            list[i] = Some(chunk.get_noise_biome(quart_pos::from_block(x), i1, quart_pos::from_block(z)));
        }
        list
    }

    /// `isGround`.
    fn is_ground(chunk: &dyn BlendingChunkAccess, pos: BlockPos) -> bool {
        let Some(state) = chunk.get_block_state(pos) else { return false };
        !state.is_air()
            && !is_block_in_named_tag(&state.block.id, &BlockTags::leaves())
            && !is_block_in_named_tag(&state.block.id, &BlockTags::logs())
            && state.block.id.get_path() != "brown_mushroom_block"
            && state.block.id.get_path() != "red_mushroom_block"
            && !chunk.collision_shape_is_empty(&state, pos)
    }

    /// `getHeight(x, y, z)` — высота поверхности (quart-координаты колонок).
    pub fn get_height(&self, x: i32, y: i32, z: i32) -> f64 {
        let _ = y;
        if x == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE || z == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE {
            self.heights[Self::get_outside_index(x, z)]
        } else if x != 0 && z != 0 {
            NO_VALUE
        } else {
            self.heights[Self::get_inside_index(x, z)]
        }
    }

    /// `getDensity(heights, y)` — приватный helper.
    fn get_density_column_value(heights: Option<&[f64]>, y: i32, column_min_y: i32, cell_count: i32) -> f64 {
        let Some(heights) = heights else { return NO_VALUE };
        let cell_y_index = y - column_min_y;
        if cell_y_index >= 0 && cell_y_index < cell_count {
            heights.get(cell_y_index as usize).copied().unwrap_or(NO_VALUE) * BLENDING_DENSITY_FACTOR
        } else {
            NO_VALUE
        }
    }

    /// `getDensity(x, y, z)`.
    pub fn get_density(&self, x: i32, y: i32, z: i32) -> f64 {
        if y == self.get_min_y() {
            return BLENDING_DENSITY_FACTOR;
        }
        if x == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE || z == CELL_HORIZONTAL_MAX_INDEX_OUTSIDE {
            let idx = Self::get_outside_index(x, z);
            return Self::get_density_column_value(self.densities[idx].as_deref(), y, self.get_column_min_y(), self.cell_count_per_column());
        }
        if x != 0 && z != 0 {
            return NO_VALUE;
        }
        let idx = Self::get_inside_index(x, z);
        Self::get_density_column_value(self.densities[idx].as_deref(), y, self.get_column_min_y(), self.cell_count_per_column())
    }

    /// `iterateBiomes`.
    pub fn iterate_biomes(&self, x: i32, y: i32, z: i32, consumer: &mut dyn FnMut(i32, i32, Holder<Biome>)) {
        if y >= quart_pos::from_block(self.area_with_old_generation.get_min_y())
            && y <= quart_pos::from_block(self.area_with_old_generation.get_max_y())
        {
            let i = y - quart_pos::from_block(self.area_with_old_generation.get_min_y());
            for i1 in 0..self.biomes.len() {
                if let Some(list) = &self.biomes[i1] {
                    if let Some(Some(holder)) = list.get(i as usize) {
                        consumer(x + Self::get_x(i1 as i32), z + Self::get_z(i1 as i32), holder.clone());
                    }
                }
            }
        }
    }

    /// `iterateHeights`.
    pub fn iterate_heights(&self, x: i32, z: i32, consumer: &mut dyn FnMut(i32, i32, f64)) {
        for (i, &d) in self.heights.iter().enumerate() {
            if d != NO_VALUE {
                consumer(x + Self::get_x(i as i32), z + Self::get_z(i as i32), d);
            }
        }
    }

    /// `iterateDensities`.
    pub fn iterate_densities(&self, x: i32, z: i32, min_y: i32, max_y: i32, consumer: &mut dyn FnMut(i32, i32, i32, f64)) {
        let column_min_y = self.get_column_min_y();
        let max = (min_y - column_min_y).max(0);
        let min = (max_y - column_min_y).min(self.cell_count_per_column());
        for (i, doubles) in self.densities.iter().enumerate() {
            if let Some(doubles) = doubles {
                let i1 = x + Self::get_x(i as i32);
                let i2 = z + Self::get_z(i as i32);
                for i3 in max..min {
                    if let Some(&d) = doubles.get(i3 as usize) {
                        consumer(i1, i3 + column_min_y, i2, d * BLENDING_DENSITY_FACTOR);
                    }
                }
            }
        }
    }

    /// `cellCountPerColumn`.
    fn cell_count_per_column(&self) -> i32 {
        self.area_with_old_generation.get_sections_count() * CELLS_PER_SECTION_Y
    }

    /// `quartCountPerColumn`.
    fn quart_count_per_column(&self) -> i32 {
        quart_pos::from_section(self.area_with_old_generation.get_sections_count())
    }

    /// `getColumnMinY`.
    fn get_column_min_y(&self) -> i32 {
        self.get_min_y() + 1
    }

    /// `getMinY`.
    fn get_min_y(&self) -> i32 {
        self.area_with_old_generation.get_min_section_y() * CELLS_PER_SECTION_Y
    }

    /// `getCellYIndex`.
    fn get_cell_y_index(&self, y: i32) -> i32 {
        y - self.get_column_min_y()
    }

    /// `getInsideIndex`.
    fn get_inside_index(x: i32, z: i32) -> usize {
        (CELL_HORIZONTAL_MAX_INDEX_INSIDE - x + z) as usize
    }

    /// `getOutsideIndex`.
    fn get_outside_index(x: i32, z: i32) -> usize {
        (CELL_COLUMN_INSIDE_COUNT + x + CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - z) as usize
    }

    /// `getX(index)`.
    fn get_x(index: i32) -> i32 {
        if index < CELL_COLUMN_INSIDE_COUNT {
            Self::zero_if_negative(CELL_HORIZONTAL_MAX_INDEX_INSIDE - index)
        } else {
            let i = index - CELL_COLUMN_INSIDE_COUNT;
            CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - Self::zero_if_negative(CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - i)
        }
    }

    /// `getZ(index)`.
    fn get_z(index: i32) -> i32 {
        if index < CELL_COLUMN_INSIDE_COUNT {
            Self::zero_if_negative(index - CELL_HORIZONTAL_MAX_INDEX_INSIDE)
        } else {
            let i = index - CELL_COLUMN_INSIDE_COUNT;
            CELL_HORIZONTAL_MAX_INDEX_OUTSIDE - Self::zero_if_negative(i - CELL_HORIZONTAL_MAX_INDEX_OUTSIDE)
        }
    }

    /// `zeroIfNegative`.
    fn zero_if_negative(value: i32) -> i32 {
        value & !(value >> 31)
    }

    /// `clone_shallow` — копия данных для getOrUpdate (calculateData
    /// мутирует только пустые слоты; hasCalculatedData сбрасывается).
    pub fn clone_shallow(&self) -> Self {
        Self {
            area_with_old_generation: SimpleLevelHeightAccessor::create(
                self.area_with_old_generation.get_min_y(),
                self.area_with_old_generation.get_height(),
            ),
            has_calculated_data: false,
            heights: self.heights,
            biomes: self.biomes.clone(),
            densities: self.densities.clone(),
        }
    }
}

/// `ChunkAccess`-шов для BlendingData (полный ChunkAccess портируется
/// в `world/level/chunk/`; до этого — этот trait).
pub trait BlendingChunkAccess {
    /// `hasPrimedHeightmap(type)`.
    fn has_primed_heightmap(&self, ty: HeightmapType) -> bool;
    /// `getHeight(type, x, z)`.
    fn get_height(&self, ty: HeightmapType, x: i32, z: i32) -> i32;
    /// `getBlockState(pos)`.
    fn get_block_state(&self, pos: BlockPos) -> Option<BlockState>;
    /// `getNoiseBiome(x, y, z)` — quart-координаты.
    fn get_noise_biome(&self, x: i32, y: i32, z: i32) -> Holder<Biome>;
    /// `getCollisionShape(state, pos).isEmpty()`.
    fn collision_shape_is_empty(&self, state: &BlockState, pos: BlockPos) -> bool;
    /// `getBlendingData()`.
    fn get_blending_data(&self) -> Option<BlendingData>;
    /// `getHighestGeneratedStatus().isBefore(BIOMES)`.
    fn get_highest_generated_status(&self) -> ChunkGenStatus;
    /// `isOldNoiseGeneration`.
    fn is_old_noise_generation(&self) -> bool;
}

/// `ChunkStatus`-подобный статус генерации (шов).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChunkGenStatus {
    /// `EMPTY`.
    Empty,
    /// `STRUCTURE_STARTS`.
    StructureStarts,
    /// `STRUCTURE_REFERENCES`.
    StructureReferences,
    /// `BIOMES`.
    Biomes,
    /// `NOISE`.
    Noise,
    /// `SURFACE`.
    Surface,
    /// `CARVERS`.
    Carvers,
    /// `FEATURES`.
    Features,
    /// `INITIALIZE_LIGHT`.
    InitializeLight,
    /// `LIGHT`.
    Light,
    /// `SPAWN`.
    Spawn,
    /// `FULL`.
    Full,
}

impl ChunkGenStatus {
    /// `isBefore(other)`.
    pub fn is_before(&self, other: ChunkGenStatus) -> bool {
        *self < other
    }
    /// `isBefore(BIOMES)`.
    pub fn is_before_biomes(&self) -> bool {
        self.is_before(ChunkGenStatus::Biomes)
    }
}

/// `WorldGenRegion`-шов для Blender::of / sideByGenerationAge.
pub trait BlendingRegion {
    /// `getCenter()`.
    fn get_center(&self) -> crate::world::level::chunk_pos::ChunkPos;
    /// `isOldChunkAround(center, chunks)`.
    fn is_old_chunk_around(&self, center: crate::world::level::chunk_pos::ChunkPos, chunks: i32) -> bool;
    /// `getChunk(x, z)` — chunk view.
    fn get_chunk(&self, x: i32, z: i32) -> Box<dyn BlendingChunkAccess + '_>;
}

// Подавление warnings за неиспользуемый импорт DensityFunction (нужен для сигнатур Blender).
#[allow(unused_imports)]
use DensityFunction as _DensityFunctionUsed;
