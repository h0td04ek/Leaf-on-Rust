//! Перенос `net.minecraft.world.level` — блоки, уровни, чанки.

pub mod block;
pub mod game_type;
pub mod chunk_pos;
pub mod saveddata;
pub mod level_reader;
pub mod clip;

/// `net.minecraft.world.level.Level` — маркер типа для `ResourceKey<Level>`
/// и GlobalPos. Полная реализация Level — отдельный этап порта.
#[derive(Debug)]
pub struct Level;

impl Level {
    /// `RESOURCE_KEY_CODEC` — `ResourceKey.codec(Registries.DIMENSION)`.
    /// (Кодек регистрируется в network-слое.)
    /// `OVERWORLD`.
    pub fn overworld() -> crate::resources::ResourceKey<Level> {
        crate::resources::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            crate::resources::identifier::Identifier::with_default_namespace("overworld").unwrap(),
        )
    }

    /// `NETHER`.
    pub fn nether() -> crate::resources::ResourceKey<Level> {
        crate::resources::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::dimension(),
            crate::resources::identifier::Identifier::with_default_namespace("the_nether").unwrap(),
        )
    }

    /// `END`.
    pub fn end() -> crate::resources::ResourceKey<Level> {
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
            crate::core::global_pos::GlobalPos::of(Level::overworld(), crate::core::block_pos::BlockPos::ZERO),
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
        dimension: crate::resources::ResourceKey<crate::world::level::Level>,
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
    pub fn dimension(&self) -> &crate::resources::ResourceKey<Level> {
        &self.global_pos.dimension
    }

    /// `pos()`.
    pub fn pos(&self) -> crate::core::block_pos::BlockPos {
        self.global_pos.pos
    }

    /// `withLevel(dimension)` (Paper).
    pub fn with_level(&self, dimension: crate::resources::ResourceKey<Level>) -> Self {
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


