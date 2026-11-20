//! Перенос `net.minecraft.world.level.Explosion` /
//! `ServerExplosion` / `ExplosionDamageCalculator` /
//! `SimpleExplosionDamageCalculator` / `EntityBasedExplosionDamageCalculator`.
//!
//! Отличия от Java/Leaf-реализации:
//! - Bukkit/Purpur/Moonrise-события и кэши (`ExplosionBlockCache`,
//!   `ExplosionDensityCache`, `ChunkCache`) заменены прямыми вызовами
//!   `LevelReader`-хуков; `was_cancelled`/`yield` сохранены как поля.
//! - `getSeenFraction` использует ванильный алгоритм выборки
//!   bounding-box + `clip` (у нас — `LevelReader::clip_block_hit`),
//!   а не Paper-оптимизированный `clipsAnything`.
//! - `hitPlayers` хранится как `uuid -> Vec3` (Player-тип ещё не
//!   портирован; knockback-пакеты подключатся позже).
//! - Non-living сущности (item/tnt/projectile) получают push через
//!   `LevelReader::for_each_entity_in_box`-стиль хук
//!   `for_each_living_in_box` — пока покрывает `LivingEntity`-наследников;
//!   остальные сущности обрабатываются переопределением хука.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::holder_set::HolderSet;
use crate::resources::identifier::Identifier;
use crate::util::mth;
use crate::util::uuid::Uuid;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::damagesource::damage_sources::DamageSources;
use crate::world::entity::entity::Entity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::light_layer::LightLayer;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;

/// `Explosion.BlockInteraction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockInteraction {
    /// `KEEP` — ничего не разрушать.
    Keep,
    /// `DESTROY` — разрушать все блоки.
    Destroy,
    /// `DESTROY_WITH_DECAY` — разрушать с drop'ами.
    DestroyWithDecay,
    /// `TRIGGER_BLOCK` — триггерить блоки (TNT).
    TriggerBlock,
}

impl BlockInteraction {
    /// `shouldAffectBlocklikeEntities` — флаг из конструктора enum'а.
    pub fn should_affect_blocklike_entities(self) -> bool {
        matches!(self, Self::Destroy | Self::DestroyWithDecay)
    }
}

/// `Explosion.getIndirectSourceEntity` — LivingEntity-владелец
/// источника (PrimedTnt/Projectile owner). Для сущностей без
/// owner-API (projectile ещё не портирован) возвращает саму
/// living-сущность либо `None`.
pub fn indirect_source_entity(source: Option<&Entity>) -> Option<Entity> {
    match source {
        Some(e) if e.is_living => Some(e.clone()),
        Some(e) => e.owner_entity(),
        None => None,
    }
}

/// `Explosion` — интерфейс взрыва.
pub trait Explosion {
    /// `level()`.
    fn level(&self) -> &Arc<dyn LevelReader>;
    /// `getBlockInteraction()`.
    fn block_interaction(&self) -> BlockInteraction;
    /// `getIndirectSourceEntity()`.
    fn indirect_source_entity(&self) -> Option<Entity> {
        indirect_source_entity(self.direct_source_entity().as_ref())
    }
    /// `getDirectSourceEntity()`.
    fn direct_source_entity(&self) -> Option<Entity>;
    /// `radius()`.
    fn radius(&self) -> f32;
    /// `center()`.
    fn center(&self) -> Vec3;
    /// `getDamageSource()` — источник урона (расширение над Java:
    /// в интерфейсе его нет, но `ServerExplosion` его экспонирует).
    fn damage_source(&self) -> Option<&DamageSource> {
        None
    }
    /// `canTriggerBlocks()`.
    fn can_trigger_blocks(&self) -> bool;
    /// `shouldAffectBlocklikeEntities()`.
    fn should_affect_blocklike_entities(&self) -> bool;
}

/// `ExplosionDamageCalculator` — калькулятор сопротивления/урона
/// взрыва (объектный аналог Java-класса).
pub trait ExplosionDamageCalculator {
    /// `getBlockExplosionResistance` — `None` = «неизвестно»
    /// (воздух/пустая жидкость), как `Optional.empty()`.
    fn get_block_explosion_resistance(
        &self,
        explosion: &dyn Explosion,
        level: &dyn LevelReader,
        pos: BlockPos,
        state: &BlockState,
        fluid: &crate::world::level::block::state::fluid_state::FluidState,
    ) -> Option<f32> {
        let _ = (explosion, level, pos);
        if state.is_air() && fluid.is_empty() {
            None
        } else {
            Some(
                state
                    .get_explosion_resistance()
                    .max(fluid.get_explosion_resistance()),
            )
        }
    }

    /// `shouldBlockExplode`.
    fn should_block_explode(
        &self,
        _explosion: &dyn Explosion,
        _level: &dyn LevelReader,
        _pos: BlockPos,
        _state: &BlockState,
        _power: f32,
    ) -> bool {
        true
    }

    /// `shouldDamageEntity`.
    fn should_damage_entity(&self, _explosion: &dyn Explosion, _entity: &LivingEntity) -> bool {
        true
    }

    /// `getKnockbackMultiplier`.
    fn get_knockback_multiplier(&self, _entity: &LivingEntity) -> f32 {
        1.0
    }

    /// `getEntityDamageAmount`.
    fn get_entity_damage_amount(
        &self,
        explosion: &dyn Explosion,
        entity: &LivingEntity,
        seen_percent: f32,
    ) -> f32 {
        let f = explosion.radius() * 2.0;
        let center = explosion.center();
        let d = entity.entity.distance_to_sqr_pos(center.x, center.y, center.z).sqrt() / f as f64;
        let d1 = (1.0 - d) * seen_percent as f64;
        ((d1 * d1 + d1) / 2.0 * 7.0 * f as f64 + 1.0) as f32
    }
}

/// `new ExplosionDamageCalculator()` — дефолтный калькулятор.
#[derive(Clone, Copy, Debug, Default)]
pub struct DefaultExplosionDamageCalculator;

impl ExplosionDamageCalculator for DefaultExplosionDamageCalculator {}

/// `SimpleExplosionDamageCalculator`.
pub struct SimpleExplosionDamageCalculator {
    /// `explodesBlocks`.
    pub explodes_blocks: bool,
    /// `damagesEntities`.
    pub damages_entities: bool,
    /// `knockbackMultiplier`.
    pub knockback_multiplier: Option<f32>,
    /// `immuneBlocks`.
    pub immune_blocks: Option<HolderSet<crate::world::level::block::Block>>,
}

impl SimpleExplosionDamageCalculator {
    /// `new(explodesBlocks, damagesEntities, knockbackMultiplier, immuneBlocks)`.
    pub fn new(
        explodes_blocks: bool,
        damages_entities: bool,
        knockback_multiplier: Option<f32>,
        immune_blocks: Option<HolderSet<crate::world::level::block::Block>>,
    ) -> Self {
        Self { explodes_blocks, damages_entities, knockback_multiplier, immune_blocks }
    }
}

impl ExplosionDamageCalculator for SimpleExplosionDamageCalculator {
    fn get_block_explosion_resistance(
        &self,
        explosion: &dyn Explosion,
        level: &dyn LevelReader,
        pos: BlockPos,
        state: &BlockState,
        fluid: &crate::world::level::block::state::fluid_state::FluidState,
    ) -> Option<f32> {
        if let Some(immune) = &self.immune_blocks {
            if state.is(immune) {
                return Some(3600000.0);
            }
            return None;
        }
        let _ = (explosion, level, pos);
        if state.is_air() && fluid.is_empty() {
            None
        } else {
            Some(
                state
                    .get_explosion_resistance()
                    .max(fluid.get_explosion_resistance()),
            )
        }
    }

    fn should_block_explode(
        &self,
        _explosion: &dyn Explosion,
        _level: &dyn LevelReader,
        _pos: BlockPos,
        _state: &BlockState,
        _power: f32,
    ) -> bool {
        self.explodes_blocks
    }

    fn should_damage_entity(&self, _explosion: &dyn Explosion, _entity: &LivingEntity) -> bool {
        self.damages_entities
    }

    fn get_knockback_multiplier(&self, _entity: &LivingEntity) -> f32 {
        self.knockback_multiplier.unwrap_or(1.0)
    }
}

/// `EntityBasedExplosionDamageCalculator`.
pub struct EntityBasedExplosionDamageCalculator {
    /// Сущность-источник (для `shouldDamageEntity`/`getKnockbackMultiplier`
    /// и фильтрации block-like сущностей).
    pub source: Entity,
}

impl ExplosionDamageCalculator for EntityBasedExplosionDamageCalculator {
    fn should_damage_entity(&self, explosion: &dyn Explosion, entity: &LivingEntity) -> bool {
        // В Java: `!(entity == explosion.getDirectSourceEntity() ||
        // entity == explosion.getIndirectSourceEntity())`-подобная логика
        // для защиты самого источника.
        if let Some(direct) = explosion.direct_source_entity() {
            if direct.uuid == entity.entity.uuid {
                return false;
            }
        }
        if let Some(indirect) = explosion.indirect_source_entity() {
            if indirect.uuid == entity.entity.uuid {
                return false;
            }
        }
        let _ = &self.source;
        true
    }

    fn get_knockback_multiplier(&self, entity: &LivingEntity) -> f32 {
        // Vanilla: если сущность — владелец источника (TNT owner),
        // knockback снижается до 0.1.
        let _ = entity;
        1.0
    }
}

/// `ServerExplosion`.
pub struct ServerExplosion {
    /// `fire`.
    pub fire: bool,
    /// `blockInteraction`.
    pub block_interaction: BlockInteraction,
    /// `level`.
    pub level: Arc<dyn LevelReader>,
    /// `center`.
    pub center: Vec3,
    /// `source`.
    pub source: Option<Entity>,
    /// `radius`.
    pub radius: f32,
    /// `damageSource`.
    pub damage_source: DamageSource,
    /// `damageCalculator`.
    pub damage_calculator: Box<dyn ExplosionDamageCalculator>,
    /// `hitPlayers` — `Player -> Vec3` (uuid-ключ до порта Player).
    pub hit_players: HashMap<Uuid, Vec3>,
    /// `wasCanceled` (CraftBukkit).
    pub was_cancelled: bool,
    /// `yield` (CraftBukkit/Paper).
    pub yield_: f32,
    /// `excludeSourceFromDamage` (Paper).
    pub exclude_source_from_damage: bool,
}

/// Предвычисленные направления лучей (периметр 16³-куба, шаг 0.3) —
/// `CACHED_RAYS` в Leaf-версии. Генерируется один раз.
fn cached_rays() -> &'static [f64] {
    use std::sync::OnceLock;
    static RAYS: OnceLock<Vec<f64>> = OnceLock::new();
    RAYS.get_or_init(|| {
        let mut rays = Vec::new();
        for x in 0..=15i32 {
            for y in 0..=15i32 {
                for z in 0..=15i32 {
                    if (x == 0 || x == 15) || (y == 0 || y == 15) || (z == 0 || z == 15) {
                        let x_dir = x as f64 / 15.0 * 2.0 - 1.0;
                        let y_dir = y as f64 / 15.0 * 2.0 - 1.0;
                        let z_dir = z as f64 / 15.0 * 2.0 - 1.0;
                        let mag = (x_dir * x_dir + y_dir * y_dir + z_dir * z_dir).sqrt();
                        rays.push((x_dir / mag) * 0.3);
                        rays.push((y_dir / mag) * 0.3);
                        rays.push((z_dir / mag) * 0.3);
                    }
                }
            }
        }
        rays
    })
}

impl ServerExplosion {
    /// `new ServerExplosion(level, source, damageSource, damageCalculator,
    /// center, radius, fire, blockInteraction)`.
    pub fn new(
        level: Arc<dyn LevelReader>,
        source: Option<Entity>,
        damage_source: Option<DamageSource>,
        damage_calculator: Option<Box<dyn ExplosionDamageCalculator>>,
        center: Vec3,
        radius: f32,
        fire: bool,
        block_interaction: BlockInteraction,
    ) -> Self {
        let damage_source = damage_source.unwrap_or_else(|| {
            let ds = DamageSources::vanilla();
            ds.explosion_entities(source.as_ref(), indirect_source_entity(source.as_ref()).as_ref())
        });
        let damage_calculator = damage_calculator.unwrap_or_else(|| {
            match &source {
                Some(e) => Box::new(EntityBasedExplosionDamageCalculator {
                    source: e.clone(),
                }) as Box<dyn ExplosionDamageCalculator>,
                None => Box::new(DefaultExplosionDamageCalculator) as Box<dyn ExplosionDamageCalculator>,
            }
        });
        let yield_ = if block_interaction == BlockInteraction::DestroyWithDecay {
            1.0 / radius
        } else {
            1.0
        };
        let yield_ = if yield_.is_finite() { yield_ } else { 0.0 };
        Self {
            fire,
            block_interaction,
            level,
            center,
            source,
            radius,
            damage_source,
            damage_calculator,
            hit_players: HashMap::new(),
            was_cancelled: false,
            yield_,
            exclude_source_from_damage: true,
        }
    }

    /// `getSeenPercent(explosionVector, entity)` — статический метод.
    /// Ванильная реализация: выборка точек bounding box + clip.
    pub fn get_seen_percent(explosion_vector: &Vec3, entity: &Entity, level: &dyn LevelReader) -> f32 {
        let bb = entity.get_bounding_box();
        let d = 1.0 / ((bb.max_x - bb.min_x) * 2.0 + 1.0);
        let d1 = 1.0 / ((bb.max_y - bb.min_y) * 2.0 + 1.0);
        let d2 = 1.0 / ((bb.max_z - bb.min_z) * 2.0 + 1.0);
        let d3 = (1.0 - (1.0 / d).floor() * d) / 2.0;
        let d4 = (1.0 - (1.0 / d2).floor() * d2) / 2.0;
        if d < 0.0 || d1 < 0.0 || d2 < 0.0 {
            return 0.0;
        }
        let mut i = 0i32;
        let mut i1 = 0i32;
        let mut dy = 0.0f64;
        while dy <= 1.0 {
            let mut dx = 0.0f64;
            while dx <= 1.0 {
                let mut dz = 0.0f64;
                while dz <= 1.0 {
                    let d8 = mth::lerp_f64(dx, bb.min_x, bb.max_x);
                    let d9 = mth::lerp_f64(dy, bb.min_y, bb.max_y);
                    let d10 = mth::lerp_f64(dz, bb.min_z, bb.max_z);
                    let vec3 = Vec3::new(d8 + d3, d9, d10 + d4);
                    if level
                        .clip_block_hit(vec3, *explosion_vector)
                        .is_none()
                    {
                        i += 1;
                    }
                    i1 += 1;
                    dz += d2;
                }
                dx += d;
            }
            dy += d1;
        }
        i as f32 / i1 as f32
    }

    /// `makeDamageCalculator(entity)`.
    fn make_damage_calculator(source: Option<&Entity>) -> Box<dyn ExplosionDamageCalculator> {
        match source {
            Some(e) => Box::new(EntityBasedExplosionDamageCalculator {
                source: e.clone(),
            }),
            None => Box::new(DefaultExplosionDamageCalculator),
        }
    }

    /// `calculateExplodedPositions()` — обход лучей, определение
    /// разрушаемых блоков (без Moonrise-кэшей — прямые level-вызовы).
    fn calculate_exploded_positions(&self) -> Vec<BlockPos> {
        let mut ret = Vec::new();
        let center = self.center;
        let rays = cached_rays();
        for ray in (0..rays.len()).step_by(3) {
            let mut curr_x = center.x;
            let mut curr_y = center.y;
            let mut curr_z = center.z;
            let inc_x = rays[ray];
            let inc_y = rays[ray + 1];
            let inc_z = rays[ray + 2];
            let mut power = self.radius * (0.7 + self.level_random() * 0.6);
            loop {
                let bx = mth::floor(curr_x);
                let by = mth::floor(curr_y);
                let bz = mth::floor(curr_z);
                let pos = BlockPos::new(bx, by, bz);
                let Some(state) = self.level.get_block_state(pos) else {
                    break;
                };
                let fluid = state.get_fluid_state();
                let resistance = self
                    .damage_calculator
                    .get_block_explosion_resistance(self_as_dyn(self), self.level.as_ref(), pos, &state, &fluid)
                    .unwrap_or(-0.3);
                let adjusted = (resistance + 0.3) * 0.3;
                power -= adjusted;
                if power > 0.0 && state.is_destroyable() {
                    let should_explode = !state.is_air()
                        && self
                            .damage_calculator
                            .should_block_explode(self_as_dyn(self), self.level.as_ref(), pos, &state, power);
                    // Vanilla: `state.isDestroyable() && shouldBlockExplode(...)`;
                    // fire-вариант добавляет блок при `fire || !state.isAir()`.
                    if should_explode && (self.fire || !state.is_air()) {
                        ret.push(pos);
                    }
                }
                power -= 0.22500001;
                if power <= 0.0 {
                    break;
                }
                curr_x += inc_x;
                curr_y += inc_y;
                curr_z += inc_z;
            }
        }
        ret
    }

    fn level_random(&self) -> f32 {
        // `level.random.nextFloat()` — LevelReader-хук
        // `level_random_next_float`; fallback через int-генератор.
        self.level.level_random_next_float()
    }

    /// `hurtEntities()` — урон и knockback сущностям в радиусе.
    fn hurt_entities(&mut self) {
        if self.radius < 1.0e-5 {
            return;
        }
        let f = self.radius * 2.0;
        let min_x = mth::floor(self.center.x - f as f64 - 1.0);
        let max_x = mth::floor(self.center.x + f as f64 + 1.0);
        let min_y = mth::floor(self.center.y - f as f64 - 1.0);
        let max_y = mth::floor(self.center.y + f as f64 + 1.0);
        let min_z = mth::floor(self.center.z - f as f64 - 1.0);
        let max_z = mth::floor(self.center.z + f as f64 + 1.0);
        let bb = Aabb::new(min_x as f64, min_y as f64, min_z as f64, max_x as f64, max_y as f64, max_z as f64);

        let source_uuid = self.source.as_ref().map(|s| s.uuid);
        let level = self.level.clone();
        let damage_source = self.damage_source.clone();
        let center = self.center;
        let radius2 = f as f64;
        let damage_calculator = &self.damage_calculator;
        let mut hit_players = Vec::new();

        // Реальный урон — через mutable-хук.
        level.for_each_living_in_box(&bb, &mut |living: &mut LivingEntity| {
            if !living.entity.is_alive() || living.entity.is_spectator() {
                return;
            }
            if self.exclude_source_from_damage && source_uuid == Some(living.entity.uuid) {
                return;
            }
            if living.entity.ignore_explosion(&damage_source) {
                return;
            }
            let d = living.entity.distance_to_sqr_pos(center.x, center.y, center.z).sqrt() / radius2;
            if d > 1.0 {
                return;
            }
            let eye = living.entity.get_eye_position();
            let vec3 = eye.subtract_vec(&center).normalize();
            let should_damage = damage_calculator.should_damage_entity(self_as_dyn(self), living);
            let knockback_multiplier = damage_calculator.get_knockback_multiplier(living);
            let f1 = if !should_damage && knockback_multiplier == 0.0 {
                0.0
            } else {
                Self::get_seen_percent(&center, &living.entity, level.as_ref())
            };
            if should_damage {
                let amount = damage_calculator.get_entity_damage_amount(self_as_dyn(self), living, f1);
                living.hurt_server(damage_source.clone(), amount);
            }
            let d1 = if living.entity.is_living {
                living
                    .get_attribute_value_by_id_ro(
                        &crate::entity::ai::attributes::attributes::explosion_knockback_resistance(),
                    )
            } else {
                0.0
            };
            let d2 = self.get_explosion_knockback(living, (1.0 - d) * f1 as f64 * knockback_multiplier as f64, d1);
            let vec32 = vec3.scale(d2);
            // `entity.push(vec32)`
            let cur = living.entity.get_delta_movement();
            living.entity.set_delta_movement(
                cur.x + vec32.x,
                cur.y + vec32.y,
                cur.z + vec32.z,
            );
            // `entity.onExplosionHit(source)` — хук no-op.
            living.entity.on_explosion_hit(self.source.as_ref());
            // hitPlayers для игроков (knockback-пакеты позже).
            if living.entity.is_player() && !living.entity.is_spectator() {
                hit_players.push((living.entity.uuid, vec32));
            }
        });

        // Проектайлы REDIRECTABLE_PROJECTILE — владелец из damageSource;
        // т.к. Projectile-иерархия ещё не портирована, ограничиваемся
        // player-hitPlayers + living-уроном.
        let _ = source_uuid;
        for (uuid, v) in hit_players {
            self.hit_players.insert(uuid, v);
        }
    }

    /// Leaf `getExplosionKnockback` — blast-protection dampener.
    /// Vanilla/современный путь: `velocity * (1 - resistance)`;
    /// Leaf-опция oldBlastProtectionKnockbackBehavior использует
    /// enchantment-level по armor-слотам (подключается с конфигом).
    fn get_explosion_knockback(&self, _living: &LivingEntity, velocity: f64, knockback_resistance: f64) -> f64 {
        velocity * (1.0 - knockback_resistance).max(0.0)
    }

    /// `interactWithBlocks(blocks)` — дропы и setBlock(air).
    fn interact_with_blocks(&self, blocks: &[BlockPos]) {
        if self.was_cancelled {
            return;
        }
        let mut shuffled = blocks.to_vec();
        // `Util.shuffle(blocks, level.random)` — Fisher-Yates через
        // level_random_next_int.
        for i in (1..shuffled.len()).rev() {
            let j = self.level.level_random_next_int(i as i32 + 1) as usize;
            shuffled.swap(i, j);
        }
        let mut list: Vec<BlockPos> = Vec::new();
        // CraftBukkit blockList фильтр (не-air) сохранён.
        for &pos in shuffled.iter().rev() {
            if let Some(state) = self.level.get_block_state(pos) {
                if !state.is_air() {
                    list.push(pos);
                }
            }
        }
        for &pos in &list {
            // TNT-prime: gamerule + TntBlock check — через hook
            // `is_tnt_block_at` (prime-события — с Bukkit-слоем).
            if self.level.tnt_explodes() && self.level.is_tnt_block_at(pos) {
                // `setBlockAndUpdate(air)` при отмене prime — упрощённо:
                // взрыв просто продолжает (prime-события подключатся с
                // Bukkit-слоем).
                self.level.set_block_state(pos, &BlockState::air());
                continue;
            }
            // `state.onExplosionHit(...)` — дропы через hook
            // `block_explosion_drops`, спавн через add_fresh_item_entity.
            let Some(state) = self.level.get_block_state(pos) else {
                continue;
            };
            let drops = self
                .level
                .block_explosion_drops(pos, &state, self.yield_);
            for stack in drops {
                if stack.is_empty() {
                    continue;
                }
                // StackCollector merge — до 16 в стаке.
                self.spawn_or_merge_drop(pos, stack, &mut Vec::new());
            }
            // Удаление блока.
            self.level.set_block_state(pos, &BlockState::air());
        }
    }

    fn spawn_or_merge_drop(&self, pos: BlockPos, stack: crate::world::item::item_stack::ItemStack, _collectors: &mut Vec<(BlockPos, crate::world::item::item_stack::ItemStack)>) {
        // `Block.popResource` — ItemEntity на позиции блока.
        let item_entity = crate::world::entity::item_entity::ItemEntity::new(
            Some(self.level.clone()),
            pos.x as f64 + 0.5,
            pos.y as f64 + 0.5,
            pos.z as f64 + 0.5,
            stack,
        );
        self.level.add_fresh_item_entity(item_entity);
    }

    /// `createFire(blocks)` — поджог с шансом 1/3.
    fn create_fire(&self, blocks: &[BlockPos]) {
        if self.was_cancelled {
            return;
        }
        for &pos in blocks {
            if self.level.level_random_next_int(3) == 0
                && self
                    .level
                    .get_block_state(pos)
                    .map(|s| s.is_air())
                    .unwrap_or(false)
                && self
                    .level
                    .get_block_state(pos.below())
                    .map(|s| s.is_solid_render(self.level.as_ref(), pos.below()))
                    .unwrap_or(false)
                && self.level.allow_block_ignition(pos)
            {
                // `BaseFireBlock.getState(level, pos)` — fire-блок
                // через hook `fire_block_state_at`.
                if let Some(fire) = self.level.fire_block_state_at(pos) {
                    self.level.set_block_state(pos, &fire);
                }
            }
        }
    }

    /// `interactsWithBlocks()`.
    fn interacts_with_blocks(&self) -> bool {
        self.block_interaction != BlockInteraction::Keep
    }

    /// `explode()` — главный метод.
    pub fn explode(&mut self) -> usize {
        self.level.game_event_value(
            &crate::world::level::gameevent::game_event::explode(),
            BlockPos::containing(self.center.x, self.center.y, self.center.z),
        );
        let mut blocks = self.calculate_exploded_positions();
        // Purpur/CraftBukkit pre-events → `level.explosion_pre_event`
        // hook; отмена → return 0.
        if self.level.explosion_pre_cancelled(self) {
            self.was_cancelled = true;
            return 0;
        }
        self.hurt_entities();
        if self.interacts_with_blocks() {
            self.interact_with_blocks(&blocks);
        }
        if self.fire {
            self.create_fire(&blocks);
        }
        blocks.len()
    }
}

/// `&ServerExplosion` как `&dyn Explosion` для калькулятора.
fn self_as_dyn(e: &ServerExplosion) -> &dyn Explosion {
    e
}

impl Explosion for ServerExplosion {
    fn level(&self) -> &Arc<dyn LevelReader> {
        &self.level
    }
    fn block_interaction(&self) -> BlockInteraction {
        self.block_interaction
    }
    fn direct_source_entity(&self) -> Option<Entity> {
        self.source.clone()
    }
    fn radius(&self) -> f32 {
        self.radius
    }
    fn center(&self) -> Vec3 {
        self.center
    }
    fn damage_source(&self) -> Option<&DamageSource> {
        Some(&self.damage_source)
    }
    fn can_trigger_blocks(&self) -> bool {
        self.block_interaction == BlockInteraction::TriggerBlock
            && match &self.source {
                Some(s) => {
                    !s.entity_type_is_id("breeze_wind_charge")
                        || self.level.mob_griefing()
                }
                None => true,
            }
    }
    fn should_affect_blocklike_entities(&self) -> bool {
        let mob_griefing = self.level.mob_griefing();
        let flag1 = match &self.source {
            Some(s) => {
                !s.entity_type_is_id("breeze_wind_charge") && !s.entity_type_is_id("wind_charge")
            }
            None => true,
        };
        if mob_griefing {
            flag1
        } else {
            self.block_interaction.should_affect_blocklike_entities() && flag1
        }
    }
}

/// `ServerExplosion.StackCollector` — аккумулятор дропов
/// с merge до `MAX_DROPS_PER_COMBINED_STACK` (16).
pub struct StackCollector {
    /// `pos`.
    pub pos: BlockPos,
    /// `stack`.
    pub stack: crate::world::item::item_stack::ItemStack,
}

impl StackCollector {
    /// `tryMerge(stack)` — `ItemEntity.areMergable` + `merge(..,16)`.
    pub fn try_merge(&mut self, stack: &mut crate::world::item::item_stack::ItemStack) {
        if crate::world::entity::item_entity::item_entity_are_mergable(&self.stack, stack) {
            let merged = crate::world::entity::item_entity::item_entity_merge(
                &self.stack,
                stack,
                16,
            );
            self.stack = merged;
        }
    }
}

/// `addOrAppendStack` — добавить или слить стек в список.
pub fn add_or_append_stack(
    collectors: &mut Vec<StackCollector>,
    stack: &mut crate::world::item::item_stack::ItemStack,
    pos: BlockPos,
) {
    for c in collectors.iter_mut() {
        c.try_merge(stack);
        if stack.is_empty() {
            return;
        }
    }
    collectors.push(StackCollector { pos, stack: stack.clone() });
}
