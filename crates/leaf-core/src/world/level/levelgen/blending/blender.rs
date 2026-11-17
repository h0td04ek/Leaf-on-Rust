//! `net.minecraft.world.level.levelgen.blending.Blender` — блендинг
//! старой и новой генерации (offset/factor, density, biome resolver,
//! border ticks, carving mask).

use super::blending_data::{BlendingChunkAccess, BlendingData, BlendingRegion, ChunkGenStatus};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction8;
use crate::core::holder::Holder;
use crate::core::quart_pos;
use crate::core::section_pos::SectionPos;
use crate::util::mth;
use crate::world::level::biome::biome::Biome;
use crate::world::level::biome::biome_resolver::BiomeResolver;
use crate::world::level::biome::climate::Sampler;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::levelgen::density_function::{DensityFunction, FunctionContext};
use crate::world::level::levelgen::synth::normal_noise::NormalNoise;
use crate::world::level::levelgen::synth::normal_noise::NoiseParameters;
use crate::world::level::level_height_accessor::LevelHeightAccessor;
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;
use crate::random::xoroshiro::XoroshiroRandomSource;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

/// `HEIGHT_BLENDING_RANGE_CELLS`.
const HEIGHT_BLENDING_RANGE_CELLS: i32 = quart_pos::from_section(7) - 1;
/// `HEIGHT_BLENDING_RANGE_CHUNKS`.
const HEIGHT_BLENDING_RANGE_CHUNKS: i32 = quart_pos::to_section(HEIGHT_BLENDING_RANGE_CELLS + 3);
/// `DENSITY_BLENDING_RANGE_CELLS`/`CHUNKS`.
const DENSITY_BLENDING_RANGE_CELLS: i64 = 2;
const DENSITY_BLENDING_RANGE_CHUNKS: i32 = quart_pos::to_section(5);
/// `OLD_CHUNK_XZ_RADIUS`.
const _OLD_CHUNK_XZ_RADIUS: f64 = 8.0;

/// `SHIFT_NOISE` — `NormalNoise.create(XoroshiroRandomSource(42), DEFAULT_SHIFT)`.
fn shift_noise() -> &'static NormalNoise {
    static N: OnceLock<NormalNoise> = OnceLock::new();
    N.get_or_init(|| {
        let mut random = WorldgenRandom::new(Box::new(XoroshiroRandomSource::new(42)));
        NormalNoise::create_with_params(&mut random, NoiseParameters::default_shift())
    })
}

/// `Blender.BlendingOutput(alpha, blendingOffset)`.
#[derive(Clone, Copy, Debug)]
pub struct BlendingOutput {
    /// `alpha`.
    pub alpha: f64,
    /// `blendingOffset`.
    pub blending_offset: f64,
}

/// `Blender.DistanceGetter`.
pub trait DistanceGetter: Send + Sync {
    /// `getDistance(x, y, z)`.
    fn get_distance(&self, x: f64, y: f64, z: f64) -> f64;
}

/// `Blender.CellValueGetter`.
enum CellValueGetter {
    Height,
    Density,
}

impl CellValueGetter {
    fn get(&self, data: &BlendingData, x: i32, y: i32, z: i32) -> f64 {
        match self {
            CellValueGetter::Height => data.get_height(x, y, z),
            CellValueGetter::Density => data.get_density(x, y, z),
        }
    }
}

/// `Blender`.
pub struct Blender {
    height_and_biome_blending_data: HashMap<i64, Arc<BlendingData>>,
    density_blending_data: HashMap<i64, Arc<BlendingData>>,
    is_empty: bool,
}

impl Blender {
    /// `Blender.empty()`.
    pub fn empty() -> Blender {
        Blender { height_and_biome_blending_data: HashMap::new(), density_blending_data: HashMap::new(), is_empty: true }
    }

    /// `Blender.of(region)`.
    pub fn of(region: Option<&dyn BlendingRegion>) -> Blender {
        let Some(region) = region else { return Blender::empty() };
        let center = region.get_center();
        if !region.is_old_chunk_around(center, HEIGHT_BLENDING_RANGE_CHUNKS) {
            return Blender::empty();
        }
        let mut map = HashMap::new();
        let mut map1 = HashMap::new();
        let squared = mth::square_i32(HEIGHT_BLENDING_RANGE_CHUNKS + 1);
        for i in -HEIGHT_BLENDING_RANGE_CHUNKS..=HEIGHT_BLENDING_RANGE_CHUNKS {
            for i1 in -HEIGHT_BLENDING_RANGE_CHUNKS..=HEIGHT_BLENDING_RANGE_CHUNKS {
                if i * i + i1 * i1 <= squared {
                    let i2 = center.x + i;
                    let i3 = center.z + i1;
                    if let Some(data) = BlendingData::get_or_update_blending_data(region, i2, i3) {
                        let key = ChunkPos::as_long_static(i2, i3);
                        let arc = Arc::new(data);
                        map.insert(key, arc.clone());
                        if (-DENSITY_BLENDING_RANGE_CHUNKS..=DENSITY_BLENDING_RANGE_CHUNKS).contains(&(i as i32))
                            && (-DENSITY_BLENDING_RANGE_CHUNKS..=DENSITY_BLENDING_RANGE_CHUNKS).contains(&(i1 as i32))
                        {
                            map1.insert(key, arc);
                        }
                    }
                }
            }
        }
        if map.is_empty() && map1.is_empty() {
            Blender::empty()
        } else {
            Blender { height_and_biome_blending_data: map, density_blending_data: map1, is_empty: false }
        }
    }

    /// `new Blender(...)`.
    pub fn new(height_and_biome: HashMap<i64, Arc<BlendingData>>, density: HashMap<i64, Arc<BlendingData>>) -> Self {
        Self {
            is_empty: height_and_biome.is_empty() && density.is_empty(),
            height_and_biome_blending_data: height_and_biome,
            density_blending_data: density,
        }
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.is_empty
    }

    /// `blendOffsetAndFactor(x, z)`.
    pub fn blend_offset_and_factor(&self, x: i32, z: i32) -> BlendingOutput {
        let quart_pos_x = quart_pos::from_block(x);
        let quart_pos_z = quart_pos::from_block(z);
        let blending_data_value = self.get_blending_data_value(quart_pos_x, 0, quart_pos_z, &CellValueGetter::Height);
        if blending_data_value != f64::MAX {
            return BlendingOutput { alpha: 0.0, blending_offset: Self::height_to_offset(blending_data_value) };
        }
        let mut mutable_double = 0.0f64;
        let mut mutable_double1 = 0.0f64;
        let mut mutable_double2 = f64::INFINITY;
        for (l, blending_data) in &self.height_and_biome_blending_data {
            blending_data.iterate_heights(
                quart_pos::from_section(ChunkPos::of(*l).x),
                quart_pos::from_section(ChunkPos::of(*l).z),
                &mut |x1, z1, height| {
                    let d2 = mth::length_3d_f32(
                        (quart_pos_x - x1) as f32,
                        0.0,
                        (quart_pos_z - z1) as f32,
                    );
                    if d2 <= HEIGHT_BLENDING_RANGE_CELLS as f32 {
                        if (d2 as f64) < mutable_double2 {
                            mutable_double2 = d2 as f64;
                        }
                        let d3 = 1.0 / mth::square_f64(mth::square_f64(d2 as f64));
                        mutable_double1 += height * d3;
                        mutable_double += d3;
                    }
                },
            );
        }
        if mutable_double2 == f64::INFINITY {
            return BlendingOutput { alpha: 1.0, blending_offset: 0.0 };
        }
        let d = mutable_double1 / mutable_double;
        let mut d1 = mth::clamp_f64(mutable_double2 / (HEIGHT_BLENDING_RANGE_CELLS as f64 + 1.0), 0.0, 1.0);
        d1 = 3.0 * d1 * d1 - 2.0 * d1 * d1 * d1;
        BlendingOutput { alpha: d1, blending_offset: Self::height_to_offset(d) }
    }

    /// `heightToOffset`.
    fn height_to_offset(height: f64) -> f64 {
        let d = 1.0;
        let d1 = height + 0.5;
        let d2 = mth::positive_modulo_pos_den_f64(d1, 8.0);
        d * (32.0 * (d1 - 128.0) - 3.0 * (d1 - 120.0) * d2 + 3.0 * d2 * d2) / (128.0 * (32.0 - 3.0 * d2))
    }

    /// `blendDensity(context, density)`.
    pub fn blend_density(&self, context: &dyn FunctionContext, density: f64) -> f64 {
        let quart_pos_x = quart_pos::from_block(context.block_x());
        let i = context.block_y() / 8;
        let quart_pos_z = quart_pos::from_block(context.block_z());
        let blending_data_value = self.get_blending_data_value(quart_pos_x, i, quart_pos_z, &CellValueGetter::Density);
        if blending_data_value != f64::MAX {
            return blending_data_value;
        }
        let mut mutable_double = 0.0f64;
        let mut mutable_double1 = 0.0f64;
        let mut mutable_double2 = f64::INFINITY;
        for (l, blending_data) in &self.density_blending_data {
            blending_data.iterate_densities(
                quart_pos::from_section(ChunkPos::of(*l).x),
                quart_pos::from_section(ChunkPos::of(*l).z),
                i - 1,
                i + 1,
                &mut |x, y, z, density1| {
                    let len = mth::length_3d_f64((quart_pos_x - x) as f64, ((i - y) * 2) as f64, (quart_pos_z - z) as f64);
                    if len <= DENSITY_BLENDING_RANGE_CELLS as f64 {
                        if len < mutable_double2 {
                            mutable_double2 = len;
                        }
                        let d2 = 1.0 / mth::square_f64(mth::square_f64(len));
                        mutable_double1 += density1 * d2;
                        mutable_double += d2;
                    }
                },
            );
        }
        if mutable_double2 == f64::INFINITY {
            return density;
        }
        let d = mutable_double1 / mutable_double;
        let d1 = mth::clamp_f64(mutable_double2 / 3.0, 0.0, 1.0);
        mth::lerp_f64(d1, d, density)
    }

    /// `getBlendingDataValue(x, y, z, getter)` — с фолбэками соседних секций.
    fn get_blending_data_value(&self, x: i32, y: i32, z: i32, getter: &CellValueGetter) -> f64 {
        let section_pos_x = quart_pos::to_section(x);
        let section_pos_z = quart_pos::to_section(z);
        let flag = (x & 3) == 0;
        let flag1 = (z & 3) == 0;
        let mut value = self.get_blending_data_value_inner(getter, section_pos_x, section_pos_z, x, y, z);
        if value == f64::MAX {
            if flag && flag1 {
                value = self.get_blending_data_value_inner(getter, section_pos_x - 1, section_pos_z - 1, x, y, z);
            }
            if value == f64::MAX {
                if flag {
                    value = self.get_blending_data_value_inner(getter, section_pos_x - 1, section_pos_z, x, y, z);
                }
                if value == f64::MAX && flag1 {
                    value = self.get_blending_data_value_inner(getter, section_pos_x, section_pos_z - 1, x, y, z);
                }
            }
        }
        value
    }

    /// `getBlendingDataValue(getter, sectionX, sectionZ, x, y, z)`.
    fn get_blending_data_value_inner(&self, getter: &CellValueGetter, section_x: i32, section_z: i32, x: i32, y: i32, z: i32) -> f64 {
        let key = ChunkPos::as_long_static(section_x, section_z);
        if let Some(blending_data) = self.height_and_biome_blending_data.get(&key) {
            getter.get(blending_data, x - quart_pos::from_section(section_x), y, z - quart_pos::from_section(section_z))
        } else {
            f64::MAX
        }
    }

    /// `getBiomeResolver(resolver)`.
    pub fn get_biome_resolver<'a>(&'a self, resolver: &'a dyn BiomeResolver) -> impl Fn(i32, i32, i32, &Sampler) -> Holder<Biome> + 'a {
        move |x, y, z, sampler| {
            if let Some(holder) = self.blend_biome(x, y, z) {
                holder
            } else {
                resolver.get_noise_biome(x, y, z, sampler)
            }
        }
    }

    /// `blendBiome(x, y, z)`.
    fn blend_biome(&self, x: i32, y: i32, z: i32) -> Option<Holder<Biome>> {
        let mut mutable_double = f64::INFINITY;
        let mut mutable_object: Option<Holder<Biome>> = None;
        for (l, blending_data) in &self.height_and_biome_blending_data {
            blending_data.iterate_biomes(
                quart_pos::from_section(ChunkPos::of(*l).x),
                y,
                quart_pos::from_section(ChunkPos::of(*l).z),
                &mut |x1, z1, biome| {
                    let d2 = mth::length_3d_f32((x - x1) as f32, 0.0, (z - z1) as f32) as f64;
                    if d2 <= HEIGHT_BLENDING_RANGE_CELLS as f64 && d2 < mutable_double {
                        mutable_object = Some(biome);
                        mutable_double = d2;
                    }
                },
            );
        }
        if mutable_double == f64::INFINITY {
            return None;
        }
        let d = shift_noise().get_value(x as f64, 0.0, z as f64) * 12.0;
        let d1 = mth::clamp_f64((mutable_double + d) / (HEIGHT_BLENDING_RANGE_CELLS as f64 + 1.0), 0.0, 1.0);
        if d1 > 0.5 { None } else { mutable_object }
    }

    /// `generateBorderTicks(region, chunk)` — border-tick'и на границах
    /// старой/новой генерации. Требует `BlendingTickChunk`-шов.
    pub fn generate_border_ticks(region: &dyn BorderTickRegion, chunk: &dyn BorderTickChunk) {
        let pos = chunk.get_pos();
        let is_old_noise_generation = chunk.is_old_noise_generation();
        let blending_data = match chunk.get_blending_data() {
            Some(d) => d,
            None => return,
        };
        let min_y = blending_data.area_with_old_generation.get_min_y();
        let max_y = blending_data.area_with_old_generation.get_max_y();
        if is_old_noise_generation {
            for i in 0..16 {
                for i1 in 0..16 {
                    chunk.generate_border_tick(BlockPos::new(pos.get_min_block_x() + i, min_y - 1, pos.get_min_block_z() + i1));
                    chunk.generate_border_tick(BlockPos::new(pos.get_min_block_x() + i, min_y, pos.get_min_block_z() + i1));
                    chunk.generate_border_tick(BlockPos::new(pos.get_min_block_x() + i, max_y, pos.get_min_block_z() + i1));
                    chunk.generate_border_tick(BlockPos::new(pos.get_min_block_x() + i, max_y + 1, pos.get_min_block_z() + i1));
                }
            }
        }
        for direction in crate::core::direction::Plane::HORIZONTAL.faces() {
            let step_x = direction.get_step_x();
            let step_z = direction.get_step_z();
            let neighbor = region.get_chunk(pos.x + step_x, pos.z + step_z);
            if neighbor.is_old_noise_generation() != is_old_noise_generation {
                let (i2, i3, i4, i5) = match direction {
                    crate::core::direction::Direction::EAST => (15, 0, 0, 15),
                    crate::core::direction::Direction::WEST => (0, 15, 0, 15),
                    crate::core::direction::Direction::SOUTH => (0, 15, 15, 15),
                    crate::core::direction::Direction::NORTH => (0, 15, 0, 0),
                    _ => (0, 15, 0, 15),
                };
                for i6 in i2..=i3 {
                    for i7 in i4..=i5 {
                        let i8 = max_y.min(chunk.get_height(crate::world::level::levelgen::heightmap::HeightmapType::MotionBlocking, i6, i7)) + 1;
                        for i9 in min_y..i8 {
                            chunk.generate_border_tick(BlockPos::new(pos.get_min_block_x() + i6, i9, pos.get_min_block_z() + i7));
                        }
                    }
                }
            }
        }
    }

    /// `addAroundOldChunksCarvingMaskFilter(level, chunk)`.
    pub fn add_around_old_chunks_carving_mask_filter(region: &dyn BorderTickRegion, chunk: &dyn CarvingMaskChunk) {
        let pos = chunk.get_pos();
        let mut surrounding: HashMap<Direction8, Arc<BlendingData>> = HashMap::new();
        for direction8 in Direction8::VALUES {
            let i = pos.x + direction8.get_step_x();
            let i1 = pos.z + direction8.get_step_z();
            if let Some(data) = region.get_chunk(i, i1).get_blending_data_arc() {
                surrounding.insert(direction8, data);
            }
        }
        if chunk.is_old_noise_generation() || !surrounding.is_empty() {
            let own = chunk.get_blending_data_arc();
            let distance_getter = Self::make_old_chunk_distance_getter(own.as_deref(), &surrounding);
            let mask = move |x: i32, y: i32, z: i32| -> bool {
                let d = x as f64 + 0.5 + shift_noise().get_value(x as f64, y as f64, z as f64) * 4.0;
                let d1 = y as f64 + 0.5 + shift_noise().get_value(y as f64, z as f64, x as f64) * 4.0;
                let d2 = z as f64 + 0.5 + shift_noise().get_value(z as f64, x as f64, y as f64) * 4.0;
                distance_getter.get_distance(d, d1, d2) < 4.0
            };
            chunk.get_or_create_carving_mask().set_additional_mask(Box::new(mask));
        }
    }

    /// `makeOldChunkDistanceGetter`.
    pub fn make_old_chunk_distance_getter(
        blending_data: Option<&BlendingData>,
        surrounding_blending_data: &HashMap<Direction8, Arc<BlendingData>>,
    ) -> Arc<dyn DistanceGetter> {
        let mut list: Vec<Arc<dyn DistanceGetter>> = Vec::new();
        if let Some(data) = blending_data {
            list.push(Self::make_offset_old_chunk_distance_getter(None, data));
        }
        for (direction8, data) in surrounding_blending_data {
            list.push(Self::make_offset_old_chunk_distance_getter(Some(*direction8), data));
        }
        Arc::new(MoveDistanceGetter { getters: list })
    }

    /// `makeOffsetOldChunkDistanceGetter`.
    fn make_offset_old_chunk_distance_getter(direction: Option<Direction8>, blending_data: &BlendingData) -> Arc<dyn DistanceGetter> {
        let mut d = 0.0f64;
        let mut d1 = 0.0f64;
        if let Some(direction) = direction {
            for direction1 in direction.get_directions() {
                d += direction1.get_step_x() as f64 * 16.0;
                d1 += direction1.get_step_z() as f64 * 16.0;
            }
        }
        let d2 = d;
        let d3 = d1;
        let d4 = blending_data.area_with_old_generation.get_height() as f64 / 2.0;
        let d5 = blending_data.area_with_old_generation.get_min_y() as f64 + d4;
        Arc::new(MoveDistanceGetter {
            getters: vec![Arc::new(OffsetDistanceGetter { d2, d3, d5, d4 })],
        })
    }

    /// `distanceToCube`.
    fn distance_to_cube(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> f64 {
        let d = x1.abs() - x2;
        let d1 = y1.abs() - y2;
        let d2 = z1.abs() - z2;
        mth::length_3d_f64(d.max(0.0), d1.max(0.0), d2.max(0.0))
    }
}

struct MoveDistanceGetter {
    getters: Vec<Arc<dyn DistanceGetter>>,
}

impl DistanceGetter for MoveDistanceGetter {
    fn get_distance(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut d = f64::INFINITY;
        for g in &self.getters {
            d = d.min(g.get_distance(x, y, z));
        }
        d
    }
}

struct OffsetDistanceGetter {
    d2: f64,
    d3: f64,
    d5: f64,
    d4: f64,
}

impl DistanceGetter for OffsetDistanceGetter {
    fn get_distance(&self, z: f64, d6: f64, d7: f64) -> f64 {
        Blender::distance_to_cube(z - 8.0 - self.d2, d6 - self.d5, d7 - 8.0 - self.d3, 8.0, self.d4, 8.0)
    }
}

/// `generateBorderTick` — tick для leaves/fluids.
fn _generate_border_tick_marker() {}

/// Шов для BorderTick-региона (WorldGenRegion).
pub trait BorderTickRegion {
    /// `getChunk(x, z)` — chunk с old-noise-info.
    fn get_chunk(&self, x: i32, z: i32) -> Box<dyn BorderTickChunk + '_>;
}

/// Шов для chunk в border-tick'ах.
pub trait BorderTickChunk {
    /// `getPos`.
    fn get_pos(&self) -> ChunkPos;
    /// `isOldNoiseGeneration`.
    fn is_old_noise_generation(&self) -> bool;
    /// `getBlendingData`.
    fn get_blending_data(&self) -> Option<BlendingData>;
    /// `getBlendingData` как Arc (для carving mask).
    fn get_blending_data_arc(&self) -> Option<Arc<BlendingData>>;
    /// `getHeight(type, x, z)`.
    fn get_height(&self, ty: crate::world::level::levelgen::heightmap::HeightmapType, x: i32, z: i32) -> i32;
    /// `generateBorderTick(pos)` — markPosForPostprocessing для leaves/fluids.
    fn generate_border_tick(&self, pos: crate::core::block_pos::BlockPos);
}

/// Шов для chunk с carving mask.
pub trait CarvingMaskChunk {
    /// `getPos`.
    fn get_pos(&self) -> ChunkPos;
    /// `isOldNoiseGeneration`.
    fn is_old_noise_generation(&self) -> bool;
    /// `getBlendingData` (Arc).
    fn get_blending_data_arc(&self) -> Option<Arc<BlendingData>>;
    /// `getOrCreateCarvingMask`.
    fn get_or_create_carving_mask(&self) -> &dyn CarvingMaskAccess;
}

/// `CarvingMask`-шов.
pub trait CarvingMaskAccess {
    /// `setAdditionalMask(mask)`.
    fn set_additional_mask(&self, mask: Box<dyn Fn(i32, i32, i32) -> bool + Send + Sync>);
}

// SectionPos импорт нужен для Blender::of-логики (from_section уже через quart_pos).
#[allow(unused_imports)]
use SectionPos as _SectionPosUsed;
