//! Перенос `net.minecraft.world.level.NaturalSpawner` —
//! естественный спавн мобов по чанкам (mob caps, spawn potential,
//! per-player caps).
//!
//! Перенесено в этом файле:
//! - `SpawnState` — глобальные/локальные mob caps + `PotentialCalculator`
//!   (energy budget по `MobSpawnCost`) + `chunkCap` (Leaf per-player).
//! - `createState` / `createStateAsync` — сбор состояния из списка
//!   сущностей.
//! - `getFilteredSpawningCategories` — фильтрация категорий по
//!   флагам friendlies/enemies/passives и лимитам.
//! - `globalLimitForCategory` — Paper mobcaps-формула.
//!
//! Адаптации:
//! - `Mob`/`ChunkAccess`/`Biome`-lookup заменены на `Entity` +
//!   resolver-замыкания (`mob_spawn_cost_at`, `rough_biome_at`) —
//!   ServerLevel/ChunkAccess реализуют их при wiring.
//! - Bukkit/Paper/Leaf конфиги (perPlayerMobSpawns, countAllMobs,
//!   spawnReason-фильтры, throttling, DEBUG_IGNORE_LOCAL_MOB_CAP)
//!   — через параметры/хуки; ванильная семантика сохранена.
//! - `MobCategory` → `usize`-индексы по `VALUES` (ordinal).

use std::collections::HashMap;
use std::rc::Rc;

use crate::core::block_pos::BlockPos;
use crate::world::entity::entity::Entity;
use crate::world::entity::mob_category::MobCategory;
use crate::world::level::biome::mob_spawn_settings::MobSpawnCost;
use crate::world::level::local_mob_cap_calculator::LocalMobCapCalculator;
use crate::world::level::potential_calculator::PotentialCalculator;

/// `MIN_SPAWN_DISTANCE`.
pub const MIN_SPAWN_DISTANCE: i32 = 24;
/// `SPAWN_DISTANCE_CHUNK`.
pub const SPAWN_DISTANCE_CHUNK: i32 = 8;
/// `SPAWN_DISTANCE_BLOCK`.
pub const SPAWN_DISTANCE_BLOCK: i32 = 128;
/// `INSCRIBED_SQUARE_SPAWN_DISTANCE_CHUNK`.
pub const INSCRIBED_SQUARE_SPAWN_DISTANCE_CHUNK: i32 = 5;
/// `MAGIC_NUMBER` — `(int)pow(17,2)`.
pub const MAGIC_NUMBER: i32 = 289;

/// `SPAWNING_CATEGORIES` — все категории кроме MISC.
pub fn spawning_categories() -> Vec<MobCategory> {
    MobCategory::VALUES
        .iter()
        .copied()
        .filter(|c| *c != MobCategory::Misc)
        .collect()
}

/// `ChunkGetter` — запрос чанка по ключу (для createState).
pub trait ChunkGetter {
    /// `query(chunkPos, consumer)` — вызвать consumer для чанка,
    /// если он загружен.
    fn query(&self, chunk_key: i64, f: &mut dyn FnMut(&dyn ChunkAccessLike));
}

/// Минимальный интерфейс чанка для `getRoughBiome`/`getMobSpawnCost` —
/// реализуется `LevelChunk`/`ChunkAccess` при wiring.
pub trait ChunkAccessLike {
    /// `getNoiseBiome(quartX, quartY, quartZ).value()` — биом в
    /// quart-координатах. Возвращает `MobSpawnSettings`-совместимый
    /// snapshot: `mob_spawn_cost(entity_type_id)`.
    fn mob_spawn_cost(&self, entity_type_id: &crate::resources::identifier::Identifier) -> Option<MobSpawnCost>;
    /// `getPos()` — ChunkPos ключ.
    fn chunk_key(&self) -> i64;
}

/// `getRoughBiome(pos, chunk).getMobSettings().getMobSpawnCost(type)` —
/// стоимость спавна сущности в биоме чанка.
pub fn mob_spawn_cost_at(
    chunk: &dyn ChunkAccessLike,
    pos: BlockPos,
    entity_type_id: &crate::resources::identifier::Identifier,
) -> Option<MobSpawnCost> {
    let _ = pos;
    chunk.mob_spawn_cost(entity_type_id)
}

/// `createState(spawnableChunkCount, entities, chunkGetter, calculator)` —
/// сбор `SpawnState` из итерации сущностей.
///
/// - `spawn_reason_natural(entity)` — фильтр Paper (NATURAL/CHUNK_GEN);
///   default-реализация пропускает фильтр (countAllMobs=true).
/// - `is_persistent(entity)` — `mob.isPersistenceRequired() ||
///   requiresCustomPersistence()` — такие сущности не считаются.
pub fn create_state(
    spawnable_chunk_count: i32,
    entities: &[Entity],
    chunk_getter: &dyn ChunkGetter,
    mut calculator: Option<LocalMobCapCalculator>,
    is_persistent: &dyn Fn(&Entity) -> bool,
    is_natural_spawn: &dyn Fn(&Entity) -> bool,
) -> SpawnState {
    let mut potential_calculator = PotentialCalculator::new();
    let mut counts: HashMap<MobCategory, i32> = HashMap::new();

    for entity in entities {
        if is_persistent(entity) {
            continue;
        }
        let Some(category) = entity_category(entity) else {
            continue;
        };
        if category == MobCategory::Misc {
            continue;
        }
        if !is_natural_spawn(entity) {
            continue;
        }
        let pos = entity.block_position();
        let chunk_key = crate::world::level::chunk_pos::ChunkPos::as_long_static(
            pos.x >> 4,
            pos.z >> 4,
        );
        chunk_getter.query(chunk_key, &mut |chunk| {
            if let Some(cost) = mob_spawn_cost_at(
                chunk,
                pos,
                &crate::world::entity::entity_type::holder_type_id(&entity.entity_type)
                    .unwrap_or_else(|| crate::resources::identifier::Identifier::minecraft("pig")),
            ) {
                potential_calculator.add_charge(pos, cost.charge);
            }
            if let Some(c) = calculator.as_mut() {
                c.add_mob(chunk_key, category);
            }
        });
        *counts.entry(category).or_insert(0) += 1;
    }

    SpawnState::new(spawnable_chunk_count, counts, potential_calculator, calculator)
}

/// `entity.getType().getCategory()` — категория по holder'у.
pub fn entity_category(entity: &Entity) -> Option<MobCategory> {
    crate::world::entity::entity_type::holder_type_category(&entity.entity_type)
}

/// `createStateAsync` (Leaf) — то же, но с `chunkCap` per-player
/// массивами `[MobCategory; 8]` по чанку.
pub fn create_state_async(
    spawnable_chunk_count: i32,
    entities: &[Entity],
    is_persistent: &dyn Fn(&Entity) -> bool,
    is_natural_spawn: &dyn Fn(&Entity) -> bool,
    chunk_cost_lookup: &dyn Fn(BlockPos, &crate::resources::identifier::Identifier) -> Option<MobCategoryPair>,
) -> SpawnState {
    let mut potential_calculator = PotentialCalculator::new();
    let mut counts: HashMap<MobCategory, i32> = HashMap::new();
    let mut chunk_cap: HashMap<i64, Vec<i32>> = HashMap::new();

    for entity in entities {
        if is_persistent(entity) {
            continue;
        }
        let Some(category) = entity_category(entity) else {
            continue;
        };
        if category == MobCategory::Misc || !is_natural_spawn(entity) {
            continue;
        }
        let pos = entity.block_position();
        let type_id = crate::world::entity::entity_type::holder_type_id(&entity.entity_type)
            .unwrap_or_else(|| crate::resources::identifier::Identifier::minecraft("pig"));
        if let Some(pair) = chunk_cost_lookup(pos, &type_id) {
            if let Some(cost) = pair.cost {
                potential_calculator.add_charge(pos, cost.charge);
            }
            let chunk_key = crate::world::level::chunk_pos::ChunkPos::as_long_static(pos.x >> 4, pos.z >> 4);
            let idx = category as usize;
            let cap = chunk_cap.entry(chunk_key).or_insert_with(|| vec![0; MobCategory::VALUES.len()]);
            cap[idx] += 1;
        }
        *counts.entry(category).or_insert(0) += 1;
    }

    SpawnState::new_with_chunk_cap(spawnable_chunk_count, counts, potential_calculator, None, chunk_cap)
}

/// Пара `(category, cost)` для `chunk_cost_lookup` — категория чанка
/// и `MobSpawnCost` если есть.
pub struct MobCategoryPair {
    /// `cost` — `MobSpawnCost` из биома.
    pub cost: Option<MobSpawnCost>,
}

/// `getFilteredSpawningCategories(spawnState, spawnFriendlies,
/// spawnEnemies, spawnPassives, limits)` — фильтр категорий.
///
/// `limit_for(category)` — лимит спавна категории на чанк
/// (vanilla `getMaxInstancesPerChunk` или world spawn limit).
/// `spawn_this_tick(category)` — ticksPerSpawnCategory-проверка.
pub fn get_filtered_spawning_categories(
    spawn_state: &SpawnState,
    spawn_friendlies: bool,
    spawn_enemies: bool,
    spawn_passives: bool,
    limit_for: &dyn Fn(MobCategory) -> i32,
    spawn_this_tick: &dyn Fn(MobCategory) -> bool,
) -> Vec<MobCategory> {
    let mut list = Vec::new();
    for &category in &spawning_categories() {
        let limit = limit_for(category);
        let this_tick = spawn_this_tick(category);
        if !this_tick || limit == 0 {
            continue;
        }
        if (spawn_friendlies || !category.is_friendly())
            && (spawn_enemies || category.is_friendly())
            && (spawn_passives || !category.is_persistent())
            && spawn_state.can_spawn_for_category_global(category, limit)
        {
            list.push(category);
        }
    }
    list
}

/// `globalLimitForCategory(level, category, spawnableChunkCount)` —
/// Paper mobcaps: `spawnLimit * spawnableChunkCount / MAGIC_NUMBER`.
pub fn global_limit_for_category(spawn_limit: i32, spawnable_chunk_count: i32) -> i32 {
    if spawn_limit < 1 {
        return spawn_limit;
    }
    spawn_limit * spawnable_chunk_count / MAGIC_NUMBER
}

/// `NaturalSpawner.SpawnState` — состояние спавна на тик.
pub struct SpawnState {
    /// `spawnableChunkCount`.
    spawnable_chunk_count: i32,
    /// `mobCategoryCounts` — текущие счётчики по категориям.
    mob_category_counts: HashMap<MobCategory, i32>,
    /// `spawnPotential` — energy budget.
    spawn_potential: PotentialCalculator,
    /// `localMobCapCalculator` — per-player caps (Paper-опция).
    local_mob_cap_calculator: Option<LocalMobCapCalculator>,
    /// `lastCheckedPos`/`lastCheckedType`/`lastCharge` — кэш
    /// последней canSpawn-проверки для afterSpawn.
    last_checked_pos: Option<BlockPos>,
    last_checked_type: Option<crate::resources::identifier::Identifier>,
    last_charge: f64,
    /// `chunkCap` — per-chunk per-category счётчики (Leaf).
    pub chunk_cap: HashMap<i64, Vec<i32>>,
}

impl SpawnState {
    fn new(
        spawnable_chunk_count: i32,
        counts: HashMap<MobCategory, i32>,
        spawn_potential: PotentialCalculator,
        calculator: Option<LocalMobCapCalculator>,
    ) -> Self {
        Self::new_with_chunk_cap(spawnable_chunk_count, counts, spawn_potential, calculator, HashMap::new())
    }

    fn new_with_chunk_cap(
        spawnable_chunk_count: i32,
        counts: HashMap<MobCategory, i32>,
        spawn_potential: PotentialCalculator,
        calculator: Option<LocalMobCapCalculator>,
        chunk_cap: HashMap<i64, Vec<i32>>,
    ) -> Self {
        Self {
            spawnable_chunk_count,
            mob_category_counts: counts,
            spawn_potential,
            local_mob_cap_calculator: calculator,
            last_checked_pos: None,
            last_checked_type: None,
            last_charge: 0.0,
            chunk_cap,
        }
    }

    /// `canSpawn(entityType, pos, chunk)` — energy-budget проверка.
    /// `mob_spawn_cost` резолвится снаружи (biome lookup).
    pub fn can_spawn(
        &mut self,
        entity_type_id: &crate::resources::identifier::Identifier,
        pos: BlockPos,
        mob_spawn_cost: Option<MobSpawnCost>,
    ) -> bool {
        self.last_checked_pos = Some(pos);
        self.last_checked_type = Some(entity_type_id.clone());
        match mob_spawn_cost {
            None => {
                self.last_charge = 0.0;
                true
            }
            Some(cost) => {
                let charge = cost.charge;
                self.last_charge = charge;
                let change = self.spawn_potential.get_potential_energy_change(pos, charge);
                change <= cost.energy_budget
            }
        }
    }

    /// `afterSpawn(mob, chunk)` — добавить charge после спавна.
    pub fn after_spawn(
        &mut self,
        entity_type_id: &crate::resources::identifier::Identifier,
        category: MobCategory,
        pos: BlockPos,
        mob_spawn_cost: Option<MobSpawnCost>,
    ) {
        let d = if self.last_checked_pos == Some(pos)
            && self.last_checked_type.as_ref() == Some(entity_type_id)
        {
            self.last_charge
        } else {
            mob_spawn_cost.map(|c| c.charge).unwrap_or(0.0)
        };
        self.spawn_potential.add_charge(pos, d);
        *self.mob_category_counts.entry(category).or_insert(0) += 1;
        if let Some(calc) = &mut self.local_mob_cap_calculator {
            calc.add_mob(crate::world::level::chunk_pos::ChunkPos::as_long_static(pos.x >> 4, pos.z >> 4), category);
        }
    }

    /// `getSpawnableChunkCount()`.
    pub fn spawnable_chunk_count(&self) -> i32 {
        self.spawnable_chunk_count
    }

    /// `getMobCategoryCounts()` — read-only снимок.
    pub fn mob_category_counts(&self) -> &HashMap<MobCategory, i32> {
        &self.mob_category_counts
    }

    /// `canSpawnForCategoryGlobal(category, limit)` —
    /// `count < limit * spawnableChunkCount / MAGIC_NUMBER`.
    pub fn can_spawn_for_category_global(&self, category: MobCategory, limit: i32) -> bool {
        let i = limit * self.spawnable_chunk_count / MAGIC_NUMBER;
        self.mob_category_counts.get(&category).copied().unwrap_or(0) < i
    }

    /// `canSpawnForCategoryLocal(category, chunkPos)` —
    /// per-player cap либо `DEBUG_IGNORE_LOCAL_MOB_CAP`.
    pub fn can_spawn_for_category_local(
        &mut self,
        category: MobCategory,
        chunk_key: i64,
        ignore_local_cap: bool,
    ) -> bool {
        ignore_local_cap
            || match &mut self.local_mob_cap_calculator {
                Some(calc) => calc.can_spawn(category, chunk_key),
                None => true,
            }
    }
}

/// Утилита: `SharedConstants.DEBUG_IGNORE_LOCAL_MOB_CAP` —
/// константа из SharedConstants (debug-флаг).
pub fn debug_ignore_local_mob_cap() -> bool {
    // `SharedConstants.DEBUG_IGNORE_LOCAL_MOB_CAP` — debug-флаг,
    // по умолчанию выключен.
    false
}

// Suppress: Rc импортирован для будущих provider-замыканий.
#[allow(unused_imports)]
use std::rc::Rc as _Rc;

// === Основной цикл спавна (spawnForChunk/spawnCategoryForPosition) ===

use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::entity::entity_spawn_reason::EntitySpawnReason;
use crate::entity::entity_type::EntityType;
use crate::world::entity::mob::Mob;
use crate::world::entity::spawn_placements;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::phys::vec3::Vec3;

/// `PreSpawnStatus` (Paper) — результат проверки позиции до
/// создания моба.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreSpawnStatus {
    /// FAIL — позиция/правила не подходят.
    Fail,
    /// SUCCESS — всё ок, спавнить.
    Success,
    /// CANCELLED — событие отменило спавн (не abort).
    Cancelled,
    /// ABORT — событие прервало весь цикл.
    Abort,
}

/// Расширение `ChunkAccessLike` для спавн-цикла: block-state и
/// heightmap внутри чанка (`LevelChunk.getBlockStateFinal` /
/// `ChunkAccess.getHeight`).
pub trait SpawnChunkAccess: ChunkAccessLike {
    /// `getBlockStateFinal(x, y, z)` — состояние блока (final,
    /// без апгрейдов). None — вне чанка/не загружен.
    fn block_state_final(&self, x: i32, y: i32, z: i32) -> Option<BlockState>;
    /// `getHeight(Heightmap.Types, x, z)` — высота по heightmap.
    fn height(&self, ty: HeightmapType, x: i32, z: i32) -> i32;
}

/// `NaturalSpawner.isValidEmptySpawnBlock` —
/// `!isCollisionShapeFullBlock && !isSignalSource && fluid.isEmpty
/// && !PREVENT_MOB_SPAWNING_INSIDE && !entityType.isBlockDangerous`.
pub fn is_valid_empty_spawn_block(
    level: &dyn LevelReader,
    pos: BlockPos,
    state: &BlockState,
    fluid_state: &FluidState,
    entity_type: &EntityType,
) -> bool {
    !state.is_collision_shape_full_block(level, pos)
        && !state.is_signal_source()
        && fluid_state.is_empty()
        && !state.is_block_tag(&crate::resources::identifier::Identifier::minecraft(
            "prevent_mob_spawning_inside",
        ))
        && !entity_type.is_block_dangerous(state)
}

/// `isInNetherFortressBounds` — monster на nether_bricks внутри
/// fortress-структуры.
pub fn is_in_nether_fortress_bounds(
    pos: BlockPos,
    level: &dyn LevelReader,
    category: MobCategory,
) -> bool {
    if category != MobCategory::Monster {
        return false;
    }
    let below = pos.below();
    match level.get_block_state(below) {
        Some(state)
            if state.block.id
                == crate::resources::identifier::Identifier::minecraft("nether_bricks") => {}
        _ => return false,
    }
    level.nether_fortress_at(pos)
}

/// `NetherFortressStructure.FORTRESS_ENEMIES` — weighted list
/// (ленивая инициализация через entity-type registry).
pub fn fortress_enemies() -> crate::util::random::weighted_list::WeightedList<
    crate::world::level::biome::mob_spawn_settings::SpawnerData,
> {
    use crate::util::random::weighted_list::WeightedListBuilder;
    use crate::world::level::biome::mob_spawn_settings::SpawnerData;

    static LIST: std::sync::OnceLock<
        crate::util::random::weighted_list::WeightedList<SpawnerData>,
    > = std::sync::OnceLock::new();
    LIST.get_or_init(|| {
        let mut b = WeightedListBuilder::new();
        for (id, min, max, weight) in [
            ("blaze", 2, 3, 10),
            ("zombified_piglin", 4, 4, 5),
            ("wither_skeleton", 5, 5, 8),
            ("skeleton", 5, 5, 2),
            ("magma_cube", 4, 4, 3),
        ] {
            if let Some(t) = crate::entity::entity_type::entity_type_by_identifier(
                &Identifier::minecraft(id),
            ) {
                b.add_with_weight(SpawnerData::new(t, min, max), weight);
            }
        }
        b.build()
    })
    .clone()
}

/// `mobsAt` — список спавн-данных для категории в позиции:
/// fortress → FORTRESS_ENEMIES, иначе `biome.getMobSettings()
/// .getMobs(category)`.
pub fn mobs_at(
    level: &dyn LevelReader,
    category: MobCategory,
    pos: BlockPos,
) -> crate::util::random::weighted_list::WeightedList<
    crate::world::level::biome::mob_spawn_settings::SpawnerData,
> {
    use crate::world::level::biome::mob_spawn_settings::empty_mob_list;
    if is_in_nether_fortress_bounds(pos, level, category) {
        return fortress_enemies();
    }
    match level.biome_at(pos).and_then(|h| h.value()) {
        Some(biome) => biome.get_mob_settings().get_mobs(category),
        None => empty_mob_list(),
    }
}

/// `getRandomSpawnMobAt` — weighted random SpawnerData;
/// WATER_AMBIENT + REDUCED_WATER_AMBIENT_SPAWNS + 0.98 → пусто.
pub fn get_random_spawn_mob_at(
    level: &dyn LevelReader,
    category: MobCategory,
    random: &mut dyn RandomSource,
    pos: BlockPos,
) -> Option<crate::world::level::biome::mob_spawn_settings::SpawnerData> {
    if category == MobCategory::WaterAmbient
        && level.biome_has_tag(
            pos,
            &crate::resources::identifier::Identifier::minecraft(
                "reduced_water_ambient_spawns",
            ),
        )
        && random.next_float() < 0.98
    {
        return None;
    }
    mobs_at(level, category, pos).get_random(random).cloned()
}

/// `canSpawnMobAt` — `mobsAt(...).contains(data)`.
pub fn can_spawn_mob_at(
    level: &dyn LevelReader,
    category: MobCategory,
    data: &crate::world::level::biome::mob_spawn_settings::SpawnerData,
    pos: BlockPos,
) -> bool {
    mobs_at(level, category, pos).contains(data)
}

/// `getRandomPosWithin` — случайная позиция в чанке на высоте
/// WORLD_SURFACE + random Y.
pub fn get_random_pos_within(
    level: &dyn LevelReader,
    chunk_pos: crate::world::level::chunk_pos::ChunkPos,
    random: &mut dyn RandomSource,
) -> BlockPos {
    let min_x = chunk_pos.get_min_block_x();
    let min_z = chunk_pos.get_min_block_z();
    let x = min_x + random.next_int_bound(16);
    let z = min_z + random.next_int_bound(16);
    let surface = level.get_height(HeightmapType::WorldSurface, x, z) + 1;
    let y = crate::util::mth::random_between_inclusive(
        random,
        level.min_build_height(),
        surface,
    );
    BlockPos::new(x, y, z)
}

/// `isRightDistanceToPlayerAndSpawnPoint` —
/// `distance <= 576 → false`; респавн-точка в 24 блоках → false;
/// иначе `canSpawnEntitiesInChunk`.
pub fn is_right_distance_to_player_and_spawn_point(
    level: &dyn LevelReader,
    chunk_key: i64,
    pos: BlockPos,
    distance_sqr: f64,
) -> bool {
    if distance_sqr <= 576.0 {
        return false;
    }
    if let Some((dim, respawn_pos)) = level.respawn_data() {
        if Some(&dim) == level.dimension_id()
            && respawn_pos.distance_to_sqr(&Vec3::new(
                pos.x as f64 + 0.5,
                pos.y as f64,
                pos.z as f64 + 0.5,
            )) < 24.0 * 24.0
        {
            return false;
        }
    }
    let same = crate::world::level::chunk_pos::ChunkPos::as_long_static(pos.x >> 4, pos.z >> 4) == chunk_key;
    same || level.can_spawn_entities_in_chunk(crate::world::level::chunk_pos::ChunkPos::of(chunk_key))
}

/// `getMobForSpawn` — `entityType.create(level, NATURAL)` → Mob.
fn get_mob_for_spawn(level: &dyn LevelReader, entity_type: &EntityType) -> Option<Mob> {
    level.create_mob_for_spawn(entity_type)
}

/// `isValidPositionForMob` —
/// `(!(distance > despawn²) || !removeWhenFarAway) &&
/// checkSpawnRules && checkSpawnObstruction`.
fn is_valid_position_for_mob(
    level: &dyn LevelReader,
    mob: &Mob,
    distance_sqr: f64,
) -> bool {
    let category = mob
        .living
        .entity
        .entity_type
        .value()
        .map(|t| t.get_category())
        .unwrap_or(MobCategory::Misc);
    let despawn = category.get_despawn_distance() as f64;
    (!(distance_sqr > despawn * despawn) || !mob.remove_when_far_away(distance_sqr))
        && mob.check_spawn_rules(level, EntitySpawnReason::Natural)
        && mob.check_spawn_obstruction_full(level)
}

/// `isValidSpawnPostitionForType` — проверка позиции для типа:
/// MISC/canSpawnFarFromPlayer/canSummon/mobsAt содержит/
/// isSpawnPositionOk/checkSpawnRules/noCollision(spawnAABB).
fn is_valid_spawn_position_for_type(
    level: &dyn LevelReader,
    category: MobCategory,
    data: &crate::world::level::biome::mob_spawn_settings::SpawnerData,
    pos: BlockPos,
    distance_sqr: f64,
) -> PreSpawnStatus {
    let entity_type = &*data.entity_type;
    if let Some(status) = level.pre_spawn_status(entity_type, pos) {
        return status;
    }
    let category_of_type = entity_type.get_category();
    let despawn = category_of_type.get_despawn_distance() as f64;
    let success = category_of_type != MobCategory::Misc
        && (entity_type.can_spawn_far_from_player()
            || !(distance_sqr > despawn * despawn))
        && entity_type.can_summon()
        && can_spawn_mob_at(level, category, data, pos)
        && spawn_placements::is_spawn_position_ok(entity_type, level, pos)
        && spawn_placements::check_spawn_rules(
            entity_type,
            level,
            EntitySpawnReason::Natural,
            pos,
            &mut crate::world::level::base_spawner::LevelRandom::new(level),
        )
        && {
            let aabb = entity_type.get_spawn_aabb(
                pos.x as f64 + 0.5,
                pos.y as f64,
                pos.z as f64 + 0.5,
            );
            level.no_collision(&aabb)
        };
    if success {
        PreSpawnStatus::Success
    } else {
        PreSpawnStatus::Fail
    }
}

/// `spawnCategoryForPosition` — полный цикл спавна категории в
/// позиции (random-bit packed, group size, caps).
///
/// `spawn_state` — `Some` → filter/callback через
/// `can_spawn`/`after_spawn` (vanilla spawnForChunk); `None` —
/// debug-вариант (всегда true/no-op).
/// `max_spawns` — Paper per-player лимит.
/// `track_entity` — колбэк на каждого заспавненного моба.
#[allow(clippy::too_many_arguments)]
pub fn spawn_category_for_position(
    category: MobCategory,
    level: &dyn LevelReader,
    chunk: &dyn SpawnChunkAccess,
    pos: BlockPos,
    mut spawn_state: Option<&mut SpawnState>,
    max_spawns: i32,
    mut track_entity: Option<&mut dyn FnMut(&Mob)>,
    random: &mut dyn RandomSource,
) -> i32 {
    let y = pos.y;
    let pos_x = pos.x;
    let pos_z = pos.z;
    let mut i = 0i32;

    let chunk_key = chunk.chunk_key();
    let same_chunk = crate::world::level::chunk_pos::ChunkPos::as_long_static(pos_x >> 4, pos_z >> 4) == chunk_key;
    let block_state = if same_chunk {
        chunk.block_state_final(pos_x, y, pos_z)
    } else {
        level.get_block_state(pos)
    };
    let Some(block_state) = block_state else {
        return 0;
    };
    if block_state.is_redstone_conductor(level, pos) {
        return 0;
    }

    let mut rand = random.next_long();
    let mut bits = 0u32;
    for _i1 in 0..3 {
        let mut x = pos_x;
        let mut z = pos_z;
        let mut spawner_data: Option<crate::world::level::biome::mob_spawn_settings::SpawnerData> = None;
        let mut spawn_group_data: Option<crate::world::entity::mob::SpawnGroupData> = None;
        let mut ceil = (((rand >> bits) & 0x3) + 1) as i32;
        bits += 2;
        if bits >= 62 {
            rand = random.next_long();
            bits = 0;
        }
        let mut i3 = 0i32;

        for _i4 in 0..ceil {
            let (mut r1, mut r2, mut r3, mut r4) = (0i32, 0i32, 0i32, 0i32);
            let mut needed = 4;
            while needed > 0 {
                let three = ((rand >> bits) & 0x7) as i32;
                bits += 3;
                if three != 7 && three != 6 {
                    match needed {
                        1 => r4 = three,
                        2 => r3 = three,
                        3 => r2 = three,
                        _ => r1 = three,
                    }
                    needed -= 1;
                }
                if bits >= 62 {
                    rand = random.next_long();
                    bits = 0;
                }
            }
            x += r1 - r2;
            z += r3 - r4;

            let mutable = BlockPos::new(x, y, z);
            let d = x as f64 + 0.5;
            let d1 = z as f64 + 0.5;
            let d2 = level.nearest_player_distance_sqr(d, y as f64, d1);
            if d2 == f64::INFINITY {
                continue;
            }
            let chunk_matches =
                crate::world::level::chunk_pos::ChunkPos::as_long_static(x >> 4, z >> 4) == chunk_key;
            if !(chunk_matches || is_loaded_and_in_bounds(level, mutable)) {
                continue;
            }
            if !is_right_distance_to_player_and_spawn_point(level, chunk_key, mutable, d2) {
                continue;
            }

            if spawner_data.is_none() {
                match get_random_spawn_mob_at(level, category, random, mutable) {
                    None => break,
                    Some(data) => {
                        ceil = data.min_count
                            + random.next_int_bound(1 + data.max_count - data.min_count);
                        spawner_data = Some(data);
                    }
                }
            }
            let data = spawner_data.clone().unwrap();

            let status = is_valid_spawn_position_for_type(level, category, &data, mutable, d2);
            match status {
                PreSpawnStatus::Abort => return i,
                PreSpawnStatus::Cancelled => {}
                _ => {
                    if status == PreSpawnStatus::Success {
                        let can_spawn = match spawn_state.as_deref_mut() {
                            Some(state) => {
                                let cost = chunk.mob_spawn_cost(&data.entity_type.id);
                                state.can_spawn(&data.entity_type.id, mutable, cost)
                            }
                            None => true,
                        };
                        if !can_spawn {
                            continue;
                        }

                        let Some(mut mob) = get_mob_for_spawn(level, &data.entity_type) else {
                            return i;
                        };
                        mob.living.entity.snap_to(
                            d,
                            y as f64,
                            d1,
                            random.next_float() * 360.0,
                            0.0,
                        );
                        if is_valid_position_for_mob(level, &mob, d2) {
                            let difficulty = level.current_difficulty_at(mob.living.entity.block_position());
                            spawn_group_data = mob.finalize_spawn(
                                level,
                                difficulty as i32 as f32,
                                EntitySpawnReason::Natural,
                                spawn_group_data,
                            );
                            level.add_fresh_entity(mob.living.entity.clone());
                            if !mob.living.entity.is_removed() {
                                i += 1;
                                i3 += 1;
                                if let Some(state) = spawn_state.as_deref_mut() {
                                    let cost = chunk.mob_spawn_cost(&data.entity_type.id);
                                    state.after_spawn(
                                        &data.entity_type.id,
                                        data.entity_type.get_category(),
                                        mob.living.entity.block_position(),
                                        cost,
                                    );
                                }
                                if let Some(cb) = track_entity.as_deref_mut() {
                                    cb(&mob);
                                }
                                if i >= mob.get_max_spawn_cluster_size() || i >= max_spawns {
                                    return i;
                                }
                                if mob.is_max_group_size_reached(i3) {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    i
}

/// `level.isLoadedAndInBounds(pos)` — загружен и в границах
/// мира (world border).
fn is_loaded_and_in_bounds(level: &dyn LevelReader, pos: BlockPos) -> bool {
    level.is_inside_build_height(pos.y)
        && level.is_loaded(pos)
        && level.world_border_is_within_bounds(pos)
}

/// `spawnCategoryForChunk` с доступом к чанку —
/// `getRandomPosWithin` → `spawnCategoryForPosition`.
pub fn spawn_category_for_chunk_with_chunk(
    category: MobCategory,
    level: &dyn LevelReader,
    chunk: &dyn SpawnChunkAccess,
    spawn_state: Option<&mut SpawnState>,
    max_spawns: i32,
    track_entity: Option<&mut dyn FnMut(&Mob)>,
    random: &mut dyn RandomSource,
) -> i32 {
    let chunk_pos = crate::world::level::chunk_pos::ChunkPos::of(chunk.chunk_key());
    let pos = get_random_pos_within(level, chunk_pos, random);
    if pos.y < level.min_build_height() + 1 {
        return 0;
    }
    spawn_category_for_position(
        category,
        level,
        chunk,
        pos,
        spawn_state,
        max_spawns,
        track_entity,
        random,
    )
}

/// `spawnForChunk` — итерация категорий с локальными caps.
/// `chunk` — `SpawnChunkAccess` реализация LevelChunk.
pub fn spawn_for_chunk(
    level: &dyn LevelReader,
    chunk: &dyn SpawnChunkAccess,
    spawn_state: &mut SpawnState,
    categories: &[MobCategory],
    random: &mut dyn RandomSource,
) {
    for &category in categories {
        let can_spawn = spawn_state.can_spawn_for_category_local(
            category,
            chunk.chunk_key(),
            debug_ignore_local_mob_cap(),
        );
        if !can_spawn {
            continue;
        }
        spawn_category_for_chunk_with_chunk(
            category,
            level,
            chunk,
            Some(spawn_state),
            i32::MAX,
            None,
            random,
        );
    }
}

/// `getTopNonCollidingPos` — верхняя неколлизионная позиция
/// для типа сущности (heightmap + ceiling + adjustSpawnPosition).
pub fn get_top_non_colliding_pos(
    level: &dyn LevelReader,
    entity_type: &EntityType,
    x: i32,
    z: i32,
) -> BlockPos {
    let height = level.get_height(spawn_placements::get_heightmap_type(Some(entity_type)), x, z);
    let mut pos = BlockPos::new(x, height, z);
    if level.dimension_has_ceiling() {
        loop {
            pos = pos.below();
            let air = level
                .get_block_state(pos)
                .map(|s| s.is_air())
                .unwrap_or(false);
            if !air {
                break;
            }
        }
        loop {
            pos = pos.below();
            let air = level
                .get_block_state(pos)
                .map(|s| s.is_air())
                .unwrap_or(false);
            if !air || pos.y <= level.min_build_height() {
                break;
            }
        }
    }
    spawn_placements::adjust_spawn_position(entity_type, level, pos)
}

/// `spawnMobsForChunkGeneration` — спавн CREATURE-мобов при
/// генерации чанка (creatureProbability loop).
#[allow(clippy::too_many_arguments)]
pub fn spawn_mobs_for_chunk_generation(
    level: &dyn LevelReader,
    mob_settings: &crate::world::level::biome::mob_spawn_settings::MobSpawnSettings,
    chunk_pos: crate::world::level::chunk_pos::ChunkPos,
    random: &mut dyn RandomSource,
) {
    let mobs = mob_settings.get_mobs(MobCategory::Creature);
    if mobs.is_empty() || !level.spawn_mobs_game_rule() {
        return;
    }
    let min_block_x = chunk_pos.get_min_block_x();
    let min_block_z = chunk_pos.get_min_block_z();

    while random.next_float() < mob_settings.get_creature_probability() {
        let Some(spawner_data) = mobs.get_random(random).cloned() else {
            continue;
        };
        let count = spawner_data.min_count
            + random.next_int_bound(1 + spawner_data.max_count - spawner_data.min_count);
        let mut spawn_group_data: Option<crate::world::entity::mob::SpawnGroupData> = None;
        let mut x = min_block_x + random.next_int_bound(16);
        let mut z = min_block_z + random.next_int_bound(16);
        let x0 = x;
        let z0 = z;

        for _ in 0..count {
            let mut flag = false;
            for _i6 in 0..4 {
                if flag {
                    break;
                }
                let top_pos = get_top_non_colliding_pos(level, &spawner_data.entity_type, x, z);
                if spawner_data.entity_type.can_summon()
                    && spawn_placements::is_spawn_position_ok(&spawner_data.entity_type, level, top_pos)
                {
                    let width = spawner_data.entity_type.get_width() as f64;
                    let d = (x as f64).clamp(
                        min_block_x as f64 + width,
                        min_block_x as f64 + 16.0 - width,
                    );
                    let d1 = (z as f64).clamp(
                        min_block_z as f64 + width,
                        min_block_z as f64 + 16.0 - width,
                    );
                    let aabb = spawner_data.entity_type.get_spawn_aabb(d, top_pos.y as f64, d1);
                    let rules_ok = spawn_placements::check_spawn_rules(
                        &spawner_data.entity_type,
                        level,
                        EntitySpawnReason::ChunkGeneration,
                        BlockPos::new(d as i32, top_pos.y, d1 as i32),
                        random,
                    );
                    if !level.no_collision(&aabb) || !rules_ok {
                        // reposition below
                    } else if let Some(mut mob) = get_mob_for_spawn(level, &spawner_data.entity_type) {
                        mob.living.entity.snap_to(d, top_pos.y as f64, d1, random.next_float() * 360.0, 0.0);
                        if mob.check_spawn_rules(level, EntitySpawnReason::ChunkGeneration)
                            && mob.check_spawn_obstruction_full(level)
                        {
                            let pos = mob.living.entity.block_position();
                            let difficulty = level.current_difficulty_at(pos);
                            spawn_group_data = mob.finalize_spawn(
                                level,
                                difficulty as i32 as f32,
                                EntitySpawnReason::ChunkGeneration,
                                spawn_group_data,
                            );
                            level.add_fresh_entity(mob.living.entity.clone());
                            flag = true;
                        }
                    }
                }
                x += random.next_int_bound(5) - random.next_int_bound(5);
                loop {
                    z += random.next_int_bound(5) - random.next_int_bound(5);
                    if x >= min_block_x && x < min_block_x + 16 && z >= min_block_z && z < min_block_z + 16 {
                        break;
                    }
                    x = x0 + random.next_int_bound(5) - random.next_int_bound(5);
                    z = z0 + random.next_int_bound(5) - random.next_int_bound(5);
                    if x >= min_block_x && x < min_block_x + 16 && z >= min_block_z && z < min_block_z + 16 {
                        break;
                    }
                }
            }
        }
    }
}

/// `debug spawnCategoryForPosition` — всегда-true filter,
/// no-op callback (vanilla `@VisibleForDebug` вариант).
pub fn spawn_category_for_position_debug(
    level: &dyn LevelReader,
    chunk: &dyn SpawnChunkAccess,
    pos: BlockPos,
    random: &mut dyn RandomSource,
) -> i32 {
    spawn_category_for_position(
        MobCategory::Creature,
        level,
        chunk,
        pos,
        None,
        i32::MAX,
        None,
        random,
    )
}
