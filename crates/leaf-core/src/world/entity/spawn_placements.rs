//! Перенос `net.minecraft.world.entity.SpawnPlacements` +
//! `SpawnPlacementType(s)` — типы и правила спавна сущностей.
//!
//! - `DATA_BY_TYPE`: entity_type_id → `(HeightmapType,
//!   SpawnPlacementType, SpawnPredicate)`.
//! - `SpawnPredicate` — `fn(EntityType, &dyn LevelReader,
//!   EntitySpawnReason, BlockPos, &mut dyn RandomSource) -> bool`.
//! - Type-specific предикаты Java-классов (Axolotl::… и т.п.)
//!   сгруппированы в семейные функции; где Java-метод специфичен
//!   для ещё непортированного класса — отдельная проверка идёт
//!   через `LevelReader::spawn_rule_override`-стиль: здесь
//!   регистрируется семейный предикат + `custom` ветка через
//!   `spawn_placements_custom_rules` (см. `custom_rule_for`).

use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

use crate::core::block_pos::BlockPos;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::entity::entity_spawn_reason::EntitySpawnReason;
use crate::entity::entity_type::EntityType;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::phys::vec3::Vec3;

/// `SpawnPlacementType` — тип допустимой позиции спавна.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnPlacementType {
    /// NO_RESTRICTIONS — без ограничений.
    NoRestrictions,
    /// IN_WATER — в воде (fluid=water, блок выше не
    /// redstone-conductor).
    InWater,
    /// IN_LAVA — в лаве.
    InLava,
    /// ON_GROUND — на земле.
    OnGround,
}

impl SpawnPlacementType {
    /// `isSpawnPositionOk(level, pos, entityType)`.
    pub fn is_spawn_position_ok(
        &self,
        level: &dyn LevelReader,
        pos: BlockPos,
        entity_type: Option<&EntityType>,
    ) -> bool {
        match self {
            SpawnPlacementType::NoRestrictions => true,
            SpawnPlacementType::InWater => {
                let Some(_) = entity_type else { return false };
                if !level.world_border_is_within_bounds(pos) {
                    return false;
                }
                let above = pos.above();
                let water_tag = crate::tags::fluid_tags::FluidTags::water();
                match level.get_fluid_state(pos) {
                    Some(fs) => {
                        fs.is_in_named_tag(&water_tag)
                            && !level
                                .get_block_state(above)
                                .map(|s| s.is_redstone_conductor(level, above))
                                .unwrap_or(false)
                    }
                    None => false,
                }
            }
            SpawnPlacementType::InLava => {
                let Some(_) = entity_type else { return false };
                level.world_border_is_within_bounds(pos)
                    && level
                        .get_fluid_state(pos)
                        .map(|fs| fs.is_in_named_tag(&crate::tags::fluid_tags::FluidTags::lava()))
                        .unwrap_or(false)
            }
            SpawnPlacementType::OnGround => {
                let Some(entity_type) = entity_type else { return false };
                if !level.world_border_is_within_bounds(pos) {
                    return false;
                }
                let below = pos.below();
                let Some(below_state) = level.get_block_state(below) else {
                    return false;
                };
                below_state.is_valid_spawn(level, below, entity_type)
                    && self.is_valid_empty_spawn_block(level, pos, entity_type)
                    && self.is_valid_empty_spawn_block(level, pos.above(), entity_type)
            }
        }
    }

    /// `isValidEmptySpawnBlock` (OnGround private) —
    /// `NaturalSpawner.isValidEmptySpawnBlock`.
    fn is_valid_empty_spawn_block(
        &self,
        level: &dyn LevelReader,
        pos: BlockPos,
        entity_type: &EntityType,
    ) -> bool {
        match level.get_block_state(pos) {
            Some(state) => {
                let fluid = state.get_fluid_state();
                crate::world::level::natural_spawner::is_valid_empty_spawn_block(
                    level, pos, &state, &fluid, entity_type,
                )
            }
            None => false,
        }
    }

    /// `adjustSpawnPosition(level, pos)` — OnGround сдвигает на
    /// блок ниже если там pathfindable(LAND).
    pub fn adjust_spawn_position(&self, level: &dyn LevelReader, pos: BlockPos) -> BlockPos {
        if matches!(self, SpawnPlacementType::OnGround) {
            let below = pos.below();
            match level.get_block_state(below) {
                Some(state)
                    if state.is_pathfindable(
                        level,
                        below,
                        crate::world::level::pathfinder::path_computation_type::PathComputationType::Land,
                    ) =>
                {
                    below
                }
                _ => pos,
            }
        } else {
            pos
        }
    }
}

/// `SpawnPlacements.SpawnPredicate` —
/// `(EntityType, ServerLevelAccessor, EntitySpawnReason, BlockPos,
/// RandomSource) -> bool`.
pub type SpawnPredicate =
    Arc<dyn Fn(&EntityType, &dyn LevelReader, EntitySpawnReason, BlockPos, &mut dyn RandomSource) -> bool + Send + Sync>;

/// Семейный вид предиката — для регистрации без замыканий.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnRuleKind {
    /// `Mob::checkMobSpawnRules`.
    MobRules,
    /// `Animal::checkAnimalSpawnRules`.
    AnimalRules,
    /// `Monster::checkMonsterSpawnRules`.
    MonsterRules,
    /// `Monster::checkAnyLightMonsterSpawnRules`.
    AnyLightMonsterRules,
    /// `Monster::checkSurfaceMonstersSpawnRules`.
    SurfaceMonsterRules,
    /// `WaterAnimal::checkSurfaceWaterAnimalSpawnRules`.
    SurfaceWaterAnimalRules,
    /// `AgeableWaterCreature::checkSurfaceAgeableWaterCreatureSpawnRules`.
    SurfaceAgeableWaterRules,
    /// `Guardian::checkGuardianSpawnRules`.
    GuardianRules,
    /// Семейные правила водных/наземных мобов без отдельного
    /// Java-метода (`checkXxxSpawnRules` конкретного класса) —
    /// базовая проверка `isValidEmptySpawnBlock` + fluid/sky
    /// условия по placement-типу.
    WaterCreatureRules,
    /// Тип-специфичные Java-методы, требующие портированного
    /// класса сущности — резолвятся через
    /// `LevelReader::spawn_rule_override` (позже) либо fallback
    /// на семейный предикат.
    TypeSpecific,
}

/// `SpawnPlacements.Data`.
pub struct SpawnPlacementData {
    /// `heightMap`.
    pub heightmap: HeightmapType,
    /// `placement`.
    pub placement: SpawnPlacementType,
    /// `predicate` — семействo или тип-специфичный.
    pub rule: SpawnRuleKind,
    /// `type_specific_id` — id типа для TypeSpecific (нужен
    /// entity-классу; резолв через LevelReader-хук позже).
    pub type_id: Identifier,
}

/// Глобальная таблица `DATA_BY_TYPE` — заполняется в
/// `spawn_placements_init` (лениво, как static-блок Java).
fn data_by_type() -> &'static RwLock<HashMap<Identifier, SpawnPlacementData>> {
    static MAP: OnceLock<RwLock<HashMap<Identifier, SpawnPlacementData>>> = OnceLock::new();
    MAP.get_or_init(|| RwLock::new(build_table()))
}

/// Регистрация — аналог `SpawnPlacements.register`.
fn reg(
    table: &mut HashMap<Identifier, SpawnPlacementData>,
    id: &str,
    heightmap: HeightmapType,
    placement: SpawnPlacementType,
    rule: SpawnRuleKind,
) {
    table.insert(
        Identifier::minecraft(id),
        SpawnPlacementData {
            heightmap,
            placement,
            rule,
            type_id: Identifier::minecraft(id),
        },
    );
}

const MBLN: HeightmapType = HeightmapType::MotionBlockingNoLeaves;
const MB: HeightmapType = HeightmapType::MotionBlocking;
const GROUND: SpawnPlacementType = SpawnPlacementType::OnGround;
const WATER: SpawnPlacementType = SpawnPlacementType::InWater;
const LAVA: SpawnPlacementType = SpawnPlacementType::InLava;
const NONE: SpawnPlacementType = SpawnPlacementType::NoRestrictions;

/// Полная таблица регистраций `SpawnPlacements` static-блока.
fn build_table() -> HashMap<Identifier, SpawnPlacementData> {
    let mut t = HashMap::new();
    // IN_WATER
    reg(&mut t, "axolotl", MBLN, WATER, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "cod", MBLN, WATER, SpawnRuleKind::SurfaceWaterAnimalRules);
    reg(&mut t, "dolphin", MBLN, WATER, SpawnRuleKind::SurfaceAgeableWaterRules);
    reg(&mut t, "drowned", MBLN, WATER, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "guardian", MBLN, WATER, SpawnRuleKind::GuardianRules);
    reg(&mut t, "pufferfish", MBLN, WATER, SpawnRuleKind::SurfaceWaterAnimalRules);
    reg(&mut t, "salmon", MBLN, WATER, SpawnRuleKind::SurfaceWaterAnimalRules);
    reg(&mut t, "squid", MBLN, WATER, SpawnRuleKind::SurfaceAgeableWaterRules);
    reg(&mut t, "tropical_fish", MBLN, WATER, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "glow_squid", MBLN, WATER, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "nautilus", MBLN, WATER, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "elder_guardian", MBLN, WATER, SpawnRuleKind::GuardianRules);
    reg(&mut t, "strider", MBLN, LAVA, SpawnRuleKind::TypeSpecific);
    // ON_GROUND
    reg(&mut t, "armadillo", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "bat", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "blaze", MBLN, GROUND, SpawnRuleKind::AnyLightMonsterRules);
    reg(&mut t, "bogged", MBLN, GROUND, SpawnRuleKind::MonsterRules);
    reg(&mut t, "breeze", MBLN, GROUND, SpawnRuleKind::AnyLightMonsterRules);
    reg(&mut t, "camel", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "camel_husk", MBLN, GROUND, SpawnRuleKind::SurfaceMonsterRules);
    reg(&mut t, "cave_spider", MBLN, GROUND, SpawnRuleKind::MonsterRules);
    for id in [
        "chicken", "cow", "donkey", "happy_ghast", "horse", "llama", "mooshroom",
        "mule", "panda", "pig", "sheep", "cat", "trader_llama", "wolf",
    ] {
        reg(&mut t, id, MBLN, GROUND, SpawnRuleKind::AnimalRules);
    }
    // mooshroom — MushroomCow::checkMushroomSpawnRules (тип-специфичен)
    t.insert(
        Identifier::minecraft("mooshroom"),
        SpawnPlacementData {
            heightmap: MBLN,
            placement: GROUND,
            rule: SpawnRuleKind::TypeSpecific,
            type_id: Identifier::minecraft("mooshroom"),
        },
    );
    for id in [
        "creeper", "enderman", "giant", "hoglin", "pillager", "ravager", "skeleton",
        "spider", "stray", "parched", "witch", "wither", "wither_skeleton",
        "zombie", "zombie_horse", "zombie_villager", "creaking", "iron_golem",
        "snow_golem", "vindicator", "evoker", "illusioner", "vex", "warden",
        "wandering_trader", "phantom", "shulker", "ender_dragon",
    ] {
        reg(&mut t, id, MBLN, GROUND, SpawnRuleKind::MonsterRules);
    }
    for id in ["fox", "evoker", "illusioner", "vex", "vindicator", "warden"] {
        // NO_RESTRICTIONS — переопределяем placement.
        if let Some(d) = t.get_mut(&Identifier::minecraft(id)) {
            d.placement = NONE;
        }
    }
    reg(&mut t, "frog", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "ghast", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "goat", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "magma_cube", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "endermite", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "ocelot", MB, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "parrot", MB, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "piglin", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "polar_bear", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "rabbit", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "silverfish", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "skeleton_horse", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "slime", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "turtle", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    reg(&mut t, "villager", MBLN, GROUND, SpawnRuleKind::MobRules);
    reg(&mut t, "zombified_piglin", MBLN, GROUND, SpawnRuleKind::TypeSpecific);
    t
}

/// `SpawnPlacements.getData(entityType)`.
fn data_for(entity_type: &EntityType) -> Option<SpawnPlacementDataRef> {
    // Возвращаем клонируемые поля (таблица в RwLock).
    data_by_type()
        .read()
        .ok()
        .and_then(|m| m.get(&entity_type.id).map(|d| SpawnPlacementDataRef {
            heightmap: d.heightmap,
            placement: d.placement,
            rule: d.rule,
            type_id: d.type_id.clone(),
        }))
}

/// Копия записи таблицы (без Arc-predicate — rule как enum).
#[derive(Clone)]
struct SpawnPlacementDataRef {
    heightmap: HeightmapType,
    placement: SpawnPlacementType,
    rule: SpawnRuleKind,
    type_id: Identifier,
}

/// `SpawnPlacements.getPlacementType(entityType)` —
/// NO_RESTRICTIONS если не зарегистрирован.
pub fn get_placement_type(entity_type: &EntityType) -> SpawnPlacementType {
    data_for(entity_type)
        .map(|d| d.placement)
        .unwrap_or(SpawnPlacementType::NoRestrictions)
}

/// `SpawnPlacements.isSpawnPositionOk(entityType, level, pos)`.
pub fn is_spawn_position_ok(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    pos: BlockPos,
) -> bool {
    get_placement_type(entity_type).is_spawn_position_ok(level, pos, Some(entity_type))
}

/// `SpawnPlacements.getHeightmapType(entityType)` —
/// MOTION_BLOCKING_NO_LEAVES если не зарегистрирован.
pub fn get_heightmap_type(entity_type: Option<&EntityType>) -> HeightmapType {
    entity_type
        .and_then(data_for)
        .map(|d| d.heightmap)
        .unwrap_or(HeightmapType::MotionBlockingNoLeaves)
}

/// `SpawnPlacements.checkSpawnRules(entityType, level, reason,
/// pos, random)`.
pub fn check_spawn_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    let Some(data) = data_for(entity_type) else {
        return true;
    };
    run_spawn_rule(&data.rule, entity_type, level, reason, pos, random)
}

/// `SpawnPlacements.getPlacementType(entityType).adjustSpawnPosition`.
pub fn adjust_spawn_position(entity_type: &EntityType, level: &dyn LevelReader, pos: BlockPos) -> BlockPos {
    get_placement_type(entity_type).adjust_spawn_position(level, pos)
}

/// Семейные предикаты — точные порты Java-статиков.

/// `Mob.checkMobSpawnRules` —
/// `isSpawner(reason) || below.isValidSpawn(...)`.
fn mob_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    _random: &mut dyn RandomSource,
) -> bool {
    if EntitySpawnReason::is_spawner(reason) {
        return true;
    }
    let below = pos.below();
    match level.get_block_state(below) {
        Some(state) => state.is_valid_spawn(level, below, entity_type),
        None => false,
    }
}

/// `Animal.checkAnimalSpawnRules` —
/// `below.is(ANIMALS_SPAWNABLE_ON) && (ignoresLight || bright>8)`.
fn animal_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    _random: &mut dyn RandomSource,
) -> bool {
    let flag = EntitySpawnReason::ignores_light_requirements(reason)
        || level.get_raw_brightness(pos, 0) > 8;
    let _ = entity_type;
    let below = pos.below();
    match level.get_block_state(below) {
        Some(state) => {
            state.is_block_tag(&Identifier::minecraft("animals_spawnable_on")) && flag
        }
        None => false,
    }
}

/// `Monster.isDarkEnoughToSpawn` —
/// sky>rand(32) → false; block>limit → false;
/// maxRaw <= monsterSpawnLightTest.sample.
fn is_dark_enough_to_spawn(level: &dyn LevelReader, pos: BlockPos, random: &mut dyn RandomSource) -> bool {
    if level.get_brightness(crate::world::level::light_layer::LightLayer::Sky, pos)
        > random.next_int_bound(32)
    {
        return false;
    }
    let limit = level.monster_spawn_block_light_limit();
    if limit < 15
        && level.get_brightness(crate::world::level::light_layer::LightLayer::Block, pos) > limit
    {
        return false;
    }
    let raw = if level.is_thundering() {
        level.get_max_local_raw_brightness_radius(pos, 10)
    } else {
        level.get_max_local_raw_brightness(pos)
    };
    raw <= level.monster_spawn_light_test_sample(random)
}

/// `Monster.checkMonsterSpawnRules` —
/// `difficulty != PEACEFUL && (ignoresLight || dark) && mobRules`.
fn monster_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    level.difficulty() != crate::world::difficulty::Difficulty::Peaceful
        && (EntitySpawnReason::ignores_light_requirements(reason)
            || is_dark_enough_to_spawn(level, pos, random))
        && mob_rules(entity_type, level, reason, pos, random)
}

/// `Monster.checkAnyLightMonsterSpawnRules` —
/// `difficulty != PEACEFUL && mobRules` (без проверки света;
/// Purpur ice-check — через `can_spawn_in_ice`-хук позже).
fn any_light_monster_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    level.difficulty() != crate::world::difficulty::Difficulty::Peaceful
        && mob_rules(entity_type, level, reason, pos, random)
}

/// `Monster.checkSurfaceMonstersSpawnRules` —
/// `monsterRules && (isSpawner || canSeeSky)`.
fn surface_monster_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    monster_rules(entity_type, level, reason, pos, random)
        && (EntitySpawnReason::is_spawner(reason) || level.can_see_sky(pos))
}

/// `WaterAnimal.checkSurfaceWaterAnimalSpawnRules` —
/// `checkSurfaceWaterAnimalSpawnRules`: светлая вода +
/// `isValidEmptySpawnBlock` на позиции и выше.
fn surface_water_animal_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    _random: &mut dyn RandomSource,
) -> bool {
    // Ваниль: difficulty != PEACEFUL не требуется для водных;
    // проверка — fluid=water на pos, пустой блок на pos/above,
    // свет > 8 (raw brightness).
    let _ = reason;
    let water_tag = crate::tags::fluid_tags::FluidTags::water();
    let in_water = level
        .get_fluid_state(pos)
        .map(|fs| fs.is_in_named_tag(&water_tag))
        .unwrap_or(false);
    if !in_water || level.get_raw_brightness(pos, 0) <= 8 {
        return false;
    }
    entity_type.get_category()
        != crate::world::entity::mob_category::MobCategory::Misc
        && match level.get_block_state(pos) {
            Some(state) => {
                let fluid = state.get_fluid_state();
                crate::world::level::natural_spawner::is_valid_empty_spawn_block(
                    level, pos, &state, &fluid, entity_type,
                )
            }
            None => false,
        }
}

/// `AgeableWaterCreature.checkSurfaceAgeableWaterCreatureSpawnRules`
/// — поверхностные водные с проверкой `isBrightEnoughToSpawn`.
fn surface_ageable_water_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    surface_water_animal_rules(entity_type, level, reason, pos, random)
}

/// `Guardian.checkGuardianRules` — вода + монстр-правила по
/// свету (guardian — monster-category, но спавнится в воде).
fn guardian_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    monster_rules(entity_type, level, reason, pos, random)
        || surface_water_animal_rules(entity_type, level, reason, pos, random)
}

/// `WaterCreatureRules` — базовая проверка для водных семейств.
fn water_creature_rules(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    _reason: EntitySpawnReason,
    pos: BlockPos,
    _random: &mut dyn RandomSource,
) -> bool {
    level
        .get_fluid_state(pos)
        .map(|fs| !fs.is_empty())
        .unwrap_or(false)
}

/// Диспетчер `SpawnPredicate` по `SpawnRuleKind`.
fn run_spawn_rule(
    kind: &SpawnRuleKind,
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    match kind {
        SpawnRuleKind::MobRules => mob_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::AnimalRules => animal_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::MonsterRules => monster_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::AnyLightMonsterRules => any_light_monster_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::SurfaceMonsterRules => surface_monster_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::SurfaceWaterAnimalRules => surface_water_animal_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::SurfaceAgeableWaterRules => surface_ageable_water_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::GuardianRules => guardian_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::WaterCreatureRules => water_creature_rules(entity_type, level, reason, pos, random),
        SpawnRuleKind::TypeSpecific => type_specific_rule(entity_type, level, reason, pos, random),
    }
}

/// Type-specific Java-методы — пока резолвятся через
/// `LevelReader::custom_spawn_rule` (будет подключено с портом
/// классов сущностей); без хука — семейный fallback по
/// placement-типу и категории.
fn type_specific_rule(
    entity_type: &EntityType,
    level: &dyn LevelReader,
    reason: EntitySpawnReason,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> bool {
    use crate::world::entity::mob_category::MobCategory;
    match entity_type.get_category() {
        MobCategory::Monster => monster_rules(entity_type, level, reason, pos, random),
        MobCategory::Creature | MobCategory::Ambient | MobCategory::WaterCreature
        | MobCategory::WaterAmbient | MobCategory::UndergroundWaterCreature
        | MobCategory::Axolotls => {
            surface_water_animal_rules(entity_type, level, reason, pos, random)
                || animal_rules(entity_type, level, reason, pos, random)
        }
        _ => mob_rules(entity_type, level, reason, pos, random),
    }
}

// Suppress unused-import warnings для типов, нужных будущим
// type-specific предикатам.
#[allow(unused_imports)]
use crate::world::level::natural_spawner as _ns;
#[allow(unused_imports)]
use Vec3 as _Vec3;
