//! Перенос `net.minecraft.world.level` — блоки, уровни, чанки.

pub mod block;
pub mod game_type;
pub mod chunk_pos;
pub mod saveddata;
pub mod level_reader;
pub mod clip;

/// `net.minecraft.world.level.Level` — маркер типа для `ResourceKey<LevelKey>`
/// и GlobalPos. Полная реализация Level — отдельный этап порта.
#[derive(Debug)]
pub struct LevelKey;

impl LevelKey {
    /// `RESOURCE_KEY_CODEC` — `ResourceKey.codec(Registries.DIMENSION)`.
    /// (Кодек регистрируется в network-слое.)
    /// `OVERWORLD`.
    pub fn overworld() -> crate::resources::ResourceKey<LevelKey> {
        crate::resources::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            crate::resources::identifier::Identifier::with_default_namespace("overworld").unwrap(),
        )
    }

    /// `NETHER`.
    pub fn nether() -> crate::resources::ResourceKey<LevelKey> {
        crate::resources::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            crate::resources::identifier::Identifier::with_default_namespace("the_nether").unwrap(),
        )
    }

    /// `END`.
    pub fn end() -> crate::resources::ResourceKey<LevelKey> {
        crate::resources::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            crate::resources::identifier::Identifier::with_default_namespace("the_end").unwrap(),
        )
    }

    /// `MAX_LEVEL_SIZE`.
    pub const MAX_LEVEL_SIZE: i32 = 30_000_000;
    /// `LONG_PARTICLE_CLIP_RANGE`.
    pub const LONG_PARTICLE_CLIP_RANGE: i32 = 512;
    /// `SHORT_PARTICLE_CLIP_RANGE`.
    pub const SHORT_PARTICLE_CLIP_RANGE: i32 = 32;
    /// `MAX_BRIGHTNESS`.
    pub const MAX_BRIGHTNESS: i32 = 15;
    /// `MAX_ENTITY_SPAWN_Y`.
    pub const MAX_ENTITY_SPAWN_Y: i32 = 20_000_000;
    /// `MIN_ENTITY_SPAWN_Y`.
    pub const MIN_ENTITY_SPAWN_Y: i32 = -20_000_000;
}

/// `LevelData.RespawnData` — точка возрождения + yaw/pitch.
/// (В Java — nested record в `LevelData`; интерфейс LevelData
/// портируется вместе с Level.)
#[derive(Clone, Debug, PartialEq)]
pub struct RespawnData {
    /// `globalPos`.
    pub global_pos: crate::core::global_pos::GlobalPos,
    /// `yaw`.
    pub yaw: f32,
    /// `pitch`.
    pub pitch: f32,
}

impl RespawnData {
    /// `DEFAULT` — overworld spawn (0,0,0), yaw/pitch 0.
    pub fn default_respawn() -> Self {
        Self::new(
            crate::core::global_pos::GlobalPos::of(LevelKey::overworld(), crate::core::block_pos::BlockPos::ZERO),
            0.0,
            0.0,
        )
    }

    /// `new RespawnData(globalPos, yaw, pitch)`.
    pub fn new(global_pos: crate::core::global_pos::GlobalPos, yaw: f32, pitch: f32) -> Self {
        Self { global_pos, yaw, pitch }
    }

    /// `of(dimension, pos, yaw, pitch)` — clamp углов.
    pub fn of(
        dimension: crate::resources::ResourceKey<crate::world::level::LevelKey>,
        pos: crate::core::block_pos::BlockPos,
        yaw: f32,
        pitch: f32,
    ) -> Self {
        Self::new(
            crate::core::global_pos::GlobalPos::of(dimension, pos),
            crate::util::mth::wrap_degrees_f32(yaw),
            crate::util::mth::clamp_f32(pitch, -90.0, 90.0),
        )
    }

    /// `STREAM_CODEC` — GlobalPos + FLOAT×2.
    pub fn read(buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf) -> Self {
        let global_pos = buf.read_global_pos();
        let yaw = buf.read_float();
        let pitch = buf.read_float();
        Self::new(global_pos, yaw, pitch)
    }

    pub fn write(&self, buf: &mut crate::network::friendly_byte_buf::FriendlyByteBuf) {
        buf.write_global_pos(&self.global_pos);
        buf.write_float(self.yaw);
        buf.write_float(self.pitch);
    }

    /// `dimension()`.
    pub fn dimension(&self) -> &crate::resources::ResourceKey<LevelKey> {
        &self.global_pos.dimension
    }

    /// `pos()`.
    pub fn pos(&self) -> crate::core::block_pos::BlockPos {
        self.global_pos.pos
    }

    /// `withLevel(dimension)` (Paper).
    pub fn with_level(&self, dimension: crate::resources::ResourceKey<LevelKey>) -> Self {
        Self::new(
            crate::core::global_pos::GlobalPos::of(dimension, self.global_pos.pos),
            self.yaw,
            self.pitch,
        )
    }

    /// `positionEquals(other)` (Paper) — pos+yaw+pitch без измерения.
    pub fn position_equals(&self, other: &RespawnData) -> bool {
        self.global_pos.pos == other.global_pos.pos
            && self.yaw == other.yaw
            && self.pitch == other.pitch
    }
}
pub mod chunk;
pub mod chunk_source;
pub mod biome;
pub mod moon_phase;
pub mod lighting;
pub mod gameevent;
pub mod color_map_color_util;
pub mod grass_color;
pub mod foliage_color;
pub mod dry_foliage_color;
pub mod light_layer;
pub mod level_height_accessor;
pub mod levelgen;
pub mod dimension;
pub mod storage;
pub mod pathfinder;
pub mod path_navigation_region;
pub mod explosion;
pub mod spawn_data;
pub mod base_spawner;
pub mod potential_calculator;
pub mod local_mob_cap_calculator;
pub mod natural_spawner;
pub mod noise_column;
pub mod scheduled_tick_access;
pub mod level_writer;
pub mod level_simulated;
pub mod common_level_accessor;
pub mod entity_getter;
pub mod redstone;
pub mod level_accessor;
pub mod clip_context;
pub mod clip_block_state_context;
pub mod block_getter;
pub mod border;
pub mod collision_getter;
pub mod block_collisions;
pub mod gamerules;
pub mod signal_getter;
pub mod item_like;
pub mod block_event_data;
pub mod data_pack_config;
pub mod world_data_configuration;
pub mod level_settings;
pub mod server_level_accessor;
pub mod world_gen_level;
pub mod color_resolver;
pub mod block_and_tint_getter;
pub mod empty_block_getter;
pub mod empty_block_and_tint_getter;



// ============================================================================
// net.minecraft.world.level.Level — общий интерфейс игрового уровня.
// Маркер типа для `ResourceKey<Level>` вынесен в `LevelKey`
// (см. выше); здесь — сам интерфейс уровня.
// ============================================================================

use std::sync::atomic::{AtomicI32, AtomicU32, Ordering};
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::resources::ResourceKey;
use crate::util::string_representable::StringRepresentable;

/// Мутабельное состояние `Level` (public-поля Java:
/// `randValue`, `oRainLevel`/`rainLevel`,
/// `oThunderLevel`/`thunderLevel`, `skyDarken`).
/// `AtomicU32` хранит `f32::to_bits`.
#[derive(Debug, Default)]
pub struct LevelState {
    /// `randValue` — сид для `getBlockRandomPos`.
    pub rand_value: AtomicI32,
    /// `oRainLevel`.
    o_rain_level: AtomicU32,
    /// `rainLevel`.
    rain_level: AtomicU32,
    /// `oThunderLevel`.
    o_thunder_level: AtomicU32,
    /// `thunderLevel`.
    thunder_level: AtomicU32,
    /// `skyDarken`.
    sky_darken: AtomicI32,
}

impl LevelState {
    /// Конструктор по умолчанию (все уровни погоды 0, randValue 0).
    pub fn new() -> Self {
        Self::default()
    }

    fn load_f32(a: &AtomicU32) -> f32 {
        f32::from_bits(a.load(Ordering::Relaxed))
    }
    fn store_f32(a: &AtomicU32, v: f32) {
        a.store(v.to_bits(), Ordering::Relaxed);
    }

    /// `oRainLevel`.
    pub fn o_rain_level(&self) -> f32 {
        Self::load_f32(&self.o_rain_level)
    }
    /// `rainLevel`.
    pub fn rain_level(&self) -> f32 {
        Self::load_f32(&self.rain_level)
    }
    /// `oThunderLevel`.
    pub fn o_thunder_level(&self) -> f32 {
        Self::load_f32(&self.o_thunder_level)
    }
    /// `thunderLevel`.
    pub fn thunder_level(&self) -> f32 {
        Self::load_f32(&self.thunder_level)
    }
    /// Установить обе пары (внутреннее использование `setRainLevel`/`setThunderLevel`).
    pub fn set_rain_levels(&self, o: f32, v: f32) {
        Self::store_f32(&self.o_rain_level, o);
        Self::store_f32(&self.rain_level, v);
    }
    /// Установить обе пары грома.
    pub fn set_thunder_levels(&self, o: f32, v: f32) {
        Self::store_f32(&self.o_thunder_level, o);
        Self::store_f32(&self.thunder_level, v);
    }
    /// `skyDarken`.
    pub fn sky_darken(&self) -> i32 {
        self.sky_darken.load(Ordering::Relaxed)
    }
    /// Установить `skyDarken`.
    pub fn set_sky_darken(&self, v: i32) {
        self.sky_darken.store(v, Ordering::Relaxed);
    }
}

/// `Level.ExplosionInteraction` — StringRepresentable.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ExplosionInteraction {
    /// `NONE`.
    None,
    /// `BLOCK`.
    Block,
    /// `MOB`.
    Mob,
    /// `TNT`.
    Tnt,
    /// `TRIGGER`.
    Trigger,
    /// `STANDARD` (CraftBukkit).
    Standard,
}

impl ExplosionInteraction {
    /// Все значения (порядок объявления Java).
    pub const VALUES: [ExplosionInteraction; 6] = [
        ExplosionInteraction::None,
        ExplosionInteraction::Block,
        ExplosionInteraction::Mob,
        ExplosionInteraction::Tnt,
        ExplosionInteraction::Trigger,
        ExplosionInteraction::Standard,
    ];

    /// `values()`.
    pub fn values() -> impl Iterator<Item = ExplosionInteraction> {
        Self::VALUES.iter().copied()
    }

    /// `byName`.
    pub fn by_name(name: &str) -> Option<ExplosionInteraction> {
        Self::values().find(|e| e.get_serialized_name() == name)
    }
}

impl crate::util::string_representable::StringRepresentable for ExplosionInteraction {
    fn get_serialized_name(&self) -> String {
        match self {
            ExplosionInteraction::None => "none".to_string(),
            ExplosionInteraction::Block => "block".to_string(),
            ExplosionInteraction::Mob => "mob".to_string(),
            ExplosionInteraction::Tnt => "tnt".to_string(),
            ExplosionInteraction::Trigger => "trigger".to_string(),
            ExplosionInteraction::Standard => "standard".to_string(),
        }
    }
}

/// `Level` — интерфейс игрового уровня.
///
/// Supertraits соответствуют Java-иерархии:
/// `LevelAccessor` (≈ `LevelAccessor`+`CommonLevelAccessor`),
/// `SignalGetter`, `CollisionGetter`, `BlockAndTintGetter`.
/// Методы, переопределённые в Java `Level` (`isRaining`,
/// `isThundering`, `canSeeSky`, `getDayTime`, `getGameTime`),
/// объявлены здесь повторно с реальной логикой — при
/// реализации нужно заполнить оба трейта (impl `LevelReader`
/// может делегировать в `Level`, как делает Java).
pub trait Level:
    level_accessor::LevelAccessor
    + signal_getter::SignalGetter
    + collision_getter::CollisionGetter
    + block_and_tint_getter::BlockAndTintGetter
    + Send
    + Sync
{
    // ------------------------------------------------------------------
    // Состояние (public-поля Java → LevelState)
    // ------------------------------------------------------------------

    /// Доступ к мутабельному состоянию уровня.
    fn level_state(&self) -> &LevelState;

    /// `tickRateManager()`.
    fn tick_rate_manager(&self) -> &crate::world::tick_rate_manager::TickRateManager;

    /// `neighborUpdater` — `NeighborUpdater` уровня.
    fn neighbor_updater(&self) -> &dyn redstone::neighbor_updater::NeighborUpdater;

    // ------------------------------------------------------------------
    // Измерение / реестры
    // ------------------------------------------------------------------

    /// `dimension()` — `ResourceKey<Level>`.
    fn dimension(&self) -> ResourceKey<LevelKey>;

    /// `dimensionTypeRegistration()` — holder типа измерения.
    fn dimension_type_registration(&self) -> &crate::core::holder::Holder<dimension::dimension_type::DimensionType>;

    /// `dimensionType()` — `dimensionTypeRegistration.value()`.
    fn dimension_type(&self) -> &dimension::dimension_type::DimensionType {
        self.dimension_type_registration()
            .value()
            .expect("dimension type holder unbound")
    }

    /// `registryAccess()`.
    fn registry_access(&self) -> &crate::core::registry::RegistryAccessMap;

    /// `damageSources()`.
    fn damage_sources(&self) -> &crate::world::damagesource::damage_sources::DamageSources;

    /// `getUncachedNoiseBiome(x, y, z)` — шумовой биом
    /// (quart-координаты).
    fn get_uncached_noise_biome(
        &self,
        x: i32,
        y: i32,
        z: i32,
    ) -> crate::core::holder::Holder<biome::biome::Biome>;

    // ------------------------------------------------------------------
    // Время / погода (LevelData + LevelState)
    // ------------------------------------------------------------------

    /// `getDayTime()` — `levelData.getDayTime()`.
    /// Переопределяет `LevelReader::day_time` (Java Level
    /// перекрывает метод `LevelReader`).
    fn day_time(&self) -> i64 {
        self.get_level_data().get_day_time()
    }

    /// `getGameTime()` — `levelData.getGameTime()`.
    /// Переопределяет `LevelReader::game_time`.
    fn game_time(&self) -> i64 {
        self.get_level_data().get_game_time()
    }

    /// `prepareWeather()` — вызывается при старте тика;
    /// при `levelData.isRaining()` выставляет уровни в 1.0.
    fn prepare_weather(&self) {
        if self.get_level_data().is_raining() {
            let s = self.level_state();
            s.set_rain_levels(1.0, 1.0);
            if self.get_level_data().is_thundering() {
                s.set_thunder_levels(1.0, 1.0);
            }
        }
    }

    /// `getThunderLevel(partialTick)` — lerp oThunder→thunder
    /// × getRainLevel.
    fn get_thunder_level(&self, partial_tick: f32) -> f32 {
        let s = self.level_state();
        crate::util::mth::lerp_f32(
            partial_tick,
            s.o_thunder_level(),
            s.thunder_level(),
        ) * self.get_rain_level(partial_tick)
    }

    /// `setThunderLevel(level)` — clamp [0,1], o=level.
    fn set_thunder_level(&self, level: f32) {
        let f = crate::util::mth::clamp_f32(level, 0.0, 1.0);
        self.level_state().set_thunder_levels(f, f);
    }

    /// `getRainLevel(partialTick)`.
    fn get_rain_level(&self, partial_tick: f32) -> f32 {
        let s = self.level_state();
        crate::util::mth::lerp_f32(partial_tick, s.o_rain_level(), s.rain_level())
    }

    /// `setRainLevel(level)`.
    fn set_rain_level(&self, level: f32) {
        let f = crate::util::mth::clamp_f32(level, 0.0, 1.0);
        self.level_state().set_rain_levels(f, f);
    }

    /// `canHaveWeather()` — sky light, нет ceiling, не END.
    fn can_have_weather(&self) -> bool {
        self.dimension_type().has_sky_light
            && !self.dimension_type().has_ceiling
            && self.dimension() != LevelKey::end()
    }

    /// `isThundering()` — переопределяет `LevelReader::is_thundering`.
    fn is_thundering(&self) -> bool {
        self.can_have_weather() && self.get_thunder_level(1.0) > 0.9
    }

    /// `isRaining()` — переопределяет `LevelReader::is_raining`.
    fn is_raining(&self) -> bool {
        self.can_have_weather() && self.get_rain_level(1.0) > 0.2
    }

    /// `isRainingAt(pos)`.
    fn is_raining_at(&self, pos: BlockPos) -> bool {
        self.precipitation_at(pos) == biome::biome::Precipitation::Rain
    }

    /// `precipitationAt(pos)` — NONE если не дождь / нет неба /
    /// выше heightmap; иначе `biome.getPrecipitationAt`.
    fn precipitation_at(&self, pos: BlockPos) -> biome::biome::Precipitation {
        use biome::biome::Precipitation;
        if !Level::is_raining(self) {
            return Precipitation::None;
        }
        if !Level::can_see_sky(self, pos) {
            return Precipitation::None;
        }
        if level_reader::LevelReader::get_heightmap_pos(
                self,
                levelgen::heightmap::HeightmapType::MotionBlocking,
                pos,
            )
            .get_y()
            > pos.get_y()
        {
            return Precipitation::None;
        }
        match self.biome_at(pos).as_ref().and_then(|holder| holder.value()) {
            Some(b) => b.get_precipitation_at(pos, self.sea_level()),
            None => Precipitation::None,
        }
    }

    // ------------------------------------------------------------------
    // Небо / освещение
    // ------------------------------------------------------------------

    /// `canSeeSky(pos)` — проверяет уровень небесного света.
    fn can_see_sky(&self, pos: BlockPos) -> bool {
        block_and_tint_getter::BlockAndTintGetter::can_see_sky(self, pos)
    }

    /// `getSkyDarken()`.
    fn get_sky_darken(&self) -> i32 {
        self.level_state().sky_darken()
    }

    /// `setSkyFlashTime(time)` — no-op на сервере (Java пустой).
    fn set_sky_flash_time(&self, _time: i32) {}

    // ------------------------------------------------------------------
    // Блоки / события
    // ------------------------------------------------------------------

    /// `isStateAtPosition(pos, predicate)`.
    fn is_state_at_position(
        &self,
        pos: BlockPos,
        predicate: &dyn Fn(&block::state::block_state::BlockState) -> bool,
    ) -> bool {
        let state = level_reader::LevelReader::get_block_state(self, pos)
            .unwrap_or_else(block::state::block_state::BlockState::air);
        predicate(&state)
    }

    /// `isFluidAtPosition(pos, predicate)`.
    fn is_fluid_at_position(
        &self,
        pos: BlockPos,
        predicate: &dyn Fn(&block::state::fluid_state::FluidState) -> bool,
    ) -> bool {
        match level_reader::LevelReader::get_fluid_state(self, pos) {
            Some(f) => predicate(&f),
            None => predicate(&block::state::fluid_state::FluidState::empty()),
        }
    }

    /// `blockEvent(pos, block, eventId, eventParam)` —
    /// делегирует событие блочной сущности.
    fn block_event(&self, pos: BlockPos, _block: &block::Block, event_id: i32, event_param: i32) {
        if let Some(entity) = level_reader::LevelReader::get_block_entity(self, pos) {
            entity.trigger_event(event_id, event_param);
        }
    }

    /// `updateNeighbourForOutputSignal(pos, block)` — компараторы
    /// по горизонтали (Java: 4 направления, проверка
    /// `isRedstoneConductor` для пропуска).
    fn update_neighbour_for_output_signal(&self, pos: BlockPos, block: &block::Block) {
        for direction in crate::core::direction::Plane::FACES_HORIZONTAL {
            let mut block_pos = pos.relative(direction);
            if self.has_chunk_at(block_pos) {
                if let Some(state) = level_reader::LevelReader::get_block_state(self, block_pos) {
                    if state.is_block_id("minecraft:comparator") {
                        self.neighbor_updater().neighbor_changed_state(
                            state,
                            block_pos,
                            &neighbor_block_state(block),
                            None,
                            false,
                        );
                    } else if state.is_redstone_conductor(self.as_level_accessor(), block_pos) {
                        block_pos = block_pos.relative(direction);
                        if let Some(state2) = level_reader::LevelReader::get_block_state(self, block_pos) {
                            if state2.is_block_id("minecraft:comparator") {
                                self.neighbor_updater().neighbor_changed_state(
                                    state2,
                                    block_pos,
                                    &neighbor_block_state(block),
                                    None,
                                    false,
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// `neighborChanged(state, pos, neighborBlock, orientation,
    /// movedByPiston)` — делегирует в `NeighborUpdater`.
    fn neighbor_changed_state(
        &self,
        state: block::state::block_state::BlockState,
        pos: BlockPos,
        neighbor_block: &block::state::block_state::BlockState,
        orientation: Option<&redstone::orientation::Orientation>,
        moved_by_piston: bool,
    ) {
        self.neighbor_updater().neighbor_changed_state(
            state,
            pos,
            neighbor_block,
            orientation,
            moved_by_piston,
        );
    }

    /// `mayInteract(entity, pos)` — default `true` (Java).
    fn may_interact(&self, _entity: &crate::world::entity::entity::Entity, _pos: BlockPos) -> bool {
        true
    }

    /// `broadcastDamageEvent(entity, damageSource)` — no-op (Java).
    fn broadcast_damage_event(
        &self,
        _entity: &crate::world::entity::entity::Entity,
        _damage_source: &crate::world::damagesource::damage_source::DamageSource,
    ) {
    }

    /// `sendPacketToServer(packet)` — на сервере бросает
    /// `UnsupportedOperationException` (Java).
    fn send_packet_to_server(&self, _packet: &dyn std::any::Any) {
        panic!("Can't send packets to server unless you're on the client.");
    }

    // ------------------------------------------------------------------
    // Случайные позиции / респавн
    // ------------------------------------------------------------------

    /// `getBlockRandomPos(x, y, z, yMask)` — детерминированный
    /// рандомный оффсет через `randValue` LCG.
    fn get_block_random_pos(&self, x: i32, y: i32, z: i32, y_mask: i32) -> BlockPos {
        let s = self.level_state();
        let next = |seed: i32| seed.wrapping_mul(3).wrapping_add(1013904223);
        let previous = s.rand_value.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |seed| Some(next(seed)))
            .expect("the seed update always returns a value");
        let i = next(previous) >> 2;
        BlockPos::new(x + (i & 15), y + ((i >> 16) & y_mask), z + ((i >> 8) & 15))
    }

    /// `getWorldBorderAdjustedRespawnData(respawnData)` —
    /// если точка вне border → heightmap-позиция над центром.
    fn get_world_border_adjusted_respawn_data(&self, respawn_data: &RespawnData) -> RespawnData {
        let border = self.get_world_border();
        let pos = respawn_data.pos();
        if !border.is_within_bounds_pos(&pos) {
            let heightmap_pos = level_reader::LevelReader::get_heightmap_pos(
                self,
                levelgen::heightmap::HeightmapType::MotionBlocking,
                BlockPos::containing(border.get_center_x(), 0.0, border.get_center_z()),
            );
            return RespawnData::of(
                respawn_data.dimension().clone(),
                heightmap_pos,
                respawn_data.yaw,
                respawn_data.pitch,
            );
        }
        respawn_data.clone()
    }

    /// `noSave()` — default `false`.
    fn no_save(&self) -> bool {
        false
    }

    /// `isDebug()` — debug-режим мира.
    fn is_debug(&self) -> bool;
}

/// Вспомогательная функция: `BlockState` для `Block`
/// (`block.defaultBlockState()` — используется
/// `updateNeighbourForOutputSignal`).
fn neighbor_block_state(block: &block::Block) -> block::state::block_state::BlockState {
    block.default_block_state().unwrap_or_else(|| {
        block::state::block_state::BlockState::without_properties(Arc::new(block.clone()))
    })
}
