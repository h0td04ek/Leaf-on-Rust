//! Перенос `net.minecraft.world.level.dimension.DimensionType` —
//! record типа измерения (fixed time, skylight, ceiling,
//! coordinate scale, высоты, infiniburn, ambient light,
//! monster settings, skybox, cardinal light, attributes,
//! timelines).
//!
//! Кодеки `DIRECT_CODEC`/`NETWORK_CODEC`/`CODEC` — как и в
//! остальных модулях, ждут порт Mojang Codec; здесь полные
//! данные и вся не-codec логика.

use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::tags::tag_key::TagKey;
use crate::util::valueproviders::int_provider::IntProvider;
use crate::util::string_representable::StringRepresentable;
use crate::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
use crate::world::level::block::Block;
use crate::world::timeline::Timeline;
use std::sync::Arc;

/// `DimensionType` — record.
#[derive(Clone, Debug)]
pub struct DimensionType {
    /// `hasFixedTime`.
    pub has_fixed_time: bool,
    /// `hasSkyLight`.
    pub has_sky_light: bool,
    /// `hasCeiling`.
    pub has_ceiling: bool,
    /// `coordinateScale`.
    pub coordinate_scale: f64,
    /// `minY`.
    pub min_y: i32,
    /// `height`.
    pub height: i32,
    /// `logicalHeight`.
    pub logical_height: i32,
    /// `infiniburn`.
    pub infiniburn: TagKey<Block>,
    /// `ambientLight`.
    pub ambient_light: f32,
    /// `monsterSettings`.
    pub monster_settings: MonsterSettings,
    /// `skybox`.
    pub skybox: Skybox,
    /// `cardinalLightType`.
    pub cardinal_light_type: CardinalLightType,
    /// `attributes`.
    pub attributes: EnvironmentAttributeMap,
    /// `timelines`.
    pub timelines: HolderSet<Timeline>,
}

impl DimensionType {
    /// `BITS_FOR_Y` — `BlockPos.PACKED_Y_LENGTH`.
    pub const BITS_FOR_Y: u32 = 12;
    /// `MIN_HEIGHT`.
    pub const MIN_HEIGHT: i32 = 16;
    /// `Y_SIZE`.
    pub const Y_SIZE: i32 = (1 << Self::BITS_FOR_Y) - 32;
    /// `MAX_Y`.
    pub const MAX_Y: i32 = (Self::Y_SIZE >> 1) - 1;
    /// `MIN_Y`.
    pub const MIN_Y: i32 = Self::MAX_Y - Self::Y_SIZE + 1;
    /// `WAY_ABOVE_MAX_Y`.
    pub const WAY_ABOVE_MAX_Y: i32 = Self::MAX_Y << 4;
    /// `WAY_BELOW_MIN_Y`.
    pub const WAY_BELOW_MIN_Y: i32 = Self::MIN_Y << 4;

    /// `MOON_BRIGHTNESS_PER_PHASE`.
    pub const MOON_BRIGHTNESS_PER_PHASE: [f32; 8] = [1.0, 0.75, 0.5, 0.25, 0.0, 0.25, 0.5, 0.75];

    /// Компактный конструктор — все поля record'а.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        has_fixed_time: bool,
        has_sky_light: bool,
        has_ceiling: bool,
        coordinate_scale: f64,
        min_y: i32,
        height: i32,
        logical_height: i32,
        infiniburn: TagKey<Block>,
        ambient_light: f32,
        monster_settings: MonsterSettings,
        skybox: Skybox,
        cardinal_light_type: CardinalLightType,
        attributes: EnvironmentAttributeMap,
        timelines: HolderSet<Timeline>,
    ) -> Self {
        // Валидация компактного конструктора Java.
        assert!(height >= Self::MIN_HEIGHT, "height has to be at least 16");
        assert!(
            min_y + height <= Self::MAX_Y + 1,
            "min_y + height cannot be higher than: {}",
            Self::MAX_Y + 1
        );
        assert!(logical_height <= height, "logical_height cannot be higher than height");
        assert!(height % 16 == 0, "height has to be multiple of 16");
        assert!(min_y % 16 == 0, "min_y has to be a multiple of 16");
        Self {
            has_fixed_time,
            has_sky_light,
            has_ceiling,
            coordinate_scale,
            min_y,
            height,
            logical_height,
            infiniburn,
            ambient_light,
            monster_settings,
            skybox,
            cardinal_light_type,
            attributes,
            timelines,
        }
    }

    /// `getTeleportationScale(firstType, secondType)`.
    pub fn get_teleportation_scale(first_type: &DimensionType, second_type: &DimensionType) -> f64 {
        first_type.coordinate_scale / second_type.coordinate_scale
    }

    /// `monsterSpawnLightTest()`.
    pub fn monster_spawn_light_test(&self) -> &dyn IntProvider {
        self.monster_settings.monster_spawn_light_test.as_ref()
    }

    /// `monsterSpawnBlockLightLimit()`.
    pub fn monster_spawn_block_light_limit(&self) -> i32 {
        self.monster_settings.monster_spawn_block_light_limit
    }

    /// `hasEndFlashes()`.
    pub fn has_end_flashes(&self) -> bool {
        self.skybox == Skybox::End
    }

    /// `logicalHeight()` — алиас для читабельности.
    pub fn logical_height(&self) -> i32 {
        self.logical_height
    }
}

/// `DimensionType.MonsterSettings` — record.
#[derive(Clone)]
pub struct MonsterSettings {
    /// `monsterSpawnLightTest`.
    pub monster_spawn_light_test: Arc<dyn IntProvider + Send + Sync>,
    /// `monsterSpawnBlockLightLimit`.
    pub monster_spawn_block_light_limit: i32,
}

impl std::fmt::Debug for MonsterSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MonsterSettings")
            .field("monster_spawn_light_test", &self.monster_spawn_light_test.describe())
            .field("monster_spawn_block_light_limit", &self.monster_spawn_block_light_limit)
            .finish()
    }
}

impl MonsterSettings {
    /// Конструктор.
    pub fn new(
        monster_spawn_light_test: Arc<dyn IntProvider + Send + Sync>,
        monster_spawn_block_light_limit: i32,
    ) -> Self {
        Self { monster_spawn_light_test, monster_spawn_block_light_limit }
    }
}

/// `DimensionType.Skybox` — enum `StringRepresentable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Skybox {
    /// `NONE`.
    None,
    /// `OVERWORLD`.
    Overworld,
    /// `END`.
    End,
}

impl Skybox {
    /// Все значения (порядок объявления Java).
    pub const VALUES: [Skybox; 3] = [Skybox::None, Skybox::Overworld, Skybox::End];

    /// `values()`-совместимый итератор.
    pub fn values() -> impl Iterator<Item = Skybox> {
        Self::VALUES.iter().copied()
    }

    /// `byName`-lookup.
    pub fn by_name(name: &str) -> Option<Skybox> {
        Self::values().find(|s| s.get_serialized_name() == name)
    }
}

impl StringRepresentable for Skybox {
    fn get_serialized_name(&self) -> String {
        match self {
            Skybox::None => "none".to_string(),
            Skybox::Overworld => "overworld".to_string(),
            Skybox::End => "end".to_string(),
        }
    }
}

/// `DimensionType.CardinalLightType` — enum `StringRepresentable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CardinalLightType {
    /// `DEFAULT`.
    Default,
    /// `NETHER`.
    Nether,
}

impl CardinalLightType {
    /// Все значения.
    pub const VALUES: [CardinalLightType; 2] = [CardinalLightType::Default, CardinalLightType::Nether];

    /// `values()`.
    pub fn values() -> impl Iterator<Item = CardinalLightType> {
        Self::VALUES.iter().copied()
    }

    /// `byName`.
    pub fn by_name(name: &str) -> Option<CardinalLightType> {
        Self::values().find(|c| c.get_serialized_name() == name)
    }
}

impl StringRepresentable for CardinalLightType {
    fn get_serialized_name(&self) -> String {
        match self {
            CardinalLightType::Default => "default".to_string(),
            CardinalLightType::Nether => "nether".to_string(),
        }
    }
}
