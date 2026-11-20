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
