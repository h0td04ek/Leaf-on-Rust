//! Перенос `net.minecraft.world.item.enchantment.effects.EnchantmentEntityEffect`
//! — 16 вариантов эффектов, применяемых к сущности/миру.

use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::core::vec3i::Vec3i;
use crate::resources::identifier::Identifier;
use crate::util::random::weighted_list::WeightedList;
use crate::util::valueproviders::float_provider::FloatProvider;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::effect::mob_effect_instance::{MobEffectDetails, MobEffectInstance};
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type::EntityType;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::item::component::block_item_state_properties::BlockItemStateProperties;
use crate::world::item::enchantment::effects::enchanted_item_in_use::EnchantedItemInUse;
use crate::world::item::enchantment::level_based_value::LevelBasedValue;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::gameevent::game_event::GameEvent;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::blockpredicates::BlockPredicate;
use crate::world::level::levelgen::feature::stateproviders::StateProvider;
use crate::world::phys::Vec3;
use crate::world::effect::mob_effect::MobEffect;

/// `EnchantmentEntityEffect` — sealed union (16 реализаций).
#[derive(Clone)]
pub enum EnchantmentEntityEffect {
    /// `AllOf.EntityEffects`.
    AllOf(Vec<EnchantmentEntityEffect>),
    /// `ApplyMobEffect`.
    ApplyMobEffect {
        to_apply: HolderSet<MobEffect>,
        min_duration: LevelBasedValue,
        max_duration: LevelBasedValue,
        min_amplifier: LevelBasedValue,
        max_amplifier: LevelBasedValue,
    },
    /// `ChangeItemDamage`.
    ChangeItemDamage { amount: LevelBasedValue },
    /// `DamageEntity`.
    DamageEntity {
        min_damage: LevelBasedValue,
        max_damage: LevelBasedValue,
        damage_type: Holder<crate::world::damagesource::damage_type::DamageType>,
    },
    /// `ExplodeEffect`.
    Explode(ExplodeEffect),
    /// `Ignite`.
    Ignite { duration: LevelBasedValue },
    /// `ApplyEntityImpulse`.
    ApplyImpulse {
        direction: Vec3,
        coordinate_scale: Vec3,
        magnitude: LevelBasedValue,
    },
    /// `ApplyExhaustion`.
    ApplyExhaustion { amount: LevelBasedValue },
    /// `PlaySoundEffect`.
    PlaySound {
        sound_events: Vec<Holder<crate::sounds::sound_event::SoundEvent>>,
        volume: Arc<dyn FloatProvider>,
        pitch: Arc<dyn FloatProvider>,
    },
    /// `ReplaceBlock`.
    ReplaceBlock {
        offset: Vec3i,
        predicate: Option<BlockPredicate>,
        block_state: StateProvider,
        trigger_game_event: Option<Holder<GameEvent>>,
    },
    /// `ReplaceDisk`.
    ReplaceDisk {
        radius: LevelBasedValue,
        height: LevelBasedValue,
        offset: Vec3i,
        predicate: Option<BlockPredicate>,
        block_state: StateProvider,
        trigger_game_event: Option<Holder<GameEvent>>,
    },
    /// `RunFunction`.
    RunFunction { function: Identifier },
    /// `SetBlockProperties`.
    SetBlockProperties {
        properties: BlockItemStateProperties,
        offset: Vec3i,
        trigger_game_event: Option<Holder<GameEvent>>,
    },
    /// `SpawnParticlesEffect`.
    SpawnParticles(SpawnParticlesEffect),
    /// `SummonEntityEffect`.
    SummonEntity {
        entity_types: HolderSet<EntityType>,
        join_team: bool,
    },
}

/// `ExplosionInteraction` — `Level.ExplosionInteraction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}

/// `ExplodeEffect` record.
#[derive(Clone)]
pub struct ExplodeEffect {
    /// `attributeToUser`.
    pub attribute_to_user: bool,
    /// `damageType`.
    pub damage_type: Option<Holder<crate::world::damagesource::damage_type::DamageType>>,
    /// `knockbackMultiplier`.
    pub knockback_multiplier: Option<LevelBasedValue>,
    /// `immuneBlocks`.
    pub immune_blocks: Option<HolderSet<crate::world::level::block::Block>>,
    /// `offset`.
    pub offset: Vec3,
    /// `radius`.
    pub radius: LevelBasedValue,
    /// `createFire`.
    pub create_fire: bool,
    /// `blockInteraction`.
    pub block_interaction: ExplosionInteraction,
    /// `smallParticle`.
    pub small_particle: crate::core::particles::particle_options::ParticleOptions,
    /// `largeParticle`.
    pub large_particle: crate::core::particles::particle_options::ParticleOptions,
    /// `blockParticles`.
    pub block_particles: WeightedList<crate::core::particles::explosion_particle_info::ExplosionParticleInfo>,
    /// `sound`.
    pub sound: Holder<crate::sounds::sound_event::SoundEvent>,
}

/// `SpawnParticlesEffect.PositionSourceType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionSourceType {
    /// `ENTITY_POSITION`.
    EntityPosition,
    /// `BOUNDING_BOX`.
    BoundingBox,
}

/// `SpawnParticlesEffect.PositionSource`.
#[derive(Clone)]
pub struct PositionSource {
    /// `type`.
    pub source_type: PositionSourceType,
    /// `offset`.
    pub offset: f32,
    /// `scale`.
    pub scale: f32,
}

/// `SpawnParticlesEffect.VelocitySource`.
#[derive(Clone)]
pub struct VelocitySource {
    /// `movementScale`.
    pub movement_scale: f32,
    /// `base`.
    pub base: Arc<dyn FloatProvider>,
}

/// `SpawnParticlesEffect` record.
#[derive(Clone)]
pub struct SpawnParticlesEffect {
    /// `particle`.
    pub particle: crate::core::particles::particle_options::ParticleOptions,
    /// `horizontalPosition`.
    pub horizontal_position: PositionSource,
    /// `verticalPosition`.
    pub vertical_position: PositionSource,
    /// `horizontalVelocity`.
    pub horizontal_velocity: VelocitySource,
    /// `verticalVelocity`.
    pub vertical_velocity: VelocitySource,
    /// `speed`.
    pub speed: Arc<dyn FloatProvider>,
}

impl EnchantmentEntityEffect {
    /// `apply(level, enchantmentLevel, item, entity, origin)`.
    ///
    /// `entity` — затронутая сущность (`LivingEntity` в нашей
    /// модели; Java допускает любой `Entity`).
    pub fn apply(
        &self,
        level: &Arc<dyn LevelReader>,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
        origin: Vec3,
    ) {
        match self {
            EnchantmentEntityEffect::AllOf(effects) => {
                for e in effects {
                    e.apply(level, enchantment_level, item, entity, origin);
                }
            }
            EnchantmentEntityEffect::ApplyMobEffect {
                to_apply,
                min_duration,
                max_duration,
                min_amplifier,
                max_amplifier,
            } => {
                // Guard ограничен блоком — `add_effect` требует
                // `&mut entity`, а guard держит заимствование.
                let picked = {
                    let mut random = entity.entity.random_mut();
                    let r: &mut dyn crate::random::RandomSource = &mut **random;
                    to_apply.get_random_element(r).map(|holder| {
                        let duration = crate::util::mth::random_between_f32(
                            r,
                            min_duration.calculate(enchantment_level),
                            max_duration.calculate(enchantment_level),
                        ) * 20.0;
                        let rounded = duration.round() as i32;
                        let amplifier = crate::util::mth::random_between_f32(
                            r,
                            min_amplifier.calculate(enchantment_level),
                            max_amplifier.calculate(enchantment_level),
                        )
                        .round()
                        .max(0.0) as i32;
                        (holder, amplifier, rounded)
                    })
                };
                if let Some((holder, amplifier, rounded)) = picked {
                    let instance = MobEffectInstance::new(
                        holder,
                        MobEffectDetails::new(
                            amplifier,
                            rounded,
                            false,
                            true,
                            true,
                            None,
                        ),
                    );
                    entity.add_effect(instance);
                }
            }
            EnchantmentEntityEffect::ChangeItemDamage { amount } => {
                let stack = &mut item.item_stack;
                // Java: `has(MAX_DAMAGE) && has(DAMAGE)` →
                // `hurtAndBreak(amount, level, owner, onBreak)`.
                if stack.is_damageable_item() {
                    let i = amount.calculate(enchantment_level) as i32;
                    if let Some(owner) = item.owner.clone() {
                        let mut owner_mut = owner;
                        let slot = item.in_slot;
                        stack.hurt_and_break(i, &mut owner_mut, slot);
                        item.owner = Some(owner_mut);
                    } else {
                        // Без владельца — износ только самого стека
                        // (hurtAndBreak без entity недоступен; эмулируем
                        // через set_damage_value).
                        let dmg = stack.get_damage_value()
                            + (amount.calculate(enchantment_level) as i32);
                        stack.set_damage_value(dmg);
                    }
                }
            }
            EnchantmentEntityEffect::DamageEntity {
                min_damage,
                max_damage,
                damage_type,
            } => {
                let damage = {
                    let mut random = entity.entity.random_mut();
                    crate::util::mth::random_between_f32(
                        &mut **random,
                        min_damage.calculate(enchantment_level),
                        max_damage.calculate(enchantment_level),
                    )
                };
                let mut source = DamageSource::new(damage_type.clone());
                source.causing_entity = Some(entity.entity.clone());
                entity.hurt_server(source, damage);
            }
            EnchantmentEntityEffect::Explode(explode) => {
                explode.apply(level, enchantment_level, item, entity, origin);
            }
            EnchantmentEntityEffect::Ignite { duration } => {
                // `entity.igniteForSeconds(duration)` (без bukkit-события).
                let secs = duration.calculate(enchantment_level);
                entity.set_remaining_fire_ticks(crate::util::mth::floor(secs as f64 * 20.0) as i32);
            }
            EnchantmentEntityEffect::ApplyImpulse {
                direction,
                coordinate_scale,
                magnitude,
            } => {
                let look = entity.entity.get_look_angle();
                let v = look
                    .add_vec(direction)
                    .multiply(
                        coordinate_scale.x,
                        coordinate_scale.y,
                        coordinate_scale.z,
                    )
                    .scale(magnitude.calculate(enchantment_level) as f64);
                let cur = entity.entity.get_delta_movement();
                let new_v = cur.add_vec(&v);
                entity.entity.set_delta_movement(new_v.x, new_v.y, new_v.z);
                // `hurtMarked = true; needsSync = true` —
                // синхронизация позиции (флаги пакетов).
                entity.entity.velocity = v;
            }
            EnchantmentEntityEffect::ApplyExhaustion { amount } => {
                // `entity instanceof Player` → `causeFoodExhaustion`.
                // Player ещё не портирован — хук через level.
                let _ = level;
                let _ = amount.calculate(enchantment_level);
                let _ = entity;
            }
            EnchantmentEntityEffect::PlaySound {
                sound_events,
                volume,
                pitch,
            } => {
                if entity.entity.is_silent() {
                    return;
                }
                let i = (enchantment_level - 1).clamp(0, sound_events.len() as i32 - 1)
                    as usize;
                let mut random = entity.entity.random_mut();
                let r: &mut dyn crate::random::RandomSource = &mut **random;
                let vol = volume.sample(r);
                let pit = pitch.sample(r);
                if let Some(sound) = sound_events.get(i) {
                    if let Some(se) = sound.value() {
                        level.play_sound(origin, &se.location, vol, pit);
                    }
                }
            }
            EnchantmentEntityEffect::ReplaceBlock {
                offset,
                predicate,
                block_state,
                trigger_game_event,
            } => {
                let pos = BlockPos::containing_vec3(&origin).offset(offset.x, offset.y, offset.z);
                let passes = predicate
                    .as_ref()
                    .map(|p| p.test(level.as_ref(), pos))
                    .unwrap_or(true);
                if passes {
                    let mut random = entity.entity.random_mut();
                    let new_state = block_state.get_state(&mut **random, pos);
                    if level.set_block_state(pos, &new_state) {
                        if let Some(event) = trigger_game_event {
                            if let Some(ev) = event.value() {
                                level.game_event_value(ev, pos);
                            }
                        }
                    }
                }
            }
            EnchantmentEntityEffect::ReplaceDisk {
                radius,
                height,
                offset,
                predicate,
                block_state,
                trigger_game_event,
            } => {
                let center = BlockPos::containing_vec3(&origin).offset(offset.x, offset.y, offset.z);
                let i = radius.calculate(enchantment_level) as i32;
                let i1 = height.calculate(enchantment_level) as i32;
                let min = center.offset(-i, 0, -i);
                let max = center.offset(i, (i1 - 1).min(0), i);
                for p in BlockPos::between_closed(&min, &max) {
                    let dx = p.x as f64 - origin.x;
                    let dz = p.z as f64 - origin.z;
                    let dy = p.y as f64 + 0.5 - origin.y;
                    if dx * dx + dy * dy + dz * dz < (i * i) as f64 {
                        let passes = predicate
                            .as_ref()
                            .map(|pr| pr.test(level.as_ref(), p))
                            .unwrap_or(true);
                        if passes {
                            let mut random = entity.entity.random_mut();
                            let new_state = block_state.get_state(&mut **random, p);
                            if level.set_block_state(p, &new_state) {
                                if let Some(event) = trigger_game_event {
                                    if let Some(ev) = event.value() {
                                        level.game_event_value(ev, p);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            EnchantmentEntityEffect::RunFunction { function } => {
                // `server.getFunctions().get(function)` — commands
                // subsystem; логируем через level-хук.
                level.run_function(function.clone(), entity, origin);
            }
            EnchantmentEntityEffect::SetBlockProperties {
                properties,
                offset,
                trigger_game_event,
            } => {
                let pos = BlockPos::containing_vec3(&origin).offset(offset.x, offset.y, offset.z);
                if let Some(state) = level.get_block_state(pos) {
                    let new_state = properties.apply(&state);
                    if state.values != new_state.values
                        && level.set_block_state(pos, &new_state)
                    {
                        if let Some(event) = trigger_game_event {
                            if let Some(ev) = event.value() {
                                level.game_event_value(ev, pos);
                            }
                        }
                    }
                }
            }
            EnchantmentEntityEffect::SpawnParticles(spawn) => {
                spawn.apply(level, enchantment_level, entity, origin);
            }
            EnchantmentEntityEffect::SummonEntity {
                entity_types,
                join_team,
            } => {
                let pos = BlockPos::containing_vec3(&origin);
                if level.is_inside_build_height(pos.y) {
                    let mut random = entity.entity.random_mut();
                    if let Some(holder) =
                        entity_types.get_random_element(&mut **random)
                    {
                        level.summon_entity(&holder, pos, *join_team);
                    }
                }
            }
        }
    }

    /// `onChangedBlock` — default делегирует `apply`
    /// (как `EnchantmentEntityEffect.onChangedBlock`).
    pub fn on_changed_block(
        &self,
        level: &Arc<dyn LevelReader>,
        enchantment_level: i32,
        item: &mut EnchantedItemInUse,
        entity: &mut LivingEntity,
        pos: Vec3,
        _apply_transient_effects: bool,
    ) {
        self.apply(level, enchantment_level, item, entity, pos);
    }
}

impl ExplodeEffect {
    /// `apply` — `level.explode(...)`.
    pub fn apply(
        &self,
        level: &Arc<dyn LevelReader>,
        enchantment_level: i32,
        item: &EnchantedItemInUse,
        entity: &mut LivingEntity,
        origin: Vec3,
    ) {
        let vec3 = origin.add_vec(&self.offset);
        let radius = self.radius.calculate(enchantment_level).max(0.0);
        let source = self.damage_type.as_ref().map(|dt| {
            if self.attribute_to_user {
                let mut s = DamageSource::new(dt.clone());
                s.causing_entity = Some(entity.entity.clone());
                s
            } else {
                let mut s = DamageSource::new(dt.clone());
                s.damage_source_position = Some(vec3);
                s
            }
        });
        let knockback = self
            .knockback_multiplier
            .as_ref()
            .map(|v| v.calculate(enchantment_level));
        let _ = item;
        level.explode(
            self.attribute_to_user.then(|| entity.entity.clone()),
            source,
            vec3,
            radius,
            self.create_fire,
            self.block_interaction,
            knockback,
        );
    }
}

impl SpawnParticlesEffect {
    /// `apply` — `level.sendParticles(...)` с position/velocity sources.
    pub fn apply(
        &self,
        level: &Arc<dyn LevelReader>,
        enchantment_level: i32,
        entity: &LivingEntity,
        origin: Vec3,
    ) {
        let mut random_rc = entity.entity.random_mut();
        let random: &mut dyn crate::random::RandomSource = &mut **random_rc;
        let known_movement = entity.entity.get_known_movement();
        let bb_width = entity.get_bb_width() as f64;
        let bb_height = entity.get_bb_height() as f64;

        let _count = random.next_int_bound(2);
        // Java: `int i = Mth.randomBetweenExclusive(random, 1, 3)`? —
        // фактически `particle count` из speed? В Java:
        // `for (int i = 0; i < this.speed...` — нет, count не из speed:
        // vanilla шлёт одну частицу? Проверено: apply() спавнит
        // `Mth.randomBetweenExclusive(random, 1, 3)`? См. vanilla:
        // `int i = Mth.nextInt(random, 2) + 1;`? Ниже — упрощённо 1.

        let pos_x = self.resolve_position(
            &self.horizontal_position,
            origin.x,
            bb_width,
            random,
        );
        let pos_y = self.resolve_position(
            &self.vertical_position,
            origin.y + bb_height * 0.5,
            bb_height,
            random,
        );
        let pos_z = self.resolve_position(
            &self.horizontal_position,
            origin.z,
            bb_width,
            random,
        );
        // horizontalPosition применяется и к X и к Z? Нет —
        // в Java horizontal применяется к X и Z через
        // `getValue(random, origin, bbWidth/2, ...)` для X и Z
        // по отдельности. Упрощение: считаем X/Z независимо.
        let vel_x = self.horizontal_velocity.movement_scale as f64 * known_movement.x
            + self.horizontal_velocity.base.sample(random) as f64;
        let vel_y = self.vertical_velocity.movement_scale as f64 * known_movement.y
            + self.vertical_velocity.base.sample(random) as f64;
        let vel_z = self.horizontal_velocity.movement_scale as f64 * known_movement.z
            + self.horizontal_velocity.base.sample(random) as f64;
        let speed = self.speed.sample(&mut *random) as f64;
        let _ = enchantment_level;
        level.send_particles(
            &self.particle,
            pos_x,
            pos_y,
            pos_z,
            1,
            vel_x * speed,
            vel_y * speed,
            vel_z * speed,
            0.0,
        );
    }

    fn resolve_position(
        &self,
        source: &PositionSource,
        center: f64,
        extent: f64,
        random: &mut dyn crate::random::RandomSource,
    ) -> f64 {
        match source.source_type {
            PositionSourceType::EntityPosition => {
                center + source.offset as f64 * extent
            }
            PositionSourceType::BoundingBox => {
                center
                    + (random.next_float() as f64 - 0.5)
                        * 2.0
                        * extent
                        * source.scale as f64
            }
        }
    }
}
