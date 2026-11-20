//! Перенос `net.minecraft.world.level.BaseSpawner` (+ `Spawner`,
//! `CustomSpawner`) — логика моб-спавнера (блок/TrialSpawner).
//!
//! Адаптации:
//! - `ValueInput`/`ValueOutput` → `CompoundTag` (`save`/`load`).
//! - Bukkit/Paper/Leaf/Purpur события и конфиги заменены
//!   `LevelReader`-хуками с ванильной семантикой.
//! - `EntityType.loadEntityRecursive` + Mob-специфичные шаги
//!   (`checkSpawnRules`, `checkSpawnObstruction`, `finalizeSpawn`,
//!   `equip`, `spawnAnim`, `tryAddFreshEntityWithPassengers`)
//!   делегированы хукам — ServerLevel реализует их с реальным
//!   entity-storage (Entity/LivingEntity/Mob пока разделены).
//! - `tickDelay` (Paper mobSpawner tick rate) сохранён; Paper-дефолт
//!   `mobSpawner = 1` тик.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::util::random::weighted_list::WeightedList;
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type::EntityType;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::spawn_data::SpawnData;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;

/// `EVENT_SPAWN`.
pub const EVENT_SPAWN: i32 = 1;
/// `DEFAULT_SPAWN_DELAY`.
pub const DEFAULT_SPAWN_DELAY: i16 = 20;
/// `DEFAULT_MIN_SPAWN_DELAY`.
pub const DEFAULT_MIN_SPAWN_DELAY: i32 = 200;
/// `DEFAULT_MAX_SPAWN_DELAY`.
pub const DEFAULT_MAX_SPAWN_DELAY: i32 = 800;
/// `DEFAULT_SPAWN_COUNT`.
pub const DEFAULT_SPAWN_COUNT: i32 = 4;
/// `DEFAULT_MAX_NEARBY_ENTITIES`.
pub const DEFAULT_MAX_NEARBY_ENTITIES: i32 = 6;
/// `DEFAULT_REQUIRED_PLAYER_RANGE`.
pub const DEFAULT_REQUIRED_PLAYER_RANGE: i32 = 16;
/// `DEFAULT_SPAWN_RANGE`.
pub const DEFAULT_SPAWN_RANGE: i32 = 4;
/// `LevelEvent.PARTICLES_MOBBLOCK_SPAWN`.
pub const PARTICLES_MOBBLOCK_SPAWN: i32 = 2004;
/// Paper-дефолт `tickRates.mobSpawner`.
pub const DEFAULT_SPAWNER_TICK_RATE: i32 = 1;

/// `Spawner` — интерфейс источника спавна (блок-спавнер шлёт
/// `broadcastEvent` — level event 1 → клиентская анимация).
pub trait Spawner {
    /// `broadcastEvent(level, pos, eventId)`.
    fn broadcast_event(&self, level: &dyn LevelReader, pos: BlockPos, event_id: i32);
}

/// `CustomSpawner` — интерфейс кастомного источника спавна
/// (`Spawner`/`TrialSpawner`-блоки).
pub trait CustomSpawner {
    /// `tick(level, pos, spawning)` — тик; возвращает оставшееся
    /// время до следующего спавна (`i32::MAX` = никогда / не активно).
    fn tick(&mut self, level: &dyn LevelReader, pos: BlockPos, spawning: bool) -> i32;
}

/// `BaseSpawner` — данные и логика моб-спавнера.
#[derive(Clone)]
pub struct BaseSpawner {
    /// `spawnDelay`.
    pub spawn_delay: i32,
    /// `spawnPotentials`.
    pub spawn_potentials: WeightedList<SpawnData>,
    /// `nextSpawnData`.
    pub next_spawn_data: Option<SpawnData>,
    /// `spin` / `oSpin` — анимация рендера.
    pub spin: f64,
    /// `oSpin`.
    pub o_spin: f64,
    /// `minSpawnDelay`.
    pub min_spawn_delay: i32,
    /// `maxSpawnDelay`.
    pub max_spawn_delay: i32,
    /// `spawnCount`.
    pub spawn_count: i32,
    /// `maxNearbyEntities`.
    pub max_nearby_entities: i32,
    /// `requiredPlayerRange`.
    pub required_player_range: i32,
    /// `spawnRange`.
    pub spawn_range: i32,
    /// `tickDelay` — Paper настраиваемый тик-рейт спавнера.
    pub tick_delay: i32,
}

impl Default for BaseSpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseSpawner {
    /// `new BaseSpawner()`.
    pub fn new() -> Self {
        Self {
            spawn_delay: DEFAULT_SPAWN_DELAY as i32,
            spawn_potentials: WeightedList::of(SpawnData::new()),
            next_spawn_data: None,
            spin: 0.0,
            o_spin: 0.0,
            min_spawn_delay: DEFAULT_MIN_SPAWN_DELAY,
            max_spawn_delay: DEFAULT_MAX_SPAWN_DELAY,
            spawn_count: DEFAULT_SPAWN_COUNT,
            max_nearby_entities: DEFAULT_MAX_NEARBY_ENTITIES,
            required_player_range: DEFAULT_REQUIRED_PLAYER_RANGE,
            spawn_range: DEFAULT_SPAWN_RANGE,
            tick_delay: 0,
        }
    }

    /// `setEntityId(type, level, random, pos)` — записать id в
    /// nextSpawnData и очистить spawnPotentials (SPIGOT-3496).
    pub fn set_entity_id(
        &mut self,
        entity_type: &EntityType,
        level: Option<&dyn LevelReader>,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) {
        let data = self.get_or_create_next_spawn_data(level, random, pos);
        data.entity_to_spawn
            .put("id", Tag::String(entity_type.get_id().to_string()));
        self.spawn_potentials = WeightedList::of(SpawnData::new());
    }

    /// `isNearPlayer(level, pos)` — игрок в requiredPlayerRange.
    /// (Purpur redstone-deactivate / Leaf config — через хуки.)
    pub fn is_near_player(&self, level: &dyn LevelReader, pos: BlockPos) -> bool {
        if level.has_neighbor_signal(pos) {
            return false;
        }
        level.spawner_near_player(
            Vec3::new(pos.x as f64 + 0.5, pos.y as f64 + 0.5, pos.z as f64 + 0.5),
            self.required_player_range as f64,
        )
    }

    /// `clientTick(level, pos)` — клиентская анимака + спин.
    pub fn client_tick(&mut self, level: &dyn LevelReader, pos: BlockPos) {
        if !self.is_near_player(level, pos) {
            self.o_spin = self.spin;
            return;
        }
        if self.display_entity_exists(level) {
            let d = pos.x as f64 + level.level_random_next_double();
            let d1 = pos.y as f64 + level.level_random_next_double();
            let d2 = pos.z as f64 + level.level_random_next_double();
            level.add_spawner_particles(d, d1, d2);
            if self.spawn_delay > 0 {
                self.spawn_delay -= 1;
            }
            self.o_spin = self.spin;
            self.spin = (self.spin + 1000.0 / (self.spawn_delay + 200) as f64) % 360.0;
        }
    }

    /// Есть ли displayEntity — через level-хук (клиентский рендер).
    fn display_entity_exists(&self, _level: &dyn LevelReader) -> bool {
        // Java: `this.displayEntity != null`. У нас display entity
        // хранится на стороне level (spawn hook); упрощённо —
        // считаем существующим, если nextSpawnData имеет id.
        self.next_spawn_data
            .as_ref()
            .and_then(|d| d.entity_id())
            .is_some()
    }

    /// `maxAllowedLight(entityType)` — Leaf-логика ограничения
    /// освещения (friendly → 15, spider/cave_spider/enderman → 7,
    /// остальные hostile → 0).
    pub fn max_allowed_light(entity_type: &EntityType) -> i32 {
        if entity_type.get_category().is_friendly() {
            15
        } else if entity_type.get_id() == &crate::resources::identifier::Identifier::minecraft("spider")
            || entity_type.get_id() == &crate::resources::identifier::Identifier::minecraft("cave_spider")
        {
            7
        } else if entity_type.get_id() == &crate::resources::identifier::Identifier::minecraft("enderman") {
            7
        } else {
            0
        }
    }

    /// `serverTick(level, pos)` — серверный тик спавна.
    /// `broadcast_event` передаётся вызывающим (Spawner-impl).
    pub fn server_tick(
        &mut self,
        level: &dyn LevelReader,
        pos: BlockPos,
        broadcast: &dyn Fn(i32),
    ) {
        if self.spawn_count <= 0 || self.max_nearby_entities <= 0 {
            return;
        }
        if self.spawn_delay > 0 {
            self.tick_delay -= level.spawner_tick_rate();
            if self.tick_delay > 0 {
                return;
            }
            self.tick_delay = level.spawner_tick_rate();
            if self.tick_delay == -1 {
                return;
            }
        } else {
            self.tick_delay = level.spawner_tick_rate();
        }
        // Paper: tickDelay==-1 → disabled; также применяется при
        // spawnDelay<=0 ветке (tick rate каждый тик).
        let tick_rate = self.tick_delay.max(1);

        if !self.is_near_player(level, pos) || !level.spawner_block_enabled() {
            return;
        }
        if self.spawn_delay < -tick_rate {
            self.delay(level, pos, broadcast);
        }
        if self.spawn_delay > 0 {
            self.spawn_delay -= tick_rate;
            return;
        }

        let mut flag = false;
        let mut rng = LevelRandom(level);
        let next_data = self.get_or_create_next_spawn_data(None, &mut rng, pos).clone();

        for _ in 0..self.spawn_count {
            // `EntityType.by(input)` — резолв типа из "id".
            let Some(entity_type) = next_data
                .entity_id()
                .and_then(|id| crate::world::entity::entity_type::entity_type_by_identifier(&id))
            else {
                self.delay(level, pos, broadcast);
                return;
            };

            // `Pos` из NBT либо случайная позиция в spawnRange.
            let vec3 = spawn_pos_from_tag(&next_data.entity_to_spawn).unwrap_or_else(|| {
                Vec3::new(
                    pos.x as f64 + (rng.next_double() - rng.next_double()) * self.spawn_range as f64 + 0.5,
                    pos.y as f64 + rng.next_int_bound(3) as f64 - 1.0,
                    pos.z as f64 + (rng.next_double() - rng.next_double()) * self.spawn_range as f64 + 0.5,
                )
            });
            let block_pos = BlockPos::containing(vec3.x, vec3.y, vec3.z);

            // `level.noCollision(type.getSpawnAABB(...))`.
            let spawn_aabb = entity_type
                .get_dimensions()
                .make_bounding_box(&vec3);
            if !level.no_collision(&spawn_aabb) {
                continue;
            }

            // Leaf light-level check (всегда применяем — ванильная
            // альтернатива покрывается spawn rules; здесь — прямой
            // порог как в Leaf при включённой опции).
            let light = level.get_max_local_raw_brightness(block_pos);
            if light > Self::max_allowed_light(&entity_type) {
                continue;
            }
            // Leaf water check.
            if level.is_water_at(block_pos) {
                continue;
            }

            // CustomSpawnRules либо SpawnPlacements.checkSpawnRules.
            let rules_ok = match &next_data.custom_spawn_rules {
                Some(rules) => {
                    if !entity_type.get_category().is_friendly()
                        && level.difficulty() == crate::world::difficulty::Difficulty::Peaceful
                    {
                        continue;
                    }
                    rules.is_valid_position(block_pos, level)
                }
                None => level.spawn_rules_valid(&entity_type, block_pos),
            };
            if !rules_ok {
                continue;
            }

            // `EntityType.loadEntityRecursive` + snapTo — через hook
            // (entity-storage реализует ServerLevel).
            let mut entity = match level.spawn_entity_from_nbt(&next_data.entity_to_spawn, vec3) {
                Some(e) => e,
                None => {
                    self.delay(level, pos, broadcast);
                    return;
                }
            };

            // maxNearbyEntities — точный класс сущности в AABB.
            let count_bb = Aabb::new(
                pos.x as f64,
                pos.y as f64,
                pos.z as f64,
                pos.x as f64 + 1.0,
                pos.y as f64 + 1.0,
                pos.z as f64 + 1.0,
            )
            .inflate_all(self.spawn_range as f64);
            let nearby = level.count_nearby_entities_in_box(&entity_type.get_id(), &count_bb);
            if nearby >= self.max_nearby_entities {
                self.delay(level, pos, broadcast);
                return;
            }

            entity.set_rot(rng.next_float() * 360.0, 0.0);

            // Mob-специфичные проверки — хуки (no-op для не-мобов).
            if !level.mob_check_spawn_rules(&entity) || !level.mob_check_spawn_obstruction(&entity) {
                continue;
            }

            // `flag1` — тег из одного "id" → finalizeSpawn.
            let tag_is_single_id = next_data.entity_to_spawn.len() == 1
                && next_data.entity_to_spawn.get_string("id").is_some();
            if tag_is_single_id {
                level.finalize_spawner_spawn(
                    &mut entity,
                    level.current_difficulty_at(block_pos),
                );
            }
            if let Some(eq) = &next_data.equipment {
                level.equip_spawner_entity(&mut entity, eq);
            }

            let entity_uuid = entity.uuid;
            flag = true;
            if !level.try_add_fresh_entity_with_passengers(entity) {
                self.delay(level, pos, broadcast);
                return;
            }

            level.level_event(PARTICLES_MOBBLOCK_SPAWN, pos, 0);
            level.game_event(
                &crate::world::level::gameevent::game_event::entity_place(),
                block_pos,
            );
            level.mob_spawn_anim(entity_uuid);
        }

        if flag {
            self.delay(level, pos, broadcast);
        }
    }

    /// `delay(level, pos)` — перевыбор nextSpawnData + задержка.
    pub fn delay(
        &mut self,
        level: &dyn LevelReader,
        pos: BlockPos,
        broadcast: &dyn Fn(i32),
    ) {
        let mut rng = LevelRandom(level);
        let random = &mut rng;
        self.spawn_delay = if self.max_spawn_delay <= self.min_spawn_delay {
            self.min_spawn_delay
        } else {
            self.min_spawn_delay
                + random.next_int_bound(self.max_spawn_delay - self.min_spawn_delay)
        };
        if let Some(data) = self.spawn_potentials.get_random(random) {
            let data = data.clone();
            self.set_next_spawn_data(Some(level), pos, data);
        }
        broadcast(EVENT_SPAWN);
    }

    /// `load(level, pos, input)` — NBT-загрузка (ValueInput → CompoundTag).
    pub fn load(
        &mut self,
        _level: Option<&dyn LevelReader>,
        _pos: BlockPos,
        input: &CompoundTag,
    ) {
        self.spawn_delay = input
            .get_int("Paper.Delay")
            .or_else(|| input.get_short("Delay").map(|s| s as i32))
            .unwrap_or(DEFAULT_SPAWN_DELAY as i32);
        if let Some(Tag::Compound(data)) = input.get("SpawnData") {
            if let Some(spawn_data) = SpawnData::load(data) {
                self.set_next_spawn_data(None, BlockPos::ZERO, spawn_data);
            }
        }
        if let Some(Tag::List(list)) = input.get("SpawnPotentials") {
            let potentials = crate::world::level::spawn_data::load_spawn_potentials(list);
            self.spawn_potentials = if potentials.is_empty() {
                WeightedList::of(
                    self.next_spawn_data
                        .clone()
                        .unwrap_or_else(SpawnData::new),
                )
            } else {
                potentials
            };
        } else {
            self.spawn_potentials = WeightedList::of(
                self.next_spawn_data
                    .clone()
                    .unwrap_or_else(SpawnData::new),
            );
        }
        self.min_spawn_delay = input
            .get_int("Paper.MinSpawnDelay")
            .or_else(|| input.get_int("MinSpawnDelay"))
            .unwrap_or(DEFAULT_MIN_SPAWN_DELAY);
        self.max_spawn_delay = input
            .get_int("Paper.MaxSpawnDelay")
            .or_else(|| input.get_int("MaxSpawnDelay"))
            .unwrap_or(DEFAULT_MAX_SPAWN_DELAY);
        self.spawn_count = input.get_int("SpawnCount").unwrap_or(DEFAULT_SPAWN_COUNT);
        self.max_nearby_entities = input
            .get_int("MaxNearbyEntities")
            .unwrap_or(DEFAULT_MAX_NEARBY_ENTITIES);
        self.required_player_range = input
            .get_int("RequiredPlayerRange")
            .unwrap_or(DEFAULT_REQUIRED_PLAYER_RANGE);
        self.spawn_range = input.get_int("SpawnRange").unwrap_or(DEFAULT_SPAWN_RANGE);
    }

    /// `save(output)` — NBT-сохранение.
    pub fn save(&self, output: &mut CompoundTag) {
        if self.spawn_delay > i16::MAX as i32 {
            output.put("Paper.Delay", Tag::Int(self.spawn_delay));
        }
        output.put("Delay", Tag::Short(self.spawn_delay.min(i16::MAX as i32) as i16));
        if self.min_spawn_delay > i16::MAX as i32 || self.max_spawn_delay > i16::MAX as i32 {
            output.put("Paper.MinSpawnDelay", Tag::Int(self.min_spawn_delay));
            output.put("Paper.MaxSpawnDelay", Tag::Int(self.max_spawn_delay));
        }
        output.put("MinSpawnDelay", Tag::Short(self.min_spawn_delay.min(i16::MAX as i32) as i16));
        output.put("MaxSpawnDelay", Tag::Short(self.max_spawn_delay.min(i16::MAX as i32) as i16));
        output.put("SpawnCount", Tag::Short(self.spawn_count.min(i16::MAX as i32) as i16));
        output.put("MaxNearbyEntities", Tag::Short(self.max_nearby_entities.min(i16::MAX as i32) as i16));
        output.put("RequiredPlayerRange", Tag::Short(self.required_player_range.min(i16::MAX as i32) as i16));
        output.put("SpawnRange", Tag::Short(self.spawn_range.min(i16::MAX as i32) as i16));
        if let Some(data) = &self.next_spawn_data {
            output.put("SpawnData", Tag::Compound(data.save()));
        }
        output.put(
            "SpawnPotentials",
            Tag::List(crate::world::level::spawn_data::save_spawn_potentials(&self.spawn_potentials)),
        );
    }

    /// `getOrCreateDisplayEntity(level, pos)` — display entity для
    /// рендера спавнера (через hook `spawn_entity_from_nbt`).
    pub fn get_or_create_display_entity(
        &mut self,
        level: &dyn LevelReader,
        pos: BlockPos,
    ) -> Option<Entity> {
        // Java хранит displayEntity; у нас — пересоздаём по тегу
        // (дешёвый путь; кэширование — на стороне block entity).
        let mut rng = LevelRandom(level);
        let data = self.get_or_create_next_spawn_data(None, &mut rng, pos).clone();
        if data.entity_to_spawn.get_string("id").is_none() {
            return None;
        }
        let vec3 = Vec3::new(pos.x as f64 + 0.5, pos.y as f64, pos.z as f64 + 0.5);
        level.spawn_entity_from_nbt(&data.entity_to_spawn, vec3)
    }

    /// `onEventTriggered(level, id)` — level event от блока.
    pub fn on_event_triggered(&mut self, _level: &dyn LevelReader, id: i32) -> bool {
        if id == EVENT_SPAWN {
            // Клиентская ветка — spawnDelay = minSpawnDelay
            // (вызывающая сторона различает client/server).
            self.spawn_delay = self.min_spawn_delay;
            true
        } else {
            false
        }
    }

    /// `setNextSpawnData(level, pos, nextSpawnData)`.
    pub fn set_next_spawn_data(
        &mut self,
        _level: Option<&dyn LevelReader>,
        _pos: BlockPos,
        next_spawn_data: SpawnData,
    ) {
        self.next_spawn_data = Some(next_spawn_data);
    }

    /// `getOrCreateNextSpawnData(level, random, pos)`.
    fn get_or_create_next_spawn_data(
        &mut self,
        level: Option<&dyn LevelReader>,
        random: &mut dyn RandomSource,
        pos: BlockPos,
    ) -> &mut SpawnData {
        if self.next_spawn_data.is_none() {
            let data = self
                .spawn_potentials
                .get_random(random)
                .cloned()
                .unwrap_or_else(SpawnData::new);
            self.set_next_spawn_data(level, pos, data);
        }
        self.next_spawn_data.as_mut().unwrap()
    }

    /// `getSpin()` / `getOSpin()`.
    pub fn get_spin(&self) -> f64 {
        self.spin
    }

    /// `getOSpin()`.
    pub fn get_o_spin(&self) -> f64 {
        self.o_spin
    }
}

/// `Pos` из entityToSpawn-тега (`ListTag` из 3 double —
/// `Vec3.CODEC` в NBT).
fn spawn_pos_from_tag(tag: &CompoundTag) -> Option<Vec3> {
    if let Some(Tag::List(list)) = tag.get("Pos") {
        if list.len() == 3 {
            let x = list[0].as_double()?;
            let y = list[1].as_double()?;
            let z = list[2].as_double()?;
            return Some(Vec3::new(x, y, z));
        }
    }
    None
}

/// `LevelRandom` — адаптер `LevelReader`-хуков под `RandomSource`
/// (нужен `WeightedList::get_random` и `next_int_bound`).
pub(crate) struct LevelRandom<'a>(pub(crate) &'a dyn LevelReader);

impl<'a> LevelRandom<'a> {
    /// `new LevelRandom(level)`.
    pub(crate) fn new(level: &'a dyn LevelReader) -> Self {
        Self(level)
    }
}

impl RandomSource for LevelRandom<'_> {
    fn set_seed(&mut self, _seed: i64) {}
    fn fork(&mut self) -> Box<dyn RandomSource> {
        crate::random::legacy::create_unique()
    }
    fn fork_positional(&mut self) -> Box<dyn crate::random::PositionalRandomFactory> {
        unimplemented!("LevelRandom::fork_positional")
    }
    fn next_int(&mut self) -> i32 {
        // 32-битное значение через два bounded-значения.
        (self.0.level_random_next_int(i32::MAX) as u32 as i64
            | ((self.0.level_random_next_int(i32::MAX) as u32 as i64) << 32)) as i32
    }
    fn next_int_bound(&mut self, bound: i32) -> i32 {
        self.0.level_random_next_int(bound)
    }
    fn next_long(&mut self) -> i64 {
        (((self.0.level_random_next_int(i32::MAX) as u64) << 32)
            | self.0.level_random_next_int(i32::MAX) as u64) as i64
    }
    fn next_boolean(&mut self) -> bool {
        self.0.level_random_next_int(2) == 0
    }
    fn next_float(&mut self) -> f32 {
        self.0.level_random_next_float()
    }
    fn next_double(&mut self) -> f64 {
        self.0.level_random_next_double() as f64
    }
    fn next_gaussian(&mut self) -> f64 {
        crate::random::bit::next_gaussian(&mut crate::random::bit::Gaussian::default(), || {
            self.next_double()
        })
    }
}
