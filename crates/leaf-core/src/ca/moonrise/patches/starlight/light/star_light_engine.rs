//! `ca.spottedleaf.moonrise.patches.starlight.light.StarLightEngine` —
//! базовый движок StarLight: кэши 5×5×N секций, BFS-распространение
//! света через increase/decrease очереди.
//!
//! Портируется полностью, включая специфичную для sky/block логику —
//! абстрактные методы Java реализованы как диспетчеризация по
//! [`EngineKind`] в `sky_star_light_engine`/`block_star_light_engine`.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::ca::moonrise::common::util::world_util::WorldUtil;
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::section_pos::SectionPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::level_chunk::ChunkStatus;
use crate::world::level::chunk::level_chunk_section::LevelChunkSection;
use crate::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::light_layer::LightLayer;
use crate::world::phys::shapes::voxel_shape::VoxelShape;

use super::swmr_nibble_array::{get_filled_empty_light_vec, EmptinessMapRef, NibbleArrayRef};
use super::{block_star_light_engine, sky_star_light_engine};

/// `AxisDirection` — 6 направлений распространения света
/// (порядок объявления важен: он кодируется в битмасках очередей).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AxisDirection {
    /// `POSITIVE_X`.
    PositiveX,
    /// `NEGATIVE_X`.
    NegativeX,
    /// `POSITIVE_Z`.
    PositiveZ,
    /// `NEGATIVE_Z`.
    NegativeZ,
    /// `POSITIVE_Y`.
    PositiveY,
    /// `NEGATIVE_Y`.
    NegativeY,
}

impl AxisDirection {
    /// Все направления в порядке объявления.
    pub const VALUES: [AxisDirection; 6] = [
        AxisDirection::PositiveX,
        AxisDirection::NegativeX,
        AxisDirection::PositiveZ,
        AxisDirection::NegativeZ,
        AxisDirection::PositiveY,
        AxisDirection::NegativeY,
    ];

    /// Только горизонтальные.
    pub const ONLY_HORIZONTAL: [AxisDirection; 4] = [
        AxisDirection::PositiveX,
        AxisDirection::NegativeX,
        AxisDirection::PositiveZ,
        AxisDirection::NegativeZ,
    ];

    pub const ALL_DIRECTIONS_BITSET: i64 = (1 << 6) - 1;

    pub(crate) fn idx(self) -> usize {
        self as usize
    }

    pub fn x(self) -> i32 {
        match self {
            AxisDirection::PositiveX => 1,
            AxisDirection::NegativeX => -1,
            _ => 0,
        }
    }

    pub fn y(self) -> i32 {
        match self {
            AxisDirection::PositiveY => 1,
            AxisDirection::NegativeY => -1,
            _ => 0,
        }
    }

    pub fn z(self) -> i32 {
        match self {
            AxisDirection::PositiveZ => 1,
            AxisDirection::NegativeZ => -1,
            _ => 0,
        }
    }

    /// `nms` — соответствующее `net.minecraft.core.Direction`.
    pub(crate) fn nms(self) -> Direction {
        match self {
            AxisDirection::PositiveX => Direction::EAST,
            AxisDirection::NegativeX => Direction::WEST,
            AxisDirection::PositiveZ => Direction::SOUTH,
            AxisDirection::NegativeZ => Direction::NORTH,
            AxisDirection::PositiveY => Direction::UP,
            AxisDirection::NegativeY => Direction::DOWN,
        }
    }

    /// `getOpposite` — положительные направления имеют чётный ordinal,
    /// отрицательные — нечётный; opposite = ordinal ^ 1.
    pub fn get_opposite(self) -> AxisDirection {
        Self::VALUES[self.idx() ^ 1]
    }

    /// `everythingButThisDirection` — битмаска всех направлений, кроме этого.
    pub fn everything_but_this_direction(self) -> i64 {
        (Self::ALL_DIRECTIONS_BITSET as i64) ^ (1i64 << self.idx())
    }

    /// `everythingButTheOppositeDirection`.
    pub fn everything_but_opposite_direction(self) -> i64 {
        (Self::ALL_DIRECTIONS_BITSET as i64) ^ (1i64 << (self.idx() ^ 1))
    }
}

/// Тип движка: sky- или block-light.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EngineKind {
    /// `SkyStarLightEngine`.
    Sky,
    /// `BlockStarLightEngine`.
    Block,
}

/// `FLAG_WRITE_LEVEL = Long.MIN_VALUE >>> 2`.
pub const FLAG_WRITE_LEVEL: i64 = 1i64 << 62;
/// `FLAG_RECHECK_LEVEL = Long.MIN_VALUE >>> 1`.
pub const FLAG_RECHECK_LEVEL: i64 = i64::MIN | (1i64 << 62);
/// `FLAG_HAS_SIDED_TRANSPARENT_BLOCKS = Long.MIN_VALUE`.
pub const FLAG_HAS_SIDED_TRANSPARENT_BLOCKS: i64 = i64::MIN;

const COORD_MASK: i64 = (1i64 << 28) - 1;
pub(crate) const LEVEL_SHIFT: u32 = 6 + 6 + 16;
pub(crate) const DIR_SHIFT: u32 = 6 + 6 + 16 + 4;

/// `StarLightEngine` — базовый класс обоих Starlight-движков.
pub struct StarLightEngine {
    /// Тип движка (sky/block) — определяет реализацию абстрактных методов.
    pub kind: EngineKind,
    /// `skylightPropagator`.
    pub skylight_propagator: bool,
    /// `emittedLightMask` — 0 для sky, 0xF для block.
    pub emitted_light_mask: i32,
    /// `isClientSide`.
    pub is_client_side: bool,
    /// `minLightSection`/`maxLightSection` — границы light-секций.
    pub min_light_section: i32,
    /// `maxLightSection`.
    pub max_light_section: i32,
    /// `minSection`/`maxSection` — границы обычных секций.
    pub min_section: i32,
    /// `maxSection`.
    pub max_section: i32,
    /// `WorldUtil.getTotalSections(world)`.
    pub total_sections: i32,
    /// `WorldUtil.getTotalLightSections(world)`.
    pub total_light_sections: i32,

    /// `sectionCache` — 5×5×(lightSections+2), `*const` на секции чанков.
    pub(crate) section_cache: Vec<Option<*const LevelChunkSection>>,
    /// `nibbleCache` — SWMRNibbleArray по той же индексации.
    pub(crate) nibble_cache: Vec<Option<NibbleArrayRef>>,
    /// `notifyUpdateCache` — клиентские уведомления об изменениях.
    pub(crate) notify_update_cache: Vec<bool>,
    /// `chunkCache` — 5×5 чанков.
    pub(crate) chunk_cache: Vec<Option<*const ChunkAccess>>,
    /// `emptinessMapCache` — 5×5 карт пустоты.
    pub(crate) emptiness_map_cache: Vec<Option<EmptinessMapRef>>,

    // sky-only состояние (SkyStarLightEngine)
    /// `heightMapBlockChange` — [16*16] максимум изменённых Y по колонке.
    pub height_map_block_change: [i32; 16 * 16],
    /// `nullPropagationCheckCache`.
    pub null_propagation_check_cache: Vec<bool>,

    /// `encodeOffsetX/Y/Z`.
    pub encode_offset_x: i32,
    /// `encodeOffsetY`.
    pub encode_offset_y: i32,
    /// `encodeOffsetZ`.
    pub encode_offset_z: i32,
    /// `coordinateOffset`.
    pub coordinate_offset: i64,
    /// `chunkOffsetX/Y/Z`.
    pub chunk_offset_x: i32,
    /// `chunkOffsetY`.
    pub chunk_offset_y: i32,
    /// `chunkOffsetZ`.
    pub chunk_offset_z: i32,
    /// `chunkIndexOffset`.
    pub chunk_index_offset: i32,
    /// `chunkSectionIndexOffset`.
    pub chunk_section_index_offset: i32,

    /// `increaseQueue`/`decreaseQueue` + их длины.
    pub increase_queue: Vec<i64>,
    /// `increaseQueueInitialLength`.
    pub increase_queue_len: usize,
    /// `decreaseQueue`.
    pub decrease_queue: Vec<i64>,
    /// `decreaseQueueInitialLength`.
    pub decrease_queue_len: usize,

    /// `chunkCheckDelayedUpdatesCenter/Neighbour`.
    pub chunk_check_delayed_center: [i32; 16 * 16],
    /// `chunkCheckDelayedUpdatesNeighbour`.
    pub chunk_check_delayed_neighbour: [i32; 16 * 16],
}

/// `OLD_CHECK_DIRECTIONS` — направления, проверяемые для каждой
/// 6-битной маски направлений (64 варианта).
static OLD_CHECK_DIRECTIONS: LazyLock<Vec<Vec<AxisDirection>>> = LazyLock::new(|| {
    let mut ret = Vec::with_capacity(64);
    for i in 0..64i32 {
        let mut directions = Vec::new();
        let mut bitset = i;
        let len = bitset.count_ones();
        for _ in 0..len {
            // IntegerUtil.getTrailingBit = x & -x; trailingZeros
            let trailing = bitset & bitset.wrapping_neg();
            directions.push(AxisDirection::VALUES[trailing.trailing_zeros() as usize]);
            bitset ^= trailing;
        }
        ret.push(directions);
    }
    ret
});

impl StarLightEngine {
    /// `new(skylightPropagator)`.
    pub fn new(kind: EngineKind) -> Self {
        let skylight_propagator = kind == EngineKind::Sky;
        Self {
            kind,
            skylight_propagator,
            emitted_light_mask: if skylight_propagator { 0 } else { 0xF },
            is_client_side: false,
            min_light_section: -5,
            max_light_section: 20,
            min_section: -4,
            max_section: 19,
            total_sections: 24,
            total_light_sections: 26,
            section_cache: Vec::new(),
            nibble_cache: Vec::new(),
            notify_update_cache: Vec::new(),
            chunk_cache: vec![None; 5 * 5],
            emptiness_map_cache: vec![None; 5 * 5],
            height_map_block_change: [i32::MIN; 16 * 16],
            null_propagation_check_cache: Vec::new(),
            encode_offset_x: 0,
            encode_offset_y: 0,
            encode_offset_z: 0,
            coordinate_offset: 0,
            chunk_offset_x: 0,
            chunk_offset_y: 0,
            chunk_offset_z: 0,
            chunk_index_offset: 0,
            chunk_section_index_offset: 0,
            increase_queue: vec![0i64; 16 * 16 * 16],
            increase_queue_len: 0,
            decrease_queue: vec![0i64; 16 * 16 * 16],
            decrease_queue_len: 0,
            chunk_check_delayed_center: [0; 16 * 16],
            chunk_check_delayed_neighbour: [0; 16 * 16],
        }
    }

    /// `setWorld(world)` — `None` сбрасывает движок (аналог `setWorld(null)`).
    pub fn set_world(&mut self, level: Option<&dyn LevelReader>) {
        match level {
            None => {
                self.section_cache.clear();
                self.nibble_cache.clear();
                self.notify_update_cache.clear();
            }
            Some(level) => {
                self.is_client_side = level.is_client_side();
                self.min_section = WorldUtil::get_min_section(level);
                self.max_section = WorldUtil::get_max_section(level);
                self.min_light_section = WorldUtil::get_min_light_section(level);
                self.max_light_section = WorldUtil::get_max_light_section(level);
                self.total_sections = WorldUtil::get_total_sections(level);
                self.total_light_sections = WorldUtil::get_total_light_sections(level);
                let min_array_size =
                    5 * 5 * ((self.max_light_section - self.min_light_section + 1) + 2) as usize;
                if self.section_cache.len() < min_array_size {
                    self.section_cache = vec![None; min_array_size];
                } else {
                    for v in self.section_cache.iter_mut() {
                        *v = None;
                    }
                }
                if self.nibble_cache.len() < min_array_size {
                    self.nibble_cache = vec![None; min_array_size];
                } else {
                    for v in self.nibble_cache.iter_mut() {
                        *v = None;
                    }
                }
                if self.notify_update_cache.len() < min_array_size {
                    self.notify_update_cache = vec![false; min_array_size];
                } else {
                    for v in self.notify_update_cache.iter_mut() {
                        *v = false;
                    }
                }
                let sky_min = WorldUtil::get_total_light_sections(level) as usize;
                if self.null_propagation_check_cache.len() < sky_min {
                    self.null_propagation_check_cache = vec![false; sky_min];
                }
            }
        }
    }

    /// `setupEncodeOffset(centerX, centerY, centerZ)` — center в
    /// block-координатах центра чанка (x*16+7 и т.п.).
    pub fn setup_encode_offset(&mut self, center_x: i32, _center_y: i32, center_z: i32) {
        // 31 = center + encodeOffset
        self.encode_offset_x = 31 - center_x;
        self.encode_offset_y = -(self.min_light_section - 1) << 4;
        self.encode_offset_z = 31 - center_z;

        // coordinateIndex = x | (z << 6) | (y << 12)
        self.coordinate_offset = self.encode_offset_x as i64
            + ((self.encode_offset_z as i64) << 6)
            + ((self.encode_offset_y as i64) << 12);

        // 2 = (centerX >> 4) + chunkOffset
        self.chunk_offset_x = 2 - (center_x >> 4);
        self.chunk_offset_y = -(self.min_light_section - 1);
        self.chunk_offset_z = 2 - (center_z >> 4);

        // chunk index = x + (5 * z)
        self.chunk_index_offset = self.chunk_offset_x + 5 * self.chunk_offset_z;

        // chunk section index = x + (5 * z) + ((5*5) * y)
        self.chunk_section_index_offset =
            self.chunk_index_offset + 25 * self.chunk_offset_y;
    }

    /// `setupCaches(chunkProvider, centerX, centerY, centerZ, relaxed,
    /// tryToLoadChunksFor2Radius)`.
    pub fn setup_caches(
        &mut self,
        light_access: &dyn LightChunkGetter,
        center_x: i32,
        center_y: i32,
        center_z: i32,
        relaxed: bool,
        try_to_load_chunks_for_2_radius: bool,
    ) {
        let center_chunk_x = center_x >> 4;
        let center_chunk_y = center_y >> 4;
        let center_chunk_z = center_z >> 4;

        self.setup_encode_offset(
            center_chunk_x * 16 + 7,
            center_chunk_y * 16 + 7,
            center_chunk_z * 16 + 7,
        );

        let radius = if try_to_load_chunks_for_2_radius { 2 } else { 1 };

        for dz in -radius..=radius {
            for dx in -radius..=radius {
                let cx = center_chunk_x + dx;
                let cz = center_chunk_z + dz;
                let is_two_radius = dx.abs().max(dz.abs()) == 2;
                let chunk = light_access.get_chunk_for_lighting(cx, cz);

                let chunk = match chunk {
                    Some(c) => c,
                    None => {
                        if relaxed || is_two_radius {
                            continue;
                        }
                        panic!("Trying to propagate light update before 1 radius neighbours ready");
                    }
                };

                if !self.can_use_chunk(chunk) {
                    continue;
                }

                // SAFETY: чанк принадлежит light_access и живёт не
                // меньше длительности вызова; указатель используется
                // только внутри setup_caches..destroyCaches.
                self.set_chunk_in_cache(cx, cz, chunk as *const ChunkAccess);
                self.set_emptiness_map_cache(cx, cz, self.get_emptiness_map_chunk(chunk));
                if !is_two_radius {
                    self.set_blocks_for_chunk_in_cache(cx, cz, &chunk.sections);
                    let nibbles = self.get_nibbles_on_chunk(chunk);
                    self.set_nibbles_for_chunk_in_cache(cx, cz, Some(&nibbles));
                }
            }
        }
    }

    /// `getChunkInCache`.
    pub(crate) fn get_chunk_in_cache(&self, chunk_x: i32, chunk_z: i32) -> Option<&ChunkAccess> {
        let idx = self.chunk_index(chunk_x, chunk_z) as usize;
        // SAFETY: указатели валидны на время вызова (см. setup_caches).
        unsafe { self.chunk_cache.get(idx).copied().flatten().map(|p| &*p) }
    }

    /// `setChunkInCache`.
    pub(crate) fn set_chunk_in_cache(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        chunk: *const ChunkAccess,
    ) {
        let idx = self.chunk_index(chunk_x, chunk_z) as usize;
        if let Some(slot) = self.chunk_cache.get_mut(idx) {
            *slot = Some(chunk);
        }
    }

    /// `chunkIndex = x + 5*z + chunkIndexOffset`.
    pub(crate) fn chunk_index(&self, chunk_x: i32, chunk_z: i32) -> i32 {
        chunk_x + 5 * chunk_z + self.chunk_index_offset
    }

    /// `getChunkSection`.
    pub(crate) fn get_chunk_section(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> Option<&LevelChunkSection> {
        let idx =
            self.chunk_section_index(chunk_x, chunk_y, chunk_z) as usize;
        // SAFETY: см. setup_caches.
        unsafe {
            self.section_cache
                .get(idx)
                .copied()
                .flatten()
                .map(|p| &*p)
        }
    }

    /// `setChunkSectionInCache`.
    pub(crate) fn set_chunk_section_in_cache(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        section: Option<*const LevelChunkSection>,
    ) {
        let idx =
            self.chunk_section_index(chunk_x, chunk_y, chunk_z) as usize;
        if let Some(slot) = self.section_cache.get_mut(idx) {
            *slot = section;
        }
    }

    /// `x + 5*z + 25*y + chunkSectionIndexOffset`.
    pub(crate) fn chunk_section_index(&self, x: i32, y: i32, z: i32) -> i32 {
        x + 5 * z + 25 * y + self.chunk_section_index_offset
    }

    /// `setBlocksForChunkInCache`.
    pub(crate) fn set_blocks_for_chunk_in_cache(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        sections: &[LevelChunkSection],
    ) {
        for cy in self.min_light_section..=self.max_light_section {
            let section = if cy >= self.min_section && cy <= self.max_section {
                sections
                    .get((cy - self.min_section) as usize)
                    .map(|s| s as *const LevelChunkSection)
            } else {
                None
            };
            self.set_chunk_section_in_cache(chunk_x, cy, chunk_z, section);
        }
    }

    /// `getNibbleFromCache`.
    pub(crate) fn get_nibble_from_cache(
        &self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) -> Option<NibbleArrayRef> {
        let idx =
            self.chunk_section_index(chunk_x, chunk_y, chunk_z) as usize;
        self.nibble_cache.get(idx).cloned().flatten()
    }

    /// `getNibblesForChunkFromCache`.
    #[allow(dead_code)]
    pub(crate) fn get_nibbles_for_chunk_from_cache(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Vec<Option<NibbleArrayRef>> {
        let mut ret = Vec::with_capacity(
            (self.max_light_section - self.min_light_section + 1) as usize,
        );
        for cy in self.min_light_section..=self.max_light_section {
            ret.push(self.get_nibble_from_cache(chunk_x, cy, chunk_z));
        }
        ret
    }

    /// `setNibbleInCache`.
    pub(crate) fn set_nibble_in_cache(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        nibble: Option<NibbleArrayRef>,
    ) {
        let idx =
            self.chunk_section_index(chunk_x, chunk_y, chunk_z) as usize;
        if let Some(slot) = self.nibble_cache.get_mut(idx) {
            *slot = nibble;
        }
    }

    /// `setNibblesForChunkInCache`.
    pub(crate) fn set_nibbles_for_chunk_in_cache(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        nibbles: Option<&[Option<NibbleArrayRef>]>,
    ) {
        for cy in self.min_light_section..=self.max_light_section {
            let nibble = nibbles
                .and_then(|n| n.get((cy - self.min_light_section) as usize))
                .cloned()
                .flatten();
            self.set_nibble_in_cache(chunk_x, cy, chunk_z, nibble);
        }
    }

    /// `updateVisible(lightAccess)` — публикует обновлённые нибблы.
    pub(crate) fn update_visible(&mut self, light_access: &dyn LightChunkGetter) {
        for index in 0..self.nibble_cache.len() {
            let nibble = match &self.nibble_cache[index] {
                Some(n) => n.clone(),
                None => {
                    if self.notify_update_cache.get(index).copied().unwrap_or(false) {
                        // notify без nibble — обрабатываем ниже
                    } else {
                        continue;
                    }
                    // nibble == null && notify — Java: `(nibble != null && nibble.updateVisible()) || notify`
                    // продолжим к декодированию координат
                    let chunk_x = (index % 5) as i32 - self.chunk_offset_x;
                    let chunk_z = ((index / 5) % 5) as i32 - self.chunk_offset_z;
                    let y_sections = self.max_section - self.min_section + 1;
                    let chunk_y =
                        ((index / 25) % (y_sections + 4) as usize) as i32 - self.chunk_offset_y;
                    light_access.on_light_update(
                        if self.skylight_propagator {
                            LightLayer::Sky
                        } else {
                            LightLayer::Block
                        },
                        SectionPos::of_i(chunk_x, chunk_y, chunk_z),
                    );
                    continue;
                }
            };

            let notify = self.notify_update_cache.get(index).copied().unwrap_or(false);
            let dirty = nibble.lock().unwrap().is_dirty();
            if !notify && !dirty {
                continue;
            }

            let chunk_x = (index % 5) as i32 - self.chunk_offset_x;
            let chunk_z = ((index / 5) % 5) as i32 - self.chunk_offset_z;
            let y_sections = self.max_section - self.min_section + 1;
            let chunk_y =
                ((index / 25) % (y_sections + 4) as usize) as i32 - self.chunk_offset_y;
            let updated = nibble.lock().unwrap().update_visible();
            if updated || notify {
                light_access.on_light_update(
                    if self.skylight_propagator {
                        LightLayer::Sky
                    } else {
                        LightLayer::Block
                    },
                    SectionPos::of_i(chunk_x, chunk_y, chunk_z),
                );
            }
        }
    }

    /// `destroyCaches`.
    pub(crate) fn destroy_caches(&mut self) {
        for v in self.section_cache.iter_mut() {
            *v = None;
        }
        for v in self.nibble_cache.iter_mut() {
            *v = None;
        }
        for v in self.chunk_cache.iter_mut() {
            *v = None;
        }
        for v in self.emptiness_map_cache.iter_mut() {
            *v = None;
        }
        if self.is_client_side {
            for v in self.notify_update_cache.iter_mut() {
                *v = false;
            }
        }
    }

    /// `getBlockState(worldX, worldY, worldZ)`.
    pub(crate) fn get_block_state(&self, world_x: i32, world_y: i32, world_z: i32) -> BlockState {
        let idx = self.chunk_section_index(world_x >> 4, world_y >> 4, world_z >> 4) as usize;
        if let Some(section) = self
            .section_cache
            .get(idx)
            .copied()
            .flatten()
            .map(|p| unsafe { &*p })
        {
            if section.has_only_air() {
                return BlockState::air();
            }
            return section.get_block_state(world_x & 15, world_y & 15, world_z & 15);
        }
        BlockState::air()
    }

    /// `getBlockState(sectionIndex, localIndex)`.
    pub(crate) fn get_block_state_at(
        &self,
        section_index: i32,
        local_index: usize,
    ) -> BlockState {
        if let Some(section) = self
            .section_cache
            .get(section_index as usize)
            .copied()
            .flatten()
            .map(|p| unsafe { &*p })
        {
            if section.has_only_air() {
                return BlockState::air();
            }
            return section.states.get_by_index(local_index);
        }
        BlockState::air()
    }

    /// `getLightLevel(worldX, worldY, worldZ)`.
    pub(crate) fn get_light_level(&self, world_x: i32, world_y: i32, world_z: i32) -> u8 {
        let idx = self.chunk_section_index(world_x >> 4, world_y >> 4, world_z >> 4) as usize;
        match self.nibble_cache.get(idx).and_then(|n| n.as_ref()) {
            Some(n) => n
                .lock()
                .unwrap()
                .get_updating(
                    (world_x & 15) as usize
                        | (((world_z & 15) as usize) << 4)
                        | (((world_y & 15) as usize) << 8),
                ),
            None => 0,
        }
    }

    /// `getLightLevel(sectionIndex, localIndex)`.
    pub(crate) fn get_light_level_at(&self, section_index: i32, local_index: usize) -> u8 {
        match self
            .nibble_cache
            .get(section_index as usize)
            .and_then(|n| n.as_ref())
        {
            Some(n) => n.lock().unwrap().get_updating(local_index),
            None => 0,
        }
    }

    /// `setLightLevel(worldX, worldY, worldZ, level)`.
    pub(crate) fn set_light_level(
        &mut self,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        level: u8,
    ) {
        let section_index =
            self.chunk_section_index(world_x >> 4, world_y >> 4, world_z >> 4);
        let nibble = self
            .nibble_cache
            .get(section_index as usize)
            .and_then(|n| n.as_ref())
            .cloned();
        if let Some(nibble) = nibble {
            nibble.lock().unwrap().set(
                (world_x & 15) as usize
                    | (((world_z & 15) as usize) << 4)
                    | (((world_y & 15) as usize) << 8),
                level,
            );
            if self.is_client_side {
                self.mark_notify_neighbours(world_x, world_y, world_z);
            }
        }
    }

    /// `postLightUpdate(worldX, worldY, worldZ)` — клиентские
    /// уведомления для соседних секций.
    pub(crate) fn post_light_update(&mut self, world_x: i32, world_y: i32, world_z: i32) {
        if self.is_client_side {
            self.mark_notify_neighbours(world_x, world_y, world_z);
        }
    }

    pub(crate) fn mark_notify_neighbours(&mut self, world_x: i32, world_y: i32, world_z: i32) {
        let cx1 = (world_x - 1) >> 4;
        let cx2 = (world_x + 1) >> 4;
        let cy1 = (world_y - 1) >> 4;
        let cy2 = (world_y + 1) >> 4;
        let cz1 = (world_z - 1) >> 4;
        let cz2 = (world_z + 1) >> 4;
        for x in cx1..=cx2 {
            for y in cy1..=cy2 {
                for z in cz1..=cz2 {
                    let idx = self
                        .chunk_section_index(x, y, z)
                        .clamp(0, self.notify_update_cache.len() as i32 - 1)
                        as usize;
                    if let Some(slot) = self.notify_update_cache.get_mut(idx) {
                        *slot = true;
                    }
                }
            }
        }
    }

    /// `setLightLevel(sectionIndex, localIndex, worldX, worldY, worldZ, level)`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn set_light_level_at(
        &mut self,
        section_index: i32,
        local_index: usize,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        level: u8,
    ) {
        let nibble = self
            .nibble_cache
            .get(section_index as usize)
            .and_then(|n| n.as_ref())
            .cloned();
        if let Some(nibble) = nibble {
            nibble.lock().unwrap().set(local_index, level);
            if self.is_client_side {
                self.mark_notify_neighbours(world_x, world_y, world_z);
            }
        }
    }

    /// `getEmptinessMap(chunkX, chunkZ)` — из кэша.
    pub(crate) fn get_emptiness_map_cached(
        &self,
        chunk_x: i32,
        chunk_z: i32,
    ) -> Option<EmptinessMapRef> {
        let idx = self.chunk_index(chunk_x, chunk_z) as usize;
        self.emptiness_map_cache.get(idx).cloned().flatten()
    }

    /// `setEmptinessMapCache`.
    pub(crate) fn set_emptiness_map_cache(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        emptiness_map: Option<EmptinessMapRef>,
    ) {
        let idx = self.chunk_index(chunk_x, chunk_z) as usize;
        if let Some(slot) = self.emptiness_map_cache.get_mut(idx) {
            *slot = emptiness_map;
        }
    }

    /// `getFilledEmptyLight(totalLightSections)`.
    pub fn get_filled_empty_light(total_light_sections: usize) -> Vec<Option<NibbleArrayRef>> {
        get_filled_empty_light_vec(total_light_sections)
    }

    // ----------------------------------------------------------
    // Абстрактные методы — диспетчеризация по kind
    // ----------------------------------------------------------

    /// `getEmptinessMap(chunk)` — поле чанка.
    pub(crate) fn get_emptiness_map_chunk(
        &self,
        chunk: &ChunkAccess,
    ) -> Option<EmptinessMapRef> {
        use crate::ca::moonrise::patches::starlight::chunk::StarlightChunk;
        match self.kind {
            EngineKind::Sky => chunk.starlight_get_sky_emptiness_map(),
            EngineKind::Block => chunk.starlight_get_block_emptiness_map(),
        }
    }

    /// `setEmptinessMap(chunk, to)`.
    pub(crate) fn set_emptiness_map_chunk(
        &self,
        chunk: &ChunkAccess,
        to: Option<EmptinessMapRef>,
    ) {
        use crate::ca::moonrise::patches::starlight::chunk::StarlightChunk;
        match self.kind {
            EngineKind::Sky => chunk.starlight_set_sky_emptiness_map(to),
            EngineKind::Block => chunk.starlight_set_block_emptiness_map(to),
        }
    }

    /// `getNibblesOnChunk(chunk)`.
    pub(crate) fn get_nibbles_on_chunk(
        &self,
        chunk: &ChunkAccess,
    ) -> Vec<Option<NibbleArrayRef>> {
        use crate::ca::moonrise::patches::starlight::chunk::StarlightChunk;
        match self.kind {
            EngineKind::Sky => chunk.starlight_get_sky_nibbles(),
            EngineKind::Block => chunk.starlight_get_block_nibbles(),
        }
    }

    /// `setNibbles(chunk, to)`.
    pub(crate) fn set_nibbles(
        &self,
        chunk: &ChunkAccess,
        to: Vec<Option<NibbleArrayRef>>,
    ) {
        use crate::ca::moonrise::patches::starlight::chunk::StarlightChunk;
        match self.kind {
            EngineKind::Sky => chunk.starlight_set_sky_nibbles(to),
            EngineKind::Block => chunk.starlight_set_block_nibbles(to),
        }
    }

    /// `canUseChunk(chunk)` — одинаково для обоих движков.
    pub(crate) fn can_use_chunk(&self, chunk: &ChunkAccess) -> bool {
        chunk
            .get_persisted_status()
            .is_or_after(ChunkStatus::Light)
            && (self.is_client_side || chunk.is_light_correct())
    }

    /// `initNibble` — абстрактный.
    pub(crate) fn init_nibble(
        &mut self,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
        extrude: bool,
        init_removed_nibbles: bool,
    ) {
        match self.kind {
            EngineKind::Sky => {
                sky_star_light_engine::init_nibble(self, chunk_x, chunk_y, chunk_z, extrude, init_removed_nibbles)
            }
            EngineKind::Block => {
                block_star_light_engine::init_nibble(self, chunk_x, chunk_y, chunk_z, extrude, init_removed_nibbles)
            }
        }
    }

    /// `setNibbleNull` — абстрактный.
    pub(crate) fn set_nibble_null(&mut self, chunk_x: i32, chunk_y: i32, chunk_z: i32) {
        match self.kind {
            EngineKind::Sky => sky_star_light_engine::set_nibble_null(self, chunk_x, chunk_y, chunk_z),
            EngineKind::Block => {
                block_star_light_engine::set_nibble_null(self, chunk_x, chunk_y, chunk_z)
            }
        }
    }

    /// `propagateBlockChanges` — абстрактный.
    pub(crate) fn propagate_block_changes(
        &mut self,
        light_access: &dyn LightChunkGetter,
        at_chunk: &ChunkAccess,
        positions: &HashSet<BlockPos>,
    ) {
        match self.kind {
            EngineKind::Sky => {
                sky_star_light_engine::propagate_block_changes(self, light_access, at_chunk, positions)
            }
            EngineKind::Block => {
                block_star_light_engine::propagate_block_changes(self, light_access, at_chunk, positions)
            }
        }
    }

    /// `checkBlock` — абстрактный.
    pub(crate) fn check_block(
        &mut self,
        light_access: &dyn LightChunkGetter,
        world_x: i32,
        world_y: i32,
        world_z: i32,
    ) {
        match self.kind {
            EngineKind::Sky => {
                sky_star_light_engine::check_block(self, light_access, world_x, world_y, world_z)
            }
            EngineKind::Block => {
                block_star_light_engine::check_block(self, light_access, world_x, world_y, world_z)
            }
        }
    }

    /// `calculateLightValue` — абстрактный.
    pub(crate) fn calculate_light_value(
        &self,
        light_access: &dyn LightChunkGetter,
        world_x: i32,
        world_y: i32,
        world_z: i32,
        expect: u8,
    ) -> u8 {
        match self.kind {
            EngineKind::Sky => sky_star_light_engine::calculate_light_value(
                self, light_access, world_x, world_y, world_z, expect,
            ),
            EngineKind::Block => block_star_light_engine::calculate_light_value(
                self, light_access, world_x, world_y, world_z, expect,
            ),
        }
    }

    /// `lightChunk` — абстрактный.
    pub(crate) fn light_chunk(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        needs_edge_checks: bool,
    ) {
        match self.kind {
            EngineKind::Sky => {
                sky_star_light_engine::light_chunk(self, light_access, chunk, needs_edge_checks)
            }
            EngineKind::Block => {
                block_star_light_engine::light_chunk(self, light_access, chunk, needs_edge_checks)
            }
        }
    }

    /// `checkChunkEdges(lightAccess, chunk, fromSection, toSection)` —
    /// sky-переопределение обёрнуто диспетчеризацией внутри sky-модуля.
    fn check_chunk_edges_range(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        from_section: i32,
        to_section: i32,
    ) {
        match self.kind {
            EngineKind::Sky => {
                sky_star_light_engine::check_chunk_edges_range(self, light_access, chunk, from_section, to_section)
            }
            EngineKind::Block => {
                self.check_chunk_edges_range_base(light_access, chunk, from_section, to_section)
            }
        }
    }

    /// `checkChunkEdges(lightAccess, chunk, sections)` — базовая реализация.
    fn check_chunk_edges_sections(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        sections: &[i16],
    ) {
        match self.kind {
            EngineKind::Sky => {
                sky_star_light_engine::check_chunk_edges_sections(self, light_access, chunk, sections)
            }
            EngineKind::Block => {
                self.check_chunk_edges_sections_base(light_access, chunk, sections)
            }
        }
    }

    // ----------------------------------------------------------
    // Публичный API
    // ----------------------------------------------------------

    /// `blocksChangedInChunk`.
    pub fn blocks_changed_in_chunk(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
        positions: &HashSet<BlockPos>,
        changed_sections: Option<&mut [Option<bool>]>,
    ) {
        self.setup_caches(
            light_access,
            chunk_x * 16 + 7,
            128,
            chunk_z * 16 + 7,
            true,
            true,
        );
        let chunk = match self.get_chunk_in_cache(chunk_x, chunk_z) {
            Some(c) => c as *const ChunkAccess,
            None => {
                self.destroy_caches();
                return;
            }
        };
        // SAFETY: chunk из кэша — валиден на время вызова.
        let chunk = unsafe { &*chunk };

        if let Some(changed_sections) = changed_sections {
            if let Some(ret) =
                self.handle_empty_section_changes_inner(light_access, chunk, changed_sections, false)
            {
                self.set_emptiness_map_chunk(chunk, Some(ret));
            }
        }
        if !positions.is_empty() {
            self.propagate_block_changes(light_access, chunk, positions);
        }
        self.update_visible(light_access);
        self.destroy_caches();
    }

    /// `checkChunkEdges(lightAccess, chunkX, chunkZ)`.
    pub fn check_chunk_edges(&mut self, light_access: &dyn LightChunkGetter, chunk_x: i32, chunk_z: i32) {
        self.setup_caches(
            light_access,
            chunk_x * 16 + 7,
            128,
            chunk_z * 16 + 7,
            true,
            false,
        );
        let chunk = self.get_chunk_in_cache(chunk_x, chunk_z).map(|c| c as *const ChunkAccess);
        match chunk {
            Some(c) => {
                // SAFETY: валиден на время вызова.
                let chunk = unsafe { &*c };
                self.check_chunk_edges_range(
                    light_access,
                    chunk,
                    self.min_light_section,
                    self.max_light_section,
                );
                self.update_visible(light_access);
            }
            None => {}
        }
        self.destroy_caches();
    }

    /// `checkChunkEdges(lightAccess, chunkX, chunkZ, sections)`.
    pub fn check_chunk_edges_with_sections(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
        sections: &[i16],
    ) {
        self.setup_caches(
            light_access,
            chunk_x * 16 + 7,
            128,
            chunk_z * 16 + 7,
            true,
            false,
        );
        let chunk = self.get_chunk_in_cache(chunk_x, chunk_z).map(|c| c as *const ChunkAccess);
        match chunk {
            Some(c) => {
                // SAFETY: валиден на время вызова.
                let chunk = unsafe { &*c };
                self.check_chunk_edges_sections(light_access, chunk, sections);
                self.update_visible(light_access);
            }
            None => {}
        }
        self.destroy_caches();
    }

    /// `getEmptySectionsForChunk(chunk)` — `Boolean[]` → `Vec<Option<bool>>`
    /// (элементы всегда `Some`).
    pub fn get_empty_sections_for_chunk(chunk: &ChunkAccess) -> Vec<Option<bool>> {
        chunk
            .sections
            .iter()
            .map(|s| Some(s.has_only_air()))
            .collect()
    }

    /// `forceHandleEmptySectionChanges`.
    pub fn force_handle_empty_section_changes(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        emptiness_changes: &mut [Option<bool>],
    ) {
        let chunk_pos = chunk.get_pos();
        let (chunk_x, chunk_z) = (chunk_pos.x, chunk_pos.z);
        self.setup_caches(
            light_access,
            chunk_x * 16 + 7,
            128,
            chunk_z * 16 + 7,
            true,
            true,
        );

        // force current chunk into cache
        // SAFETY: chunk принадлежит light_access; живёт дольше вызова.
        self.set_chunk_in_cache(chunk_x, chunk_z, chunk as *const ChunkAccess);
        self.set_blocks_for_chunk_in_cache(chunk_x, chunk_z, &chunk.sections);
        self.set_nibbles_for_chunk_in_cache(
            chunk_x,
            chunk_z,
            Some(&self.get_nibbles_on_chunk(chunk)),
        );
        self.set_emptiness_map_cache(chunk_x, chunk_z, self.get_emptiness_map_chunk(chunk));

        if let Some(ret) =
            self.handle_empty_section_changes_inner(light_access, chunk, emptiness_changes, false)
        {
            self.set_emptiness_map_chunk(chunk, Some(ret));
        }
        self.update_visible(light_access);
        self.destroy_caches();
    }

    /// `handleEmptySectionChanges(lightAccess, chunkX, chunkZ, emptinessChanges)`.
    pub fn handle_empty_section_changes(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_z: i32,
        emptiness_changes: &mut [Option<bool>],
    ) {
        self.setup_caches(
            light_access,
            chunk_x * 16 + 7,
            128,
            chunk_z * 16 + 7,
            true,
            true,
        );
        let chunk = self.get_chunk_in_cache(chunk_x, chunk_z).map(|c| c as *const ChunkAccess);
        match chunk {
            Some(c) => {
                // SAFETY: валиден на время вызова.
                let chunk = unsafe { &*c };
                if let Some(ret) = self.handle_empty_section_changes_inner(
                    light_access,
                    chunk,
                    emptiness_changes,
                    false,
                ) {
                    self.set_emptiness_map_chunk(chunk, Some(ret));
                }
                self.update_visible(light_access);
            }
            None => {}
        }
        self.destroy_caches();
    }

    /// `handleEmptySectionChanges(lightAccess, chunk, emptinessChanges, unlit)` —
    /// внутренняя реализация; возвращает новую карту пустоты, если она
    /// была создана (`ret != null` в Java).
    pub(crate) fn handle_empty_section_changes_inner(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        emptiness_changes: &mut [Option<bool>],
        unlit: bool,
    ) -> Option<EmptinessMapRef> {
        let chunk_pos = chunk.get_pos();
        let chunk_x = chunk_pos.x;
        let chunk_z = chunk_pos.z;

        let mut chunk_emptiness_map = self.get_emptiness_map_cached(chunk_x, chunk_z);
        let mut ret: Option<EmptinessMapRef> = None;
        let needs_init = unlit || chunk_emptiness_map.is_none();
        if needs_init {
            let new_map: EmptinessMapRef =
                std::sync::Arc::new(std::sync::Mutex::new(vec![false; self.total_sections as usize]));
            self.set_emptiness_map_cache(chunk_x, chunk_z, Some(new_map.clone()));
            chunk_emptiness_map = Some(new_map.clone());
            ret = Some(new_map);
        }
        let chunk_emptiness_map = chunk_emptiness_map.unwrap();

        // update emptiness map
        for section_index in (0..emptiness_changes.len()).rev() {
            let mut value = emptiness_changes[section_index];
            if value.is_none() {
                if !needs_init {
                    continue;
                }
                let section =
                    self.get_chunk_section(chunk_x, section_index as i32 + self.min_section, chunk_z);
                value = Some(match section {
                    None => true,
                    Some(s) => s.has_only_air(),
                });
                emptiness_changes[section_index] = value;
            }
            chunk_emptiness_map.lock().unwrap()[section_index] = value.unwrap();
        }

        // now init neighbour nibbles
        for section_index in (0..emptiness_changes.len()).rev() {
            let value = match emptiness_changes[section_index] {
                Some(v) => v,
                None => continue,
            };
            if value {
                continue;
            }
            let section_y = section_index as i32 + self.min_section;

            for dz in -1..=1 {
                for dx in -1..=1 {
                    let extrude = (dx | dz) != 0 || !unlit;
                    for dy in (0..=2).rev() {
                        // Java: for (dy = 1; dy >= -1; --dy)
                        let dy = dy as i32 - 1;
                        self.init_nibble(
                            dx + chunk_x,
                            dy + section_y,
                            dz + chunk_z,
                            extrude,
                            false,
                        );
                    }
                }
            }
        }

        // check for de-init and lazy-init
        for dz in -1..=1 {
            for dx in -1..=1 {
                // does this neighbour have 1 radius loaded?
                let mut neighbours_loaded = true;
                'neighbour_loaded_search: for dz2 in -1..=1 {
                    for dx2 in -1..=1 {
                        if self
                            .get_emptiness_map_cached(dx + dx2 + chunk_x, dz + dz2 + chunk_z)
                            .is_none()
                        {
                            neighbours_loaded = false;
                            break 'neighbour_loaded_search;
                        }
                    }
                }

                for section_y in (self.min_light_section..=self.max_light_section).rev() {
                    // check neighbours to see if we need to de-init this one
                    let mut all_empty = true;
                    'neighbour_search: for dy2 in -1..=1 {
                        for dz2 in -1..=1 {
                            for dx2 in -1..=1 {
                                let y = section_y + dy2;
                                if y < self.min_section || y > self.max_section {
                                    // empty
                                    continue;
                                }
                                let emptiness_map = self.get_emptiness_map_cached(
                                    dx + dx2 + chunk_x,
                                    dz + dz2 + chunk_z,
                                );
                                match emptiness_map {
                                    Some(map) => {
                                        if !map.lock().unwrap()
                                            [(y - self.min_section) as usize]
                                        {
                                            all_empty = false;
                                            break 'neighbour_search;
                                        }
                                    }
                                    None => {
                                        let section = self.get_chunk_section(
                                            dx + dx2 + chunk_x,
                                            y,
                                            dz + dz2 + chunk_z,
                                        );
                                        if let Some(s) = section {
                                            if !s.has_only_air() {
                                                all_empty = false;
                                                break 'neighbour_search;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if all_empty && neighbours_loaded {
                        // all were empty, so de-init
                        self.set_nibble_null(dx + chunk_x, section_y, dz + chunk_z);
                    } else if !all_empty {
                        // must init
                        let extrude = (dx | dz) != 0 || !unlit;
                        self.init_nibble(
                            dx + chunk_x,
                            section_y,
                            dz + chunk_z,
                            extrude,
                            false,
                        );
                    }
                }
            }
        }

        ret
    }

    /// `light(lightAccess, chunk, emptySections)`.
    pub fn light(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        empty_sections: &mut [Option<bool>],
    ) {
        let chunk_pos = chunk.get_pos();
        let (chunk_x, chunk_z) = (chunk_pos.x, chunk_pos.z);
        self.setup_caches(
            light_access,
            chunk_x * 16 + 7,
            128,
            chunk_z * 16 + 7,
            true,
            true,
        );

        let nibbles = get_filled_empty_light_vec(
            (self.max_light_section - self.min_light_section + 1) as usize,
        );
        // force current chunk into cache
        // SAFETY: chunk принадлежит light_access; живёт дольше вызова.
        self.set_chunk_in_cache(chunk_x, chunk_z, chunk as *const ChunkAccess);
        self.set_blocks_for_chunk_in_cache(chunk_x, chunk_z, &chunk.sections);
        self.set_nibbles_for_chunk_in_cache(chunk_x, chunk_z, Some(&nibbles));
        self.set_emptiness_map_cache(chunk_x, chunk_z, self.get_emptiness_map_chunk(chunk));

        if let Some(ret) =
            self.handle_empty_section_changes_inner(light_access, chunk, empty_sections, true)
        {
            self.set_emptiness_map_chunk(chunk, Some(ret));
        }
        self.light_chunk(light_access, chunk, true);
        self.set_nibbles(chunk, nibbles);
        self.update_visible(light_access);
        self.destroy_caches();
    }

    /// `relightChunks(lightAccess, chunks, chunkLightCallback, onComplete)`.
    pub fn relight_chunks(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunks: &HashSet<ChunkPos>,
        chunk_light_callback: Option<&mut dyn FnMut(ChunkPos)>,
        on_complete: Option<&mut dyn FnMut(i32)>,
    ) {
        let mut nibbles_by_chunk: HashMap<i64, Vec<Option<NibbleArrayRef>>> = HashMap::new();
        let mut emptiness_map_by_chunk: HashMap<i64, EmptinessMapRef> = HashMap::new();

        let neighbour_light_order: [i32; 18] = [
            // d = 0
            0, 0,
            // d = 1
            -1, 0,
            0, -1,
            1, 0,
            0, 1,
            // d = 2
            -1, 1,
            1, 1,
            -1, -1,
            1, -1,
        ];

        let mut light_calls = 0i32;
        let mut chunk_light_callback = chunk_light_callback;
        let mut on_complete = on_complete;

        for chunk_pos in chunks {
            let chunk_x = chunk_pos.x;
            let chunk_z = chunk_pos.z;
            let chunk = match light_access.get_chunk_for_lighting(chunk_x, chunk_z) {
                Some(c) if self.can_use_chunk(c) => c as *const ChunkAccess,
                _ => panic!("relightChunks: chunk missing or unusable"),
            };

            for i in (0..neighbour_light_order.len()).step_by(2) {
                let dx = neighbour_light_order[i];
                let dz = neighbour_light_order[i + 1];
                let neighbour_x = dx + chunk_x;
                let neighbour_z = dz + chunk_z;
                let neighbour_key = ChunkPos::as_long_static(neighbour_x, neighbour_z);

                let neighbour = match light_access.get_chunk_for_lighting(neighbour_x, neighbour_z) {
                    Some(c) if self.can_use_chunk(c) => c as *const ChunkAccess,
                    _ => continue,
                };

                if nibbles_by_chunk.contains_key(&neighbour_key) {
                    // lit already called for neighbour, no need to light it now
                    continue;
                }

                // light neighbour chunk
                self.setup_encode_offset(
                    neighbour_x * 16 + 7,
                    128,
                    neighbour_z * 16 + 7,
                );

                // insert all neighbouring chunks for this neighbour that we have data for
                for dz2 in -1..=1 {
                    for dx2 in -1..=1 {
                        let neighbour_x2 = neighbour_x + dx2;
                        let neighbour_z2 = neighbour_z + dz2;
                        let key = ChunkPos::as_long_static(neighbour_x2, neighbour_z2);
                        let neighbour2 =
                            match light_access.get_chunk_for_lighting(neighbour_x2, neighbour_z2) {
                                Some(c) if self.can_use_chunk(c) => c as *const ChunkAccess,
                                _ => continue,
                            };

                        let nibbles = match nibbles_by_chunk.get(&key) {
                            Some(n) => n.clone(),
                            // we haven't lit this chunk
                            None => continue,
                        };

                        // SAFETY: валиден на время вызова.
                        let neighbour2 = unsafe { &*neighbour2 };
                        self.set_chunk_in_cache(neighbour_x2, neighbour_z2, neighbour2 as *const ChunkAccess);
                        self.set_blocks_for_chunk_in_cache(neighbour_x2, neighbour_z2, &neighbour2.sections);
                        self.set_nibbles_for_chunk_in_cache(neighbour_x2, neighbour_z2, Some(&nibbles));
                        self.set_emptiness_map_cache(
                            neighbour_x2,
                            neighbour_z2,
                            emptiness_map_by_chunk.get(&key).cloned(),
                        );
                    }
                }

                // now insert the neighbour chunk and light it
                let nibbles = get_filled_empty_light_vec(self.total_light_sections as usize);
                nibbles_by_chunk.insert(neighbour_key, nibbles.clone());

                // SAFETY: валиден на время вызова.
                let neighbour = unsafe { &*neighbour };
                self.set_chunk_in_cache(neighbour_x, neighbour_z, neighbour as *const ChunkAccess);
                self.set_blocks_for_chunk_in_cache(neighbour_x, neighbour_z, &neighbour.sections);
                self.set_nibbles_for_chunk_in_cache(neighbour_x, neighbour_z, Some(&nibbles));

                let mut empty_sections = Self::get_empty_sections_for_chunk(neighbour);
                if let Some(neighbour_emptiness) = self.handle_empty_section_changes_inner(
                    light_access,
                    neighbour,
                    &mut empty_sections,
                    true,
                ) {
                    emptiness_map_by_chunk.insert(neighbour_key, neighbour_emptiness.clone());
                    if chunks.contains(&ChunkPos::new(neighbour_x, neighbour_z)) {
                        self.set_emptiness_map_chunk(neighbour, Some(neighbour_emptiness));
                    }
                }

                self.light_chunk(light_access, neighbour, false);
                self.destroy_caches();
            }

            // done lighting all neighbours, so the chunk is now fully lit

            // make sure nibbles are fully updated before calling back
            // SAFETY: chunk валиден на время вызова.
            let chunk = unsafe { &*chunk };
            let nibbles = nibbles_by_chunk
                .get(&ChunkPos::as_long_static(chunk_x, chunk_z))
                .cloned()
                .unwrap_or_default();
            for nibble in &nibbles {
                if let Some(n) = nibble {
                    n.lock().unwrap().update_visible();
                }
            }

            self.set_nibbles(chunk, nibbles);

            for y in self.min_light_section..=self.max_light_section {
                light_access.on_light_update(
                    if self.skylight_propagator {
                        LightLayer::Sky
                    } else {
                        LightLayer::Block
                    },
                    SectionPos::of_i(chunk_x, y, chunk_z),
                );
            }

            // now do callback
            if let Some(callback) = chunk_light_callback.as_deref_mut() {
                callback(ChunkPos::new(chunk_x, chunk_z));
            }
            light_calls += 1;
        }

        if let Some(on_complete) = on_complete.as_deref_mut() {
            on_complete(light_calls);
        }
    }

    // ----------------------------------------------------------
    // Очереди и распространение
    // ----------------------------------------------------------

    pub(crate) fn resize_increase_queue(&mut self) {
        let new_len = (self.increase_queue.len() + (self.increase_queue.len() >> 1)).max(4);
        self.increase_queue.resize(new_len, 0);
    }

    pub(crate) fn resize_decrease_queue(&mut self) {
        let new_len = (self.decrease_queue.len() + (self.decrease_queue.len() >> 1)).max(4);
        self.decrease_queue.resize(new_len, 0);
    }

    /// `appendToIncreaseQueue`.
    pub(crate) fn append_to_increase_queue(&mut self, value: i64) {
        let idx = self.increase_queue_len;
        self.increase_queue_len += 1;
        if idx >= self.increase_queue.len() {
            self.resize_increase_queue();
        }
        self.increase_queue[idx] = value;
    }

    /// `appendToDecreaseQueue`.
    pub(crate) fn append_to_decrease_queue(&mut self, value: i64) {
        let idx = self.decrease_queue_len;
        self.decrease_queue_len += 1;
        if idx >= self.decrease_queue.len() {
            self.resize_decrease_queue();
        }
        self.decrease_queue[idx] = value;
    }

    /// Кодирование позиции в 28 бит: `x | (z << 6) | (y << 12) + offset`.
    pub(crate) fn encode_position(&self, x: i32, y: i32, z: i32) -> i64 {
        (x as i64 + ((z as i64) << 6) + ((y as i64) << 12) + self.coordinate_offset)
            & COORD_MASK
    }

    /// Полная запись очереди: позиция | уровень << 28 | направления << 32 | флаги.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn make_queue_value(
        &self,
        x: i32,
        y: i32,
        z: i32,
        level: i64,
        direction_mask: i64,
        flags: i64,
    ) -> i64 {
        self.encode_position(x, y, z)
            | ((level & 0xF) << LEVEL_SHIFT)
            | (direction_mask << DIR_SHIFT)
            | flags
    }

    /// `checkChunkEdge` — проверка края чанка в одном направлении.
    pub(crate) fn check_chunk_edge(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk_x: i32,
        chunk_y: i32,
        chunk_z: i32,
    ) {
        let curr_nibble = match self.get_nibble_from_cache(chunk_x, chunk_y, chunk_z) {
            Some(n) => n,
            None => return,
        };

        for direction in AxisDirection::ONLY_HORIZONTAL {
            let neighbour_off_x = direction.x();
            let neighbour_off_z = direction.z();

            let neighbour_nibble = self.get_nibble_from_cache(
                chunk_x + neighbour_off_x,
                chunk_y,
                chunk_z + neighbour_off_z,
            );

            let neighbour_nibble = match neighbour_nibble {
                Some(n) => n,
                None => continue,
            };

            {
                let curr = curr_nibble.lock().unwrap();
                let neighbour = neighbour_nibble.lock().unwrap();
                if !curr.is_initialised_updating() && !neighbour.is_initialised_updating() {
                    // both are zero, nothing to check.
                    continue;
                }
            }

            // this chunk
            let (inc_x, inc_z, start_x, start_z);
            if neighbour_off_x != 0 {
                // x direction
                inc_x = 0;
                inc_z = 1;
                start_x = if direction.x() < 0 {
                    chunk_x << 4
                } else {
                    (chunk_x << 4) | 15
                };
                start_z = chunk_z << 4;
            } else {
                // z direction
                inc_x = 1;
                inc_z = 0;
                start_z = if neighbour_off_z < 0 {
                    chunk_z << 4
                } else {
                    (chunk_z << 4) | 15
                };
                start_x = chunk_x << 4;
            }

            let mut center_delayed_checks = 0usize;
            let mut neighbour_delayed_checks = 0usize;
            let mut curr_y = chunk_y << 4;
            let max_y = curr_y | 15;
            while curr_y <= max_y {
                let mut i = 0;
                let mut curr_x = start_x;
                let mut curr_z = start_z;
                while i < 16 {
                    let neighbour_x = curr_x + neighbour_off_x;
                    let neighbour_z = curr_z + neighbour_off_z;

                    let current_index = ((curr_x & 15)
                        | (((curr_z & 15)) << 4)
                        | (((curr_y & 15) << 8))) as usize;
                    let current_level =
                        curr_nibble.lock().unwrap().get_updating(current_index);

                    let neighbour_index = ((neighbour_x & 15)
                        | (((neighbour_z & 15)) << 4)
                        | (((curr_y & 15) << 8))) as usize;
                    let neighbour_level =
                        neighbour_nibble.lock().unwrap().get_updating(neighbour_index);

                    // the checks are delayed because the checkBlock method clobbers
                    // light values — см. комментарий в Java.
                    if self.calculate_light_value(
                        light_access,
                        curr_x,
                        curr_y,
                        curr_z,
                        current_level,
                    ) != current_level
                    {
                        self.chunk_check_delayed_center[center_delayed_checks] = current_index as i32;
                        center_delayed_checks += 1;
                    }

                    if self.calculate_light_value(
                        light_access,
                        neighbour_x,
                        curr_y,
                        neighbour_z,
                        neighbour_level,
                    ) != neighbour_level
                    {
                        self.chunk_check_delayed_neighbour[neighbour_delayed_checks] =
                            neighbour_index as i32;
                        neighbour_delayed_checks += 1;
                    }

                    i += 1;
                    curr_x += inc_x;
                    curr_z += inc_z;
                }
                curr_y += 1;
            }

            let current_chunk_off_x = chunk_x << 4;
            let current_chunk_off_z = chunk_z << 4;
            let neighbour_chunk_off_x = (chunk_x + direction.x()) << 4;
            let neighbour_chunk_off_z = (chunk_z + direction.z()) << 4;
            let chunk_off_y = chunk_y << 4;
            for i in 0..center_delayed_checks.max(neighbour_delayed_checks) {
                // try to queue neighbouring data together
                // index = x | (z << 4) | (y << 8)
                if i < center_delayed_checks {
                    let value = self.chunk_check_delayed_center[i] as usize;
                    self.check_block(
                        light_access,
                        current_chunk_off_x | (value & 15) as i32,
                        chunk_off_y | ((value >> 8) & 0xFF) as i32,
                        current_chunk_off_z | (((value >> 4) & 0xF) as i32),
                    );
                }
                if i < neighbour_delayed_checks {
                    let value = self.chunk_check_delayed_neighbour[i] as usize;
                    self.check_block(
                        light_access,
                        neighbour_chunk_off_x | (value & 15) as i32,
                        chunk_off_y | ((value >> 8) & 0xFF) as i32,
                        neighbour_chunk_off_z | (((value >> 4) & 0xF) as i32),
                    );
                }
            }
        }
    }

    /// `checkChunkEdges(lightAccess, chunk, fromSection, toSection)` — базовая.
    pub(crate) fn check_chunk_edges_range_base(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        from_section: i32,
        to_section: i32,
    ) {
        let chunk_pos = chunk.get_pos();
        let (chunk_x, chunk_z) = (chunk_pos.x, chunk_pos.z);

        let mut curr_section_y = to_section;
        while curr_section_y >= from_section {
            self.check_chunk_edge(light_access, chunk_x, curr_section_y, chunk_z);
            curr_section_y -= 1;
        }

        self.perform_light_decrease(light_access);
    }

    /// `checkChunkEdges(lightAccess, chunk, sections)` — базовая.
    pub(crate) fn check_chunk_edges_sections_base(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        sections: &[i16],
    ) {
        let chunk_pos = chunk.get_pos();
        let (chunk_x, chunk_z) = (chunk_pos.x, chunk_pos.z);

        for &section in sections {
            self.check_chunk_edge(light_access, chunk_x, section as i32, chunk_z);
        }

        self.perform_light_decrease(light_access);
    }

    /// `propagateNeighbourLevels` — тянет свет от соседей в increase-очередь.
    pub(crate) fn propagate_neighbour_levels(
        &mut self,
        light_access: &dyn LightChunkGetter,
        chunk: &ChunkAccess,
        from_section: i32,
        to_section: i32,
    ) {
        let chunk_pos = chunk.get_pos();
        let (chunk_x, chunk_z) = (chunk_pos.x, chunk_pos.z);

        let mut curr_section_y = to_section;
        while curr_section_y >= from_section {
            let curr_nibble = self.get_nibble_from_cache(chunk_x, curr_section_y, chunk_z);
            curr_section_y -= 1;
            let curr_nibble = match curr_nibble {
                Some(n) => n,
                None => continue,
            };
            for direction in AxisDirection::ONLY_HORIZONTAL {
                let neighbour_off_x = direction.x();
                let neighbour_off_z = direction.z();

                let neighbour_nibble = self.get_nibble_from_cache(
                    chunk_x + neighbour_off_x,
                    curr_section_y,
                    chunk_z + neighbour_off_z,
                );

                let neighbour_nibble = match neighbour_nibble {
                    Some(n) => n,
                    None => continue,
                };
                if !neighbour_nibble.lock().unwrap().is_initialised_updating() {
                    // can't pull from 0
                    continue;
                }

                // neighbour chunk
                let (inc_x, inc_z, start_x, start_z);
                if neighbour_off_x != 0 {
                    // x direction
                    inc_x = 0;
                    inc_z = 1;
                    start_x = if direction.x() < 0 {
                        (chunk_x << 4) - 1
                    } else {
                        (chunk_x << 4) + 16
                    };
                    start_z = chunk_z << 4;
                } else {
                    // z direction
                    inc_x = 1;
                    inc_z = 0;
                    start_z = if neighbour_off_z < 0 {
                        (chunk_z << 4) - 1
                    } else {
                        (chunk_z << 4) + 16
                    };
                    start_x = chunk_x << 4;
                }

                let propagate_direction = 1i64 << direction.get_opposite().idx();
                let encode_offset = self.coordinate_offset;

                let mut curr_y = curr_section_y << 4;
                let max_y = curr_y | 15;
                while curr_y <= max_y {
                    let mut i = 0;
                    let mut curr_x = start_x;
                    let mut curr_z = start_z;
                    while i < 16 {
                        let level = neighbour_nibble.lock().unwrap().get_updating(
                            ((curr_x & 15)
                                | (((curr_z & 15) << 4))
                                | (((curr_y & 15) << 8))) as usize,
                        );

                        if level > 1 {
                            // something to propagate
                            self.append_to_increase_queue(
                                self.encode_position(curr_x, curr_y, curr_z)
                                    | ((level as i64 & 0xF) << LEVEL_SHIFT)
                                    | (propagate_direction << DIR_SHIFT)
                                    | FLAG_HAS_SIDED_TRANSPARENT_BLOCKS,
                            );
                        }

                        i += 1;
                        curr_x += inc_x;
                        curr_z += inc_z;
                    }
                    curr_y += 1;
                }
            }
            let _ = curr_nibble;
        }
    }

    /// `performLightIncrease` — BFS-распространение света.
    pub(crate) fn perform_light_increase(&mut self, light_access: &dyn LightChunkGetter) {
        let decode_offset_x = -self.encode_offset_x;
        let decode_offset_y = -self.encode_offset_y;
        let decode_offset_z = -self.encode_offset_z;
        let encode_offset = self.coordinate_offset;
        let section_offset = self.chunk_section_index_offset;

        let mut queue_read_index = 0usize;
        let mut queue_length = self.increase_queue_len;
        self.increase_queue_len = 0;

        while queue_read_index < queue_length {
            let queue_value = self.increase_queue[queue_read_index];
            queue_read_index += 1;

            let pos_x = ((queue_value & 63) as i32) + decode_offset_x;
            let pos_z = (((queue_value >> 6) & 63) as i32) + decode_offset_z;
            let pos_y = (((queue_value >> 12) & ((1i64 << 16) - 1)) as i32) + decode_offset_y;
            let propagated_light_level = ((queue_value >> LEVEL_SHIFT) & 0xF) as u8;
            let check_directions =
                &OLD_CHECK_DIRECTIONS[((queue_value >> DIR_SHIFT) & 63) as usize];

            if (queue_value & FLAG_RECHECK_LEVEL) != 0 {
                if self.get_light_level(pos_x, pos_y, pos_z) != propagated_light_level {
                    // not at the level we expect, so something changed.
                    continue;
                }
            } else if (queue_value & FLAG_WRITE_LEVEL) != 0 {
                // these are used to restore block sources after a propagation decrease
                self.set_light_level(pos_x, pos_y, pos_z, propagated_light_level);
            }

            if (queue_value & FLAG_HAS_SIDED_TRANSPARENT_BLOCKS) == 0 {
                // we don't need to worry about our state here.
                for &propagate in check_directions {
                    let off_x = pos_x + propagate.x();
                    let off_y = pos_y + propagate.y();
                    let off_z = pos_z + propagate.z();

                    let section_index =
                        self.chunk_section_index(off_x >> 4, off_y >> 4, off_z >> 4);
                    let local_index = ((off_x & 15)
                        | (((off_z & 15) << 4))
                        | (((off_y & 15) << 8))) as usize;

                    let current_nibble =
                        self.nibble_cache.get(section_index as usize).and_then(|n| n.as_ref()).cloned();
                    let current_level = match &current_nibble {
                        None => continue,
                        Some(n) => {
                            let lvl = n.lock().unwrap().get_updating(local_index);
                            if lvl >= propagated_light_level.saturating_sub(1) {
                                continue; // already at the level we want or unloaded
                            }
                            lvl
                        }
                    };
                    let _ = current_level;

                    let block_state = self.get_block_state_at(section_index, local_index);
                    let mut flags = 0i64;
                    if block_state.starlight_is_conditionally_full_opaque() {
                        let culling_face = block_state
                            .face_occlusion_shape_unleveled(propagate.get_opposite().nms());
                        if crate::world::phys::shapes::face_shape_occludes(&VoxelShape::empty(), &culling_face) {
                            continue;
                        }
                        flags |= FLAG_HAS_SIDED_TRANSPARENT_BLOCKS;
                    }

                    let opacity = block_state.get_light_block();
                    let target_level =
                        propagated_light_level as i32 - opacity.max(1);
                    if target_level <= 0 {
                        continue;
                    }
                    let current_level = self.get_light_level_at(section_index, local_index);
                    if target_level as u8 <= current_level {
                        continue;
                    }

                    if let Some(n) = &current_nibble {
                        n.lock().unwrap().set(local_index, target_level as u8);
                    }
                    self.post_light_update(off_x, off_y, off_z);

                    if target_level > 1 {
                        if queue_length >= self.increase_queue.len() {
                            self.resize_increase_queue();
                        }
                        self.increase_queue[queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((target_level as i64) & 0xF) << LEVEL_SHIFT)
                            | (propagate.everything_but_opposite_direction() << DIR_SHIFT)
                            | flags;
                        queue_length += 1;
                    }
                }
            } else {
                // we actually need to worry about our state here
                let from_block = self.get_block_state(pos_x, pos_y, pos_z);
                for &propagate in check_directions {
                    let off_x = pos_x + propagate.x();
                    let off_y = pos_y + propagate.y();
                    let off_z = pos_z + propagate.z();

                    let from_shape = if from_block.starlight_is_conditionally_full_opaque() {
                        from_block.face_occlusion_shape_unleveled(propagate.nms())
                    } else {
                        VoxelShape::empty()
                    };

                    if !from_shape.is_empty()
                        && crate::world::phys::shapes::face_shape_occludes(&VoxelShape::empty(), &from_shape)
                    {
                        continue;
                    }

                    let section_index =
                        self.chunk_section_index(off_x >> 4, off_y >> 4, off_z >> 4);
                    let local_index = ((off_x & 15)
                        | (((off_z & 15) << 4))
                        | (((off_y & 15) << 8))) as usize;

                    let current_nibble =
                        self.nibble_cache.get(section_index as usize).and_then(|n| n.as_ref()).cloned();
                    let current_level = match &current_nibble {
                        None => continue,
                        Some(n) => {
                            let lvl = n.lock().unwrap().get_updating(local_index);
                            if lvl >= propagated_light_level.saturating_sub(1) {
                                continue; // already at the level we want
                            }
                            lvl
                        }
                    };
                    let _ = current_level;

                    let block_state = self.get_block_state_at(section_index, local_index);
                    let mut flags = 0i64;
                    if block_state.starlight_is_conditionally_full_opaque() {
                        let culling_face = block_state
                            .face_occlusion_shape_unleveled(propagate.get_opposite().nms());
                        if crate::world::phys::shapes::face_shape_occludes(&from_shape, &culling_face) {
                            continue;
                        }
                        flags |= FLAG_HAS_SIDED_TRANSPARENT_BLOCKS;
                    }

                    let opacity = block_state.get_light_block();
                    let target_level = propagated_light_level as i32 - opacity.max(1);
                    if target_level <= 0 {
                        continue;
                    }
                    let current_level = self.get_light_level_at(section_index, local_index);
                    if target_level as u8 <= current_level {
                        continue;
                    }

                    if let Some(n) = &current_nibble {
                        n.lock().unwrap().set(local_index, target_level as u8);
                    }
                    self.post_light_update(off_x, off_y, off_z);

                    if target_level > 1 {
                        if queue_length >= self.increase_queue.len() {
                            self.resize_increase_queue();
                        }
                        self.increase_queue[queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((target_level as i64) & 0xF) << LEVEL_SHIFT)
                            | (propagate.everything_but_opposite_direction() << DIR_SHIFT)
                            | flags;
                        queue_length += 1;
                    }
                }
            }
        }
        let _ = encode_offset;
        let _ = section_offset;
    }

    /// `performLightDecrease` — удаление света + re-propagate источников.
    pub(crate) fn perform_light_decrease(&mut self, light_access: &dyn LightChunkGetter) {
        let decode_offset_x = -self.encode_offset_x;
        let decode_offset_y = -self.encode_offset_y;
        let decode_offset_z = -self.encode_offset_z;
        let section_offset = self.chunk_section_index_offset;
        let emitted_mask = self.emitted_light_mask;

        let mut queue_read_index = 0usize;
        let mut queue_length = self.decrease_queue_len;
        self.decrease_queue_len = 0;
        let mut increase_queue_length = self.increase_queue_len;
        let mut increase_queue = std::mem::take(&mut self.increase_queue);
        let mut queue = std::mem::take(&mut self.decrease_queue);

        while queue_read_index < queue_length {
            let queue_value = queue[queue_read_index];
            queue_read_index += 1;

            let pos_x = ((queue_value & 63) as i32) + decode_offset_x;
            let pos_z = (((queue_value >> 6) & 63) as i32) + decode_offset_z;
            let pos_y = (((queue_value >> 12) & ((1i64 << 16) - 1)) as i32) + decode_offset_y;
            let propagated_light_level = ((queue_value >> LEVEL_SHIFT) & 0xF) as u8;
            let check_directions =
                &OLD_CHECK_DIRECTIONS[((queue_value >> DIR_SHIFT) & 63) as usize];

            if (queue_value & FLAG_HAS_SIDED_TRANSPARENT_BLOCKS) == 0 {
                // we don't need to worry about our state here.
                for &propagate in check_directions {
                    let off_x = pos_x + propagate.x();
                    let off_y = pos_y + propagate.y();
                    let off_z = pos_z + propagate.z();

                    let section_index =
                        self.chunk_section_index(off_x >> 4, off_y >> 4, off_z >> 4);
                    let local_index = ((off_x & 15)
                        | (((off_z & 15) << 4))
                        | (((off_y & 15) << 8))) as usize;

                    let current_nibble =
                        self.nibble_cache.get(section_index as usize).and_then(|n| n.as_ref()).cloned();
                    let light_level = match &current_nibble {
                        None => continue,
                        Some(n) => {
                            let lvl = n.lock().unwrap().get_updating(local_index);
                            if lvl == 0 {
                                continue; // already at lowest (or unloaded)
                            }
                            lvl
                        }
                    };

                    let block_state = self.get_block_state_at(section_index, local_index);
                    let mut flags = 0i64;
                    if block_state.starlight_is_conditionally_full_opaque() {
                        let culling_face = block_state
                            .face_occlusion_shape_unleveled(propagate.get_opposite().nms());
                        if crate::world::phys::shapes::face_shape_occludes(&VoxelShape::empty(), &culling_face) {
                            continue;
                        }
                        flags |= FLAG_HAS_SIDED_TRANSPARENT_BLOCKS;
                    }

                    let opacity = block_state.get_light_block();
                    let target_level =
                        (propagated_light_level as i32 - opacity.max(1)).max(0) as u8;
                    if light_level > target_level {
                        // it looks like another source propagated here, so re-propagate it
                        if increase_queue_length >= increase_queue.len() {
                            let new_len =
                                (increase_queue.len() + (increase_queue.len() >> 1)).max(4);
                            increase_queue.resize(new_len, 0);
                        }
                        increase_queue[increase_queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((light_level as i64) & 0xF) << LEVEL_SHIFT)
                            | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << DIR_SHIFT)
                            | (FLAG_RECHECK_LEVEL | flags);
                        increase_queue_length += 1;
                        continue;
                    }
                    let emitted_light =
                        (block_state.get_light_emission() as i32 & emitted_mask) as u8;
                    if emitted_light != 0 {
                        // re-propagate source
                        // note: do not set recheck level, or else the propagation will fail
                        if increase_queue_length >= increase_queue.len() {
                            let new_len =
                                (increase_queue.len() + (increase_queue.len() >> 1)).max(4);
                            increase_queue.resize(new_len, 0);
                        }
                        increase_queue[increase_queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((emitted_light as i64) & 0xF) << LEVEL_SHIFT)
                            | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << DIR_SHIFT)
                            | (flags | FLAG_WRITE_LEVEL);
                        increase_queue_length += 1;
                    }

                    if let Some(n) = &current_nibble {
                        n.lock().unwrap().set(local_index, 0);
                    }
                    self.post_light_update(off_x, off_y, off_z);

                    if target_level > 0 {
                        if queue_length >= queue.len() {
                            let new_len = (queue.len() + (queue.len() >> 1)).max(4);
                            queue.resize(new_len, 0);
                        }
                        queue[queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((target_level as i64) & 0xF) << LEVEL_SHIFT)
                            | (propagate.everything_but_opposite_direction() << DIR_SHIFT)
                            | flags;
                        queue_length += 1;
                    }
                }
            } else {
                // we actually need to worry about our state here
                let from_block = self.get_block_state(pos_x, pos_y, pos_z);
                for &propagate in check_directions {
                    let off_x = pos_x + propagate.x();
                    let off_y = pos_y + propagate.y();
                    let off_z = pos_z + propagate.z();

                    let from_shape = if from_block.starlight_is_conditionally_full_opaque() {
                        from_block.face_occlusion_shape_unleveled(propagate.nms())
                    } else {
                        VoxelShape::empty()
                    };

                    if !from_shape.is_empty()
                        && crate::world::phys::shapes::face_shape_occludes(&VoxelShape::empty(), &from_shape)
                    {
                        continue;
                    }

                    let section_index =
                        self.chunk_section_index(off_x >> 4, off_y >> 4, off_z >> 4);
                    let local_index = ((off_x & 15)
                        | (((off_z & 15) << 4))
                        | (((off_y & 15) << 8))) as usize;

                    let current_nibble =
                        self.nibble_cache.get(section_index as usize).and_then(|n| n.as_ref()).cloned();
                    let light_level = match &current_nibble {
                        None => continue,
                        Some(n) => {
                            let lvl = n.lock().unwrap().get_updating(local_index);
                            if lvl == 0 {
                                continue; // already at lowest (or unloaded)
                            }
                            lvl
                        }
                    };

                    let block_state = self.get_block_state_at(section_index, local_index);
                    let mut flags = 0i64;
                    if block_state.starlight_is_conditionally_full_opaque() {
                        let culling_face = block_state
                            .face_occlusion_shape_unleveled(propagate.get_opposite().nms());
                        if crate::world::phys::shapes::face_shape_occludes(&from_shape, &culling_face) {
                            continue;
                        }
                        flags |= FLAG_HAS_SIDED_TRANSPARENT_BLOCKS;
                    }

                    let opacity = block_state.get_light_block();
                    let target_level =
                        (propagated_light_level as i32 - opacity.max(1)).max(0) as u8;
                    if light_level > target_level {
                        // it looks like another source propagated here, so re-propagate it
                        if increase_queue_length >= increase_queue.len() {
                            let new_len =
                                (increase_queue.len() + (increase_queue.len() >> 1)).max(4);
                            increase_queue.resize(new_len, 0);
                        }
                        increase_queue[increase_queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((light_level as i64) & 0xF) << LEVEL_SHIFT)
                            | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << DIR_SHIFT)
                            | (FLAG_RECHECK_LEVEL | flags);
                        increase_queue_length += 1;
                        continue;
                    }
                    let emitted_light =
                        (block_state.get_light_emission() as i32 & emitted_mask) as u8;
                    if emitted_light != 0 {
                        // re-propagate source
                        if increase_queue_length >= increase_queue.len() {
                            let new_len =
                                (increase_queue.len() + (increase_queue.len() >> 1)).max(4);
                            increase_queue.resize(new_len, 0);
                        }
                        increase_queue[increase_queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((emitted_light as i64) & 0xF) << LEVEL_SHIFT)
                            | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << DIR_SHIFT)
                            | (flags | FLAG_WRITE_LEVEL);
                        increase_queue_length += 1;
                    }

                    if let Some(n) = &current_nibble {
                        n.lock().unwrap().set(local_index, 0);
                    }
                    self.post_light_update(off_x, off_y, off_z);

                    if target_level > 0 {
                        // we actually need to propagate 0 just in case we find a neighbour...
                        if queue_length >= queue.len() {
                            let new_len = (queue.len() + (queue.len() >> 1)).max(4);
                            queue.resize(new_len, 0);
                        }
                        queue[queue_length] = self.encode_position(off_x, off_y, off_z)
                            | (((target_level as i64) & 0xF) << LEVEL_SHIFT)
                            | (propagate.everything_but_opposite_direction() << DIR_SHIFT)
                            | flags;
                        queue_length += 1;
                    }
                }
            }
        }

        // propagate sources we clobbered
        self.increase_queue = increase_queue;
        self.increase_queue_len = increase_queue_length;
        self.decrease_queue = queue;
        self.perform_light_increase(light_access);
        let _ = section_offset;
    }

}
