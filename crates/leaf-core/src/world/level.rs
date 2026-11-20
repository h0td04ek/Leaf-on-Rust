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
