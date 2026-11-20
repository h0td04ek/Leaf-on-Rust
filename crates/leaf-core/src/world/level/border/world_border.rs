//! `net.minecraft.world.level.border.WorldBorder` — граница мира:
//! центр, размер, lerp-анимация, предупреждения, слушатели.
//!
//! Отличия от Java:
//! - Paper-события Bukkit (`WorldBorder*Event`) не вызываются —
//!   plugin API отложен; точки вызова отмечены `// Paper:`.
//! - `tick()` принимает `current_tick` параметром вместо чтения
//!   `MinecraftServer.currentTick`.
//! - `world` (для `applySettings` gameTime) заменён на
//!   `game_time`-поле, выставляемое через `set_game_time`.
//! - `BorderExtent` получает копию параметров (`BorderParams`)
//!   вместо `&WorldBorder` — иначе aliasing `self.extent.f(self)`.

use std::any::Any;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::world::level::saveddata::saved_data::{SavedData, SavedDataBase};
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::shapes::voxel_shape::{BooleanOp, VoxelShape};
use crate::world::phys::vec3::Vec3;

use super::border_change_listener::BorderChangeListener;
use super::border_status::BorderStatus;

/// Максимальный размер границы.
pub const MAX_SIZE: f64 = 5.999997E7;
/// Максимальная координата центра.
pub const MAX_CENTER_COORDINATE: f64 = 2.9999984E7;

/// `WorldBorder.Settings` — снапшот состояния для NBT/команд.
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub center_x: f64,
    pub center_z: f64,
    pub damage_per_block: f64,
    pub safe_zone: f64,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub size: f64,
    pub lerp_time: i64,
    pub lerp_target: f64,
}

impl Settings {
    /// `Settings.DEFAULT`.
    pub const DEFAULT: Self = Self {
        center_x: 0.0,
        center_z: 0.0,
        damage_per_block: 0.2,
        safe_zone: 5.0,
        warning_blocks: 5,
        warning_time: 300,
        size: 5.999997E7,
        lerp_time: 0,
        lerp_target: 0.0,
    };

    /// `new Settings(border)`.
    pub fn from_border(border: &WorldBorder) -> Self {
        Self {
            center_x: border.center_x,
            center_z: border.center_z,
            damage_per_block: border.damage_per_block,
            safe_zone: border.safe_zone,
            warning_blocks: border.warning_blocks,
            warning_time: border.warning_time,
            size: border.extent.get_size(),
            lerp_time: border.extent.get_lerp_time(),
            lerp_target: border.extent.get_lerp_target(),
        }
    }
}

/// Параметры, нужные `BorderExtent` (центр + absoluteMaxSize) —
/// копия вместо `&WorldBorder` для избежания aliasing.
#[derive(Debug, Clone, Copy)]
struct BorderParams {
    center_x: f64,
    center_z: f64,
    absolute_max_size: i32,
}

/// `WorldBorder.BorderExtent` — текущий режим границы.
#[derive(Debug)]
enum BorderExtent {
    Moving(MovingBorderExtent),
    Static(StaticBorderExtent),
}

impl BorderExtent {
    fn get_min_x(&self, p: BorderParams, partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Moving(m) => m.get_min_x(p, partial_tick),
            BorderExtent::Static(s) => s.min_x,
        }
    }
    fn get_min_z(&self, p: BorderParams, partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Moving(m) => m.get_min_z(p, partial_tick),
            BorderExtent::Static(s) => s.min_z,
        }
    }
    fn get_max_x(&self, p: BorderParams, partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Moving(m) => m.get_max_x(p, partial_tick),
            BorderExtent::Static(s) => s.max_x,
        }
    }
    fn get_max_z(&self, p: BorderParams, partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Moving(m) => m.get_max_z(p, partial_tick),
            BorderExtent::Static(s) => s.max_z,
        }
    }
    fn get_size(&self) -> f64 {
        match self {
            BorderExtent::Moving(m) => m.size,
            BorderExtent::Static(s) => s.size,
        }
    }
    fn get_lerp_speed(&self) -> f64 {
        match self {
            BorderExtent::Moving(m) => m.get_lerp_speed(),
            BorderExtent::Static(_) => 0.0,
        }
    }
    fn get_lerp_time(&self) -> i64 {
        match self {
            BorderExtent::Moving(m) => m.lerp_progress,
            BorderExtent::Static(_) => 0,
        }
    }
    fn get_lerp_target(&self) -> f64 {
        match self {
            BorderExtent::Moving(m) => m.to,
            BorderExtent::Static(s) => s.size,
        }
    }
    fn get_status(&self) -> BorderStatus {
        match self {
            BorderExtent::Moving(m) => m.get_status(),
            BorderExtent::Static(_) => BorderStatus::Stationary,
        }
    }
    fn on_absolute_max_size_change(&mut self, p: BorderParams) {
        if let BorderExtent::Static(s) = self {
            s.update_box(p);
        }
    }
    fn on_center_change(&mut self, p: BorderParams) {
        if let BorderExtent::Static(s) = self {
            s.update_box(p);
        }
    }
    /// `update()` — тик анимации. Возвращает `(новый extent,
    /// завершилась ли lerp-анимация)` — для `setDirty` снаружи.
    fn update(&self, p: BorderParams) -> (BorderExtent, bool) {
        match self {
            BorderExtent::Static(s) => (
                BorderExtent::Static(s.clone()),
                false,
            ),
            BorderExtent::Moving(m) => {
                let mut m = m.clone();
                m.lerp_progress -= 1;
                m.previous_size = m.size;
                m.size = m.calculate_size();
                if m.lerp_progress <= 0 {
                    // Paper: WorldBorderBoundsChangeFinishEvent — отложено.
                    (BorderExtent::Static(StaticBorderExtent::new(m.to, p)), true)
                } else {
                    (BorderExtent::Moving(m), false)
                }
            }
        }
    }
    /// `getCollisionShape()`.
    fn get_collision_shape(&self, p: BorderParams) -> VoxelShape {
        match self {
            BorderExtent::Moving(m) => m.get_collision_shape(p),
            BorderExtent::Static(s) => s.shape.clone(),
        }
    }
}

/// `WorldBorder.MovingBorderExtent`.
#[derive(Debug, Clone)]
struct MovingBorderExtent {
    from: f64,
    to: f64,
    lerp_end: i64,
    lerp_begin: i64,
    lerp_duration: f64,
    lerp_progress: i64,
    size: f64,
    previous_size: f64,
}

impl MovingBorderExtent {
    fn new(from: f64, to: f64, lerp_duration: i64, lerp_begin: i64) -> Self {
        let mut m = Self {
            from,
            to,
            lerp_end: lerp_begin + lerp_duration,
            lerp_begin,
            lerp_duration: lerp_duration as f64,
            lerp_progress: lerp_duration,
            size: 0.0,
            previous_size: 0.0,
        };
        let d = m.calculate_size();
        m.size = d;
        m.previous_size = d;
        m
    }

    fn lerped_size(&self, partial_tick: f32) -> f64 {
        crate::util::mth::lerp_f64(
            partial_tick as f64,
            self.previous_size,
            self.size,
        )
    }

    fn clamped(&self, p: BorderParams, v: f64) -> f64 {
        crate::util::mth::clamp_f64(
            v,
            -(p.absolute_max_size as f64),
            p.absolute_max_size as f64,
        )
    }

    fn get_min_x(&self, p: BorderParams, partial_tick: f32) -> f64 {
        self.clamped(p, p.center_x - self.lerped_size(partial_tick) / 2.0)
    }

    fn get_min_z(&self, p: BorderParams, partial_tick: f32) -> f64 {
        self.clamped(p, p.center_z - self.lerped_size(partial_tick) / 2.0)
    }

    fn get_max_x(&self, p: BorderParams, partial_tick: f32) -> f64 {
        self.clamped(p, p.center_x + self.lerped_size(partial_tick) / 2.0)
    }

    fn get_max_z(&self, p: BorderParams, partial_tick: f32) -> f64 {
        self.clamped(p, p.center_z + self.lerped_size(partial_tick) / 2.0)
    }

    fn calculate_size(&self) -> f64 {
        let d = (self.lerp_duration - self.lerp_progress as f64) / self.lerp_duration;
        if d < 1.0 {
            crate::util::mth::lerp_f64(d, self.from, self.to)
        } else {
            self.to
        }
    }

    fn get_lerp_speed(&self) -> f64 {
        (self.from - self.to).abs() / (self.lerp_end - self.lerp_begin) as f64
    }

    fn get_status(&self) -> BorderStatus {
        if self.to < self.from {
            BorderStatus::Shrinking
        } else {
            BorderStatus::Growing
        }
    }

    fn get_collision_shape(&self, p: BorderParams) -> VoxelShape {
        collision_shape_outside(
            self.get_min_x(p, 0.0),
            self.get_min_z(p, 0.0),
            self.get_max_x(p, 0.0),
            self.get_max_z(p, 0.0),
        )
    }
}

/// `WorldBorder.StaticBorderExtent`.
#[derive(Debug, Clone)]
struct StaticBorderExtent {
    size: f64,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
    shape: VoxelShape,
}

impl StaticBorderExtent {
    fn new(size: f64, p: BorderParams) -> Self {
        let mut s = Self {
            size,
            min_x: 0.0,
            min_z: 0.0,
            max_x: 0.0,
            max_z: 0.0,
            shape: VoxelShape::empty(),
        };
        s.update_box(p);
        s
    }

    fn update_box(&mut self, p: BorderParams) {
        let m = p.absolute_max_size as f64;
        let clamp = |v: f64| crate::util::mth::clamp_f64(v, -m, m);
        self.min_x = clamp(p.center_x - self.size / 2.0);
        self.min_z = clamp(p.center_z - self.size / 2.0);
        self.max_x = clamp(p.center_x + self.size / 2.0);
        self.max_z = clamp(p.center_z + self.size / 2.0);
        self.shape = collision_shape_outside(self.min_x, self.min_z, self.max_x, self.max_z);
    }
}

/// `Shapes.join(Shapes.INFINITY, Shapes.box(minX,-inf,minZ,
/// maxX,+inf,maxZ), ONLY_FIRST)` — коллизионная область ВНЕ
/// границы. Наша `VoxelShape` — список AABB, дополнение бокса
/// непредставимо напрямую; эквивалент — `outer \ inner`
/// (`BooleanOp::Not`) в пределах ±`MAX_CENTER_COORDINATE*2` —
/// физически достижимой области мира.
fn collision_shape_outside(min_x: f64, min_z: f64, max_x: f64, max_z: f64) -> VoxelShape {
    let bound = MAX_CENTER_COORDINATE * 2.0;
    let outer = VoxelShape::of(Aabb::new(
        -bound,
        f64::NEG_INFINITY,
        -bound,
        bound,
        f64::INFINITY,
        bound,
    ));
    let inner = VoxelShape::of(Aabb::new(
        min_x.floor(),
        f64::NEG_INFINITY,
        min_z.floor(),
        max_x.ceil(),
        f64::INFINITY,
        max_z.ceil(),
    ));
    outer.join(&inner, BooleanOp::Not)
}

/// `WorldBorder` — `SavedData` границы мира.
pub struct WorldBorder {
    settings: Settings,
    initialized: bool,
    listeners: Vec<Arc<dyn BorderChangeListener>>,
    damage_per_block: f64,
    safe_zone: f64,
    center_x: f64,
    center_z: f64,
    warning_time: i32,
    warning_blocks: i32,
    absolute_max_size: i32,
    extent: BorderExtent,
    saved: SavedDataBase,
    /// Paper `lastTick` — дедупликация тиков виртуальных границ.
    last_tick: i64,
    /// Game time мира для `applySettings` (Paper `world`).
    game_time: i64,
}

impl WorldBorder {
    /// `new WorldBorder()`.
    pub fn new() -> Self {
        Self::with_settings(Settings::DEFAULT)
    }

    /// `new WorldBorder(settings)`.
    pub fn with_settings(settings: Settings) -> Self {
        Self {
            settings,
            initialized: false,
            listeners: Vec::new(),
            damage_per_block: 0.2,
            safe_zone: 5.0,
            center_x: 0.0,
            center_z: 0.0,
            warning_time: 15,
            warning_blocks: 5,
            absolute_max_size: 29999984,
            extent: BorderExtent::Static(StaticBorderExtent::new(
                MAX_SIZE,
                BorderParams {
                    center_x: 0.0,
                    center_z: 0.0,
                    absolute_max_size: 29999984,
                },
            )),
            saved: SavedDataBase::default(),
            last_tick: -1,
            game_time: 0,
        }
    }

    fn params(&self) -> BorderParams {
        BorderParams {
            center_x: self.center_x,
            center_z: self.center_z,
            absolute_max_size: self.absolute_max_size,
        }
    }

    /// `isWithinBounds(BlockPos)`.
    pub fn is_within_bounds_pos(&self, pos: &BlockPos) -> bool {
        self.is_within_bounds(pos.x as f64, pos.z as f64)
    }

    /// `isWithinBounds(Vec3)`.
    pub fn is_within_bounds_vec(&self, pos: &Vec3) -> bool {
        self.is_within_bounds(pos.x, pos.z)
    }

    /// `isWithinBounds(ChunkPos)` — оба угла чанка.
    pub fn is_within_bounds_chunk(&self, chunk_pos: &ChunkPos) -> bool {
        self.is_within_bounds(
            (chunk_pos.x * 16) as f64,
            (chunk_pos.z * 16) as f64,
        ) && self.is_within_bounds(
            (chunk_pos.x * 16 + 15) as f64,
            (chunk_pos.z * 16 + 15) as f64,
        )
    }

    /// Paper `isBlockInBounds`.
    pub fn is_block_in_bounds(&self, x: i32, z: i32) -> bool {
        self.is_within_bounds_pos(&BlockPos::new(x, 64, z))
    }

    /// Paper `isChunkInBounds`.
    pub fn is_chunk_in_bounds(&self, chunk_x: i32, chunk_z: i32) -> bool {
        self.is_within_bounds_pos(&BlockPos::new(
            (chunk_x << 4) + 15,
            64,
            (chunk_z << 4) + 15,
        ))
    }

    /// `isWithinBounds(AABB)`.
    pub fn is_within_bounds_aabb(&self, b: &Aabb) -> bool {
        self.is_within_bounds_pair(b.min_x, b.min_z, b.max_x - 1.0E-5, b.max_z - 1.0E-5)
    }

    fn is_within_bounds_pair(&self, x1: f64, z1: f64, x2: f64, z2: f64) -> bool {
        self.is_within_bounds(x1, z1) && self.is_within_bounds(x2, z2)
    }

    /// `isWithinBounds(x, z)`.
    pub fn is_within_bounds(&self, x: f64, z: f64) -> bool {
        self.is_within_bounds_offset(x, z, 0.0)
    }

    /// `isWithinBounds(x, z, offset)`.
    pub fn is_within_bounds_offset(&self, x: f64, z: f64, offset: f64) -> bool {
        x >= self.get_min_x() - offset
            && x < self.get_max_x() + offset
            && z >= self.get_min_z() - offset
            && z < self.get_max_z() + offset
    }

    /// `clampToBounds(BlockPos)`.
    pub fn clamp_to_bounds_pos(&self, pos: &BlockPos) -> BlockPos {
        self.clamp_to_bounds(pos.x as f64, pos.y as f64, pos.z as f64)
    }

    /// `clampToBounds(Vec3)`.
    pub fn clamp_to_bounds_vec(&self, pos: &Vec3) -> BlockPos {
        self.clamp_to_bounds(pos.x, pos.y, pos.z)
    }

    /// `clampToBounds(x, y, z)`.
    pub fn clamp_to_bounds(&self, x: f64, y: f64, z: f64) -> BlockPos {
        BlockPos::containing_vec3(&self.clamp_vec3_to_bound(x, y, z))
    }

    /// `clampVec3ToBound(Vec3)`.
    pub fn clamp_vec3_to_bound_vec(&self, vec3: &Vec3) -> Vec3 {
        self.clamp_vec3_to_bound(vec3.x, vec3.y, vec3.z)
    }

    /// `clampVec3ToBound(x, y, z)`.
    pub fn clamp_vec3_to_bound(&self, x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(
            crate::util::mth::clamp_f64(x, self.get_min_x(), self.get_max_x() - 1.0E-5),
            y,
            crate::util::mth::clamp_f64(z, self.get_min_z(), self.get_max_z() - 1.0E-5),
        )
    }

    /// `getDistanceToBorder(Entity)`.
    pub fn get_distance_to_border_entity(
        &self,
        entity: &crate::world::entity::entity::Entity,
    ) -> f64 {
        let p = entity.position();
        self.get_distance_to_border(p.x, p.z)
    }

    /// `getCollisionShape()`.
    pub fn get_collision_shape(&self) -> VoxelShape {
        self.extent.get_collision_shape(self.params())
    }

    /// `getDistanceToBorder(x, z)`.
    pub fn get_distance_to_border(&self, x: f64, z: f64) -> f64 {
        let d = z - self.get_min_z();
        let d1 = self.get_max_z() - z;
        let d2 = x - self.get_min_x();
        let d3 = self.get_max_x() - x;
        let mut min = d2.min(d3);
        min = min.min(d);
        min.min(d1)
    }

    /// `isInsideCloseToBorder(Entity, AABB)`.
    pub fn is_inside_close_to_border(
        &self,
        entity: &crate::world::entity::entity::Entity,
        bounds: &Aabb,
    ) -> bool {
        let max = (bounds.max_x - bounds.min_x)
            .abs()
            .max((bounds.max_z - bounds.min_z).abs())
            .max(1.0);
        self.get_distance_to_border_entity(entity) < max * 2.0
            && self.is_within_bounds_offset(entity.position().x, entity.position().z, max)
    }

    /// `getStatus()`.
    pub fn get_status(&self) -> BorderStatus {
        self.extent.get_status()
    }

    /// `getMinX()`.
    pub fn get_min_x(&self) -> f64 {
        self.get_min_x_partial(0.0)
    }

    /// `getMinX(partialTick)`.
    pub fn get_min_x_partial(&self, partial_tick: f32) -> f64 {
        self.extent.get_min_x(self.params(), partial_tick)
    }

    /// `getMinZ()`.
    pub fn get_min_z(&self) -> f64 {
        self.get_min_z_partial(0.0)
    }

    /// `getMinZ(partialTick)`.
    pub fn get_min_z_partial(&self, partial_tick: f32) -> f64 {
        self.extent.get_min_z(self.params(), partial_tick)
    }

    /// `getMaxX()`.
    pub fn get_max_x(&self) -> f64 {
        self.get_max_x_partial(0.0)
    }

    /// `getMaxX(partialTick)`.
    pub fn get_max_x_partial(&self, partial_tick: f32) -> f64 {
        self.extent.get_max_x(self.params(), partial_tick)
    }

    /// `getMaxZ()`.
    pub fn get_max_z(&self) -> f64 {
        self.get_max_z_partial(0.0)
    }

    /// `getMaxZ(partialTick)`.
    pub fn get_max_z_partial(&self, partial_tick: f32) -> f64 {
        self.extent.get_max_z(self.params(), partial_tick)
    }

    /// `getCenterX()`.
    pub fn get_center_x(&self) -> f64 {
        self.center_x
    }

    /// `getCenterZ()`.
    pub fn get_center_z(&self) -> f64 {
        self.center_z
    }

    /// `setCenter(x, z)`.
    pub fn set_center(&mut self, x: f64, z: f64) {
        // Paper: WorldBorderCenterChangeEvent — отложено.
        self.center_x = x;
        self.center_z = z;
        let p = self.params();
        self.extent.on_center_change(p);
        self.set_dirty();
        for l in self.listeners.clone() {
            l.on_set_center(self, x, z);
        }
    }

    /// `getSize()`.
    pub fn get_size(&self) -> f64 {
        self.extent.get_size()
    }

    /// `getLerpTime()`.
    pub fn get_lerp_time(&self) -> i64 {
        self.extent.get_lerp_time()
    }

    /// `getLerpTarget()`.
    pub fn get_lerp_target(&self) -> f64 {
        self.extent.get_lerp_target()
    }

    /// `setSize(size)` — vanilla: `Mth.clamp(size, 1.0, MAX_SIZE)`.
    pub fn set_size(&mut self, size: f64) {
        // Paper: WorldBorderBoundsChangeEvent — отложено.
        let size = crate::util::mth::clamp_f64(size, 1.0, MAX_SIZE);
        self.extent = BorderExtent::Static(StaticBorderExtent::new(size, self.params()));
        self.set_dirty();
        for l in self.listeners.clone() {
            l.on_set_size(self, size);
        }
    }

    /// `lerpSizeBetween(oldSize, newSize, time, startTime)` —
    /// vanilla не клампит `newSize`.
    pub fn lerp_size_between(&mut self, old_size: f64, new_size: f64, time: i64, start_time: i64) {
        // Paper: WorldBorderBoundsChangeEvent — отложено.
        self.extent = if old_size == new_size {
            BorderExtent::Static(StaticBorderExtent::new(new_size, self.params()))
        } else {
            BorderExtent::Moving(MovingBorderExtent::new(old_size, new_size, time, start_time))
        };
        self.set_dirty();
        for l in self.listeners.clone() {
            l.on_lerp_size(self, old_size, new_size, time, start_time);
        }
    }

    /// `getListeners()`.
    pub fn get_listeners(&self) -> Vec<Arc<dyn BorderChangeListener>> {
        self.listeners.clone()
    }

    /// `addListener`.
    pub fn add_listener(&mut self, listener: Arc<dyn BorderChangeListener>) {
        if self.listeners.iter().any(|l| Arc::ptr_eq(l, &listener)) {
            return;
        }
        self.listeners.push(listener);
    }

    /// `removeListener`.
    pub fn remove_listener(&mut self, listener: &Arc<dyn BorderChangeListener>) {
        self.listeners.retain(|l| !Arc::ptr_eq(l, listener));
    }

    /// `setAbsoluteMaxSize`.
    pub fn set_absolute_max_size(&mut self, size: i32) {
        self.absolute_max_size = size;
        let p = self.params();
        self.extent.on_absolute_max_size_change(p);
    }

    /// `getAbsoluteMaxSize`.
    pub fn get_absolute_max_size(&self) -> i32 {
        self.absolute_max_size
    }

    /// `getSafeZone`.
    pub fn get_safe_zone(&self) -> f64 {
        self.safe_zone
    }

    /// `setSafeZone`.
    pub fn set_safe_zone(&mut self, safe_zone: f64) {
        self.safe_zone = safe_zone;
        self.set_dirty();
        for l in self.listeners.clone() {
            l.on_set_safe_zone(self, safe_zone);
        }
    }

    /// `getDamagePerBlock`.
    pub fn get_damage_per_block(&self) -> f64 {
        self.damage_per_block
    }

    /// `setDamagePerBlock`.
    pub fn set_damage_per_block(&mut self, damage_per_block: f64) {
        self.damage_per_block = damage_per_block;
        self.set_dirty();
        for l in self.listeners.clone() {
            l.on_set_damage_per_block(self, damage_per_block);
        }
    }

    /// `getLerpSpeed`.
    pub fn get_lerp_speed(&self) -> f64 {
        self.extent.get_lerp_speed()
    }

    /// `getWarningTime`.
    pub fn get_warning_time(&self) -> i32 {
        self.warning_time
    }

    /// `setWarningTime`.
    pub fn set_warning_time(&mut self, warning_time: i32) {
        self.warning_time = warning_time;
        self.set_dirty();
        for l in self.listeners.clone() {
            l.on_set_warning_time(self, warning_time);
        }
    }

    /// `getWarningBlocks`.
    pub fn get_warning_blocks(&self) -> i32 {
        self.warning_blocks
    }

    /// `setWarningBlocks`.
    pub fn set_warning_blocks(&mut self, warning_blocks: i32) {
        self.warning_blocks = warning_blocks;
        self.set_dirty();
        for l in self.listeners.clone() {
            l.on_set_warning_blocks(self, warning_blocks);
        }
    }

    /// `tick()` — Paper-вариант с дедупликацией по тику сервера.
    /// `current_tick` — `MinecraftServer.currentTick`.
    pub fn tick(&mut self, current_tick: i64) {
        if self.last_tick == current_tick {
            return;
        }
        self.last_tick = current_tick;
        let (new_extent, finished) = self.extent.update(self.params());
        self.extent = new_extent;
        if finished {
            self.set_dirty();
        }
    }

    /// `applyInitialSettings(lerpBegin)`.
    pub fn apply_initial_settings(&mut self, lerp_begin: i64) {
        if !self.initialized {
            self.set_center(self.settings.center_x, self.settings.center_z);
            self.set_damage_per_block(self.settings.damage_per_block);
            self.set_safe_zone(self.settings.safe_zone);
            self.set_warning_blocks(self.settings.warning_blocks);
            self.set_warning_time(self.settings.warning_time);
            if self.settings.lerp_time > 0 {
                self.lerp_size_between(
                    self.settings.size,
                    self.settings.lerp_target,
                    self.settings.lerp_time,
                    lerp_begin,
                );
            } else {
                self.set_size(self.settings.size);
            }
            self.initialized = true;
        }
    }

    /// Paper `applySettings(settings)` — применить без initialized-гейта.
    pub fn apply_settings(&mut self, settings: Settings) {
        self.set_center(settings.center_x, settings.center_z);
        self.set_damage_per_block(settings.damage_per_block);
        self.set_safe_zone(settings.safe_zone);
        self.set_warning_blocks(settings.warning_blocks);
        self.set_warning_time(settings.warning_time);
        if settings.lerp_time > 0 {
            self.lerp_size_between(
                settings.size,
                settings.lerp_target,
                settings.lerp_time,
                self.game_time,
            );
        } else {
            self.set_size(settings.size);
        }
    }

    /// Установить game time (Paper `world.getGameTime()`).
    pub fn set_game_time(&mut self, game_time: i64) {
        self.game_time = game_time;
    }

    /// Текущий `Settings`-снапшот.
    pub fn settings(&self) -> Settings {
        Settings::from_border(self)
    }

    /// Границы extent `(minX, minZ, maxX, maxZ)`.
    pub fn bounds(&self) -> (f64, f64, f64, f64) {
        (
            self.get_min_x(),
            self.get_min_z(),
            self.get_max_x(),
            self.get_max_z(),
        )
    }
}

impl Default for WorldBorder {
    fn default() -> Self {
        Self::new()
    }
}

impl SavedData for WorldBorder {
    fn is_dirty(&self) -> bool {
        self.saved.is_dirty()
    }
    fn set_dirty_flag(&mut self, dirty: bool) {
        self.saved.set_dirty_flag(dirty);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn border_static_bounds() {
        let mut b = WorldBorder::new();
        b.apply_initial_settings(0);
        // DEFAULT size = MAX_SIZE -> граница покрывает всё.
        assert!(b.is_within_bounds(0.0, 0.0));
        assert_eq!(b.get_status(), BorderStatus::Stationary);
    }

    #[test]
    fn border_set_size_and_clamp() {
        let mut b = WorldBorder::new();
        b.set_size(100.0);
        assert_eq!(b.get_size(), 100.0);
        assert!(!b.is_within_bounds(60.0, 0.0));
        assert!(b.is_within_bounds(49.0, 0.0));
        let c = b.clamp_vec3_to_bound(60.0, 5.0, -60.0);
        assert!(c.x <= 49.99999);
        assert_eq!(c.z, -50.0);
    }

    #[test]
    fn border_lerp() {
        let mut b = WorldBorder::new();
        b.set_center(0.0, 0.0);
        b.lerp_size_between(100.0, 200.0, 10, 0);
        assert_eq!(b.get_status(), BorderStatus::Growing);
        for t in 0..10 {
            b.tick(t);
        }
        assert_eq!(b.get_status(), BorderStatus::Stationary);
        assert_eq!(b.get_size(), 200.0);
    }
}
