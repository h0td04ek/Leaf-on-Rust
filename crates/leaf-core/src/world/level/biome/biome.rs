//! `net.minecraft.world.level.biome.Biome` — биом: климат, генерация,
//! mob spawn, special effects, цвета, осадки, замерзание/снег.

use super::biome_generation_settings::BiomeGenerationSettings;
use super::biome_special_effects::BiomeSpecialEffects;
use super::mob_spawn_settings::MobSpawnSettings;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::util::mth;
use crate::util::string_representable::StringRepresentable;
use crate::world::attribute::environment_attribute_map::EnvironmentAttributeMap;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::synth::improved_noise::ImprovedNoise;
use crate::world::level::levelgen::synth::perlin_simplex_noise::PerlinSimplexNoise;
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;
use crate::random::legacy::LegacyRandomSource;
use crate::random::RandomSource;
use std::sync::{Arc, OnceLock};

const TEMPERATURE_CACHE_SIZE: usize = 1024;

fn legacy_random(seed: i64) -> Box<dyn RandomSource> {
    Box::new(LegacyRandomSource::new(seed))
}

fn temperature_noise() -> &'static PerlinSimplexNoise {
    static N: OnceLock<PerlinSimplexNoise> = OnceLock::new();
    N.get_or_init(|| {
        let mut r = WorldgenRandom::new(legacy_random(1234));
        PerlinSimplexNoise::new(&mut r, [0])
    })
}

fn frozen_temperature_noise() -> &'static PerlinSimplexNoise {
    static N: OnceLock<PerlinSimplexNoise> = OnceLock::new();
    N.get_or_init(|| {
        let mut r = WorldgenRandom::new(legacy_random(3456));
        PerlinSimplexNoise::new(&mut r, [-2, -1, 0])
    })
}

/// `Biome.BIOME_INFO_NOISE` (deprecated-for-removal, используется special effects).
pub fn biome_info_noise() -> &'static PerlinSimplexNoise {
    static N: OnceLock<PerlinSimplexNoise> = OnceLock::new();
    N.get_or_init(|| {
        let mut r = WorldgenRandom::new(legacy_random(2345));
        PerlinSimplexNoise::new(&mut r, [0])
    })
}

/// `Biome.TEMPERATURE_NOISE`.
pub fn temperature_noise_pub() -> &'static PerlinSimplexNoise {
    temperature_noise()
}

/// `Biome` — финальный класс биома.
#[derive(Clone)]
pub struct Biome {
    /// Уникальный id экземпляра (Java: identity equals).
    pub uid: usize,
    /// `climateSettings`.
    pub climate_settings: ClimateSettings,
    generation_settings: BiomeGenerationSettings,
    mob_settings: MobSpawnSettings,
    attributes: EnvironmentAttributeMap,
    special_effects: BiomeSpecialEffects,
    /// `temperatureCache` — LRU-кэш температуры (1024 записи).
    temperature_cache: Arc<std::sync::Mutex< TemperatureCache >>,
}

/// LRU-кэш температуры (`Long2FloatLinkedOpenHashMap` с rehash-noop).
#[derive(Default)]
pub struct TemperatureCache {
    map: std::collections::HashMap<i64, f32>,
    order: std::collections::VecDeque<i64>,
}

impl TemperatureCache {
    fn get(&mut self, key: i64) -> Option<f32> {
        if let Some(&v) = self.map.get(&key) {
            if v.is_nan() {
                return None;
            }
            // LRU: перенос в конец
            if let Some(pos) = self.order.iter().position(|&k| k == key) {
                let k = self.order.remove(pos).unwrap();
                self.order.push_back(k);
            }
            return Some(v);
        }
        None
    }

    fn put(&mut self, key: i64, value: f32) {
        if self.map.len() >= TEMPERATURE_CACHE_SIZE && !self.map.contains_key(&key) {
            if let Some(old) = self.order.pop_front() {
                self.map.remove(&old);
            }
        }
        self.map.insert(key, value);
        if let Some(pos) = self.order.iter().position(|&k| k == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key);
    }
}

impl Biome {
    /// `new Biome(...)`.
    pub fn new(
        climate_settings: ClimateSettings,
        attributes: EnvironmentAttributeMap,
        special_effects: BiomeSpecialEffects,
        generation_settings: BiomeGenerationSettings,
        mob_settings: MobSpawnSettings,
    ) -> Self {
        Self {
            uid: next_biome_uid(),
            climate_settings,
            generation_settings,
            mob_settings,
            attributes,
            special_effects,
            temperature_cache: Arc::new(std::sync::Mutex::new(TemperatureCache::default())),
        }
    }

    /// `getMobSettings`.
    pub fn get_mob_settings(&self) -> &MobSpawnSettings {
        &self.mob_settings
    }

    /// `hasPrecipitation`.
    pub fn has_precipitation(&self) -> bool {
        self.climate_settings.has_precipitation
    }

    /// `getPrecipitationAt(pos, seaLevel)`.
    pub fn get_precipitation_at(&self, pos: BlockPos, sea_level: i32) -> Precipitation {
        if !self.has_precipitation() {
            return Precipitation::None;
        }
        if self.cold_enough_to_snow(pos, sea_level) {
            Precipitation::Snow
        } else {
            Precipitation::Rain
        }
    }

    /// `getHeightAdjustedTemperature` (private в Java).
    fn get_height_adjusted_temperature(&self, pos: BlockPos, sea_level: i32) -> f32 {
        let f = self.climate_settings.temperature_modifier.modify_temperature(pos, self.get_base_temperature());
        let i = sea_level + 17;
        if pos.y > i {
            let f1 = (temperature_noise().get_value(pos.x as f64 / 8.0, pos.z as f64 / 8.0, false) * 8.0) as f32;
            f - (f1 + pos.y as f32 - i as f32) * 0.05 / 40.0
        } else {
            f
        }
    }

    /// `getTemperature(pos, seaLevel)` — с LRU-кэшем.
    pub fn get_temperature(&self, pos: BlockPos, sea_level: i32) -> f32 {
        let key = pos.as_long();
        let mut cache = self.temperature_cache.lock().unwrap();
        if let Some(v) = cache.get(key) {
            return v;
        }
        let value = self.get_height_adjusted_temperature(pos, sea_level);
        cache.put(key, value);
        value
    }

    /// `shouldFreeze(level, pos)`.
    pub fn should_freeze(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        self.should_freeze_must_be_at_edge(level, pos, true)
    }

    /// `shouldFreeze(level, water, mustBeAtEdge)`.
    pub fn should_freeze_must_be_at_edge(&self, level: &dyn LevelReader, water: BlockPos, must_be_at_edge: bool) -> bool {
        if self.warm_enough_to_rain(water, level.sea_level()) {
            return false;
        }
        if level.is_inside_build_height(water.y) && level.get_brightness(crate::world::level::light_layer::LightLayer::Block, water) < 10 {
            let block_state = level.get_block_state(water);
            let fluid_state = level.get_fluid_state(water);
            let is_water = fluid_state.as_ref().map(|f| f.is_water()).unwrap_or(false);
            if is_water {
                if let Some(bs) = block_state {
                    // `blockState.getBlock() instanceof LiquidBlock` —
                    // проверяем через fluid-water + liquid block флаг.
                    if bs.is_liquid_block() {
                        if !must_be_at_edge {
                            return true;
                        }
                        let flag = level.is_water_at(water.west())
                            && level.is_water_at(water.east())
                            && level.is_water_at(water.north())
                            && level.is_water_at(water.south());
                        if !flag {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// `coldEnoughToSnow`.
    pub fn cold_enough_to_snow(&self, pos: BlockPos, sea_level: i32) -> bool {
        !self.warm_enough_to_rain(pos, sea_level)
    }

    /// `warmEnoughToRain`.
    pub fn warm_enough_to_rain(&self, pos: BlockPos, sea_level: i32) -> bool {
        self.get_temperature(pos, sea_level) >= 0.15
    }

    /// `shouldMeltFrozenOceanIcebergSlightly`.
    pub fn should_melt_frozen_ocean_iceberg_slightly(&self, pos: BlockPos, sea_level: i32) -> bool {
        self.get_temperature(pos, sea_level) > 0.1
    }

    /// `shouldSnow(level, pos)`.
    pub fn should_snow(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        if self.get_precipitation_at(pos, level.sea_level()) != Precipitation::Snow {
            return false;
        }
        if level.is_inside_build_height(pos.y) && level.get_brightness(crate::world::level::light_layer::LightLayer::Block, pos) < 10 {
            if let Some(block_state) = level.get_block_state(pos) {
                let is_snow = block_state.is_air()
                    || crate::world::level::block::block_by_identifier(&crate::resources::identifier::Identifier::minecraft("snow"))
                        .map(|b| block_state.is_block(&b))
                        .unwrap_or(false);
                if is_snow {
                    // `Blocks.SNOW.defaultBlockState().canSurvive(level, pos)` —
                    // canSurvive требует LevelReader; делегируем состоянию.
                    if let Some(snow_state) = snow_block_state() {
                        if snow_state.can_survive(level, pos) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    /// `getGenerationSettings`.
    pub fn get_generation_settings(&self) -> &BiomeGenerationSettings {
        &self.generation_settings
    }

    /// `getGrassColor(x, z)`.
    pub fn get_grass_color(&self, pos_x: f64, pos_z: f64) -> i32 {
        let base = self.get_base_grass_color();
        self.special_effects.grass_color_modifier.modify_color(pos_x, pos_z, base)
    }

    /// `getBaseGrassColor` (private).
    fn get_base_grass_color(&self) -> i32 {
        self.special_effects.grass_color_override.unwrap_or_else(|| self.get_grass_color_from_texture())
    }

    /// `getGrassColorFromTexture` (private).
    fn get_grass_color_from_texture(&self) -> i32 {
        let d = mth::clamp_f64(self.climate_settings.temperature as f64, 0.0, 1.0);
        let d1 = mth::clamp_f64(self.climate_settings.downfall as f64, 0.0, 1.0);
        crate::world::level::grass_color::get(d, d1)
    }

    /// `getFoliageColor`.
    pub fn get_foliage_color(&self) -> i32 {
        self.special_effects.foliage_color_override.unwrap_or_else(|| self.get_foliage_color_from_texture())
    }

    fn get_foliage_color_from_texture(&self) -> i32 {
        let d = mth::clamp_f64(self.climate_settings.temperature as f64, 0.0, 1.0);
        let d1 = mth::clamp_f64(self.climate_settings.downfall as f64, 0.0, 1.0);
        crate::world::level::foliage_color::get(d, d1)
    }

    /// `getDryFoliageColor`.
    pub fn get_dry_foliage_color(&self) -> i32 {
        self.special_effects.dry_foliage_color_override.unwrap_or_else(|| self.get_dry_foliage_color_from_texture())
    }

    fn get_dry_foliage_color_from_texture(&self) -> i32 {
        let d = mth::clamp_f64(self.climate_settings.temperature as f64, 0.0, 1.0);
        let d1 = mth::clamp_f64(self.climate_settings.downfall as f64, 0.0, 1.0);
        crate::world::level::dry_foliage_color::get(d, d1)
    }

    /// `getBaseTemperature`.
    pub fn get_base_temperature(&self) -> f32 {
        self.climate_settings.temperature
    }

    /// `getAttributes`.
    pub fn get_attributes(&self) -> &EnvironmentAttributeMap {
        &self.attributes
    }

    /// `getSpecialEffects`.
    pub fn get_special_effects(&self) -> &BiomeSpecialEffects {
        &self.special_effects
    }

    /// `getWaterColor`.
    pub fn get_water_color(&self) -> i32 {
        self.special_effects.water_color
    }
}

fn snow_block_state() -> Option<BlockState> {
    crate::world::level::block::block_by_identifier(&crate::resources::identifier::Identifier::minecraft("snow"))
        .and_then(|b| b.default_block_state())
}

impl Default for TemperatureModifier {
    fn default() -> Self {
        TemperatureModifier::None
    }
}

/// `Biome.ClimateSettings(hasPrecipitation, temperature, temperatureModifier, downfall)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClimateSettings {
    /// `hasPrecipitation`.
    pub has_precipitation: bool,
    /// `temperature`.
    pub temperature: f32,
    /// `temperatureModifier`.
    pub temperature_modifier: TemperatureModifier,
    /// `downfall`.
    pub downfall: f32,
}

/// `Biome.Precipitation`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Precipitation {
    /// `NONE`.
    None,
    /// `RAIN`.
    Rain,
    /// `SNOW`.
    Snow,
}

impl StringRepresentable for Precipitation {
    fn get_serialized_name(&self) -> String {
        match self {
            Precipitation::None => "none".into(),
            Precipitation::Rain => "rain".into(),
            Precipitation::Snow => "snow".into(),
        }
    }
}

/// `Biome.TemperatureModifier`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemperatureModifier {
    /// `NONE`.
    None,
    /// `FROZEN`.
    Frozen,
}

impl TemperatureModifier {
    /// `modifyTemperature(pos, temperature)`.
    pub fn modify_temperature(&self, pos: BlockPos, temperature: f32) -> f32 {
        match self {
            TemperatureModifier::None => temperature,
            TemperatureModifier::Frozen => {
                let d = (frozen_temperature_noise().get_value(pos.x as f64 * 0.05, pos.z as f64 * 0.05, false) * 7.0) as f32;
                let value = biome_info_noise().get_value(pos.x as f64 * 0.2, pos.z as f64 * 0.2, false) as f32;
                let d1 = d + value;
                if d1 < 0.3 {
                    let value1 = biome_info_noise().get_value(pos.x as f64 * 0.09, pos.z as f64 * 0.09, false) as f32;
                    if value1 < 0.8 {
                        return 0.2;
                    }
                }
                temperature
            }
        }
    }
}

impl StringRepresentable for TemperatureModifier {
    fn get_serialized_name(&self) -> String {
        match self {
            TemperatureModifier::None => "none".into(),
            TemperatureModifier::Frozen => "frozen".into(),
        }
    }
}

/// `Biome.BiomeBuilder`.
#[derive(Default)]
pub struct BiomeBuilder {
    has_precipitation: bool,
    temperature: f32,
    downfall: f32,
    temperature_modifier: TemperatureModifier,
    special_effects: Option<BiomeSpecialEffects>,
    mob_spawn_settings: Option<MobSpawnSettings>,
    generation_settings: Option<BiomeGenerationSettings>,
    attributes: EnvironmentAttributeMap,
}

impl BiomeBuilder
            {
    /// `new BiomeBuilder()`.
    pub fn new() -> Self {
        Self { temperature_modifier: TemperatureModifier::None, ..Default::default() }
    }
    /// `hasPrecipitation`.
    pub fn has_precipitation(mut self, v: bool) -> Self {
        self.has_precipitation = v;
        self
    }
    /// `temperature`.
    pub fn temperature(mut self, v: f32) -> Self {
        self.temperature = v;
        self
    }
    /// `downfall`.
    pub fn downfall(mut self, v: f32) -> Self {
        self.downfall = v;
        self
    }
    /// `putAttributes(map)`.
    pub fn put_attributes(mut self, attributes: EnvironmentAttributeMap) -> Self {
        self.attributes = attributes;
        self
    }
    /// `specialEffects`.
    pub fn special_effects(mut self, effects: BiomeSpecialEffects) -> Self {
        self.special_effects = Some(effects);
        self
    }
    /// `mobSpawnSettings`.
    pub fn mob_spawn_settings(mut self, settings: MobSpawnSettings) -> Self {
        self.mob_spawn_settings = Some(settings);
        self
    }
    /// `generationSettings`.
    pub fn generation_settings(mut self, settings: BiomeGenerationSettings) -> Self {
        self.generation_settings = Some(settings);
        self
    }
    /// `temperatureAdjustment`.
    pub fn temperature_adjustment(mut self, modifier: TemperatureModifier) -> Self {
        self.temperature_modifier = modifier;
        self
    }
    /// `build`.
    pub fn build(self) -> Biome {
        Biome::new(
            ClimateSettings {
                has_precipitation: self.has_precipitation,
                temperature: self.temperature,
                temperature_modifier: self.temperature_modifier,
                downfall: self.downfall,
            },
            self.attributes,
            self.special_effects.unwrap_or_else(|| {
                BiomeSpecialEffectsBuilder::default().water_color(0).build()
            }),
            self.generation_settings.unwrap_or_default(),
            self.mob_spawn_settings.unwrap_or_else(super::mob_spawn_settings::empty),
        )
    }
}

use super::biome_special_effects::BiomeSpecialEffectsBuilder;

static BIOME_UID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
fn next_biome_uid() -> usize {
    BIOME_UID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}
impl PartialEq for Biome {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}
impl Eq for Biome {}
impl std::hash::Hash for Biome {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.uid.hash(state);
    }
}
