//! Перенос `net.minecraft.world.effect.MobEffect` — полный порт:
//! категория, цвет, модификаторы атрибутов, blend-длительности,
//! звук при добавлении и поведенческие варианты (`MobEffectKind`
//! покрывает все ванильные подклассы).

use crate::core::holder::Holder;
use crate::entity::ai::attributes::attribute_modifier::{AttributeModifier, AttributeModifierOperation};
use crate::entity::ai::attributes::attribute_map::AttributeMap;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::sounds::sound_event::SoundEvent;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::damagesource::damage_types;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::level::level_reader::LevelReader;

/// `MobEffectCategory` — BENEFICIAL/HARMFUL/NEUTRAL.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MobEffectCategory {
    /// `BENEFICIAL`.
    Beneficial,
    /// `HARMFUL`.
    Harmful,
    /// `NEUTRAL`.
    Neutral,
}

impl MobEffectCategory {
    /// serialized name.
    pub fn serialized_name(&self) -> &'static str {
        match self {
            MobEffectCategory::Beneficial => "beneficial",
            MobEffectCategory::Harmful => "harmful",
            MobEffectCategory::Neutral => "neutral",
        }
    }
    /// `byName`.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "beneficial" => Some(MobEffectCategory::Beneficial),
            "harmful" => Some(MobEffectCategory::Harmful),
            "neutral" => Some(MobEffectCategory::Neutral),
            _ => None,
        }
    }
}

/// `MobEffect.AttributeTemplate` — (id, amount, operation).
#[derive(Clone, Debug, PartialEq)]
pub struct AttributeTemplate {
    /// `id` — identifier модификатора.
    pub id: Identifier,
    /// `amount`.
    pub amount: f64,
    /// `operation`.
    pub operation: AttributeModifierOperation,
}

impl AttributeTemplate {
    /// `create(amplifier)` — `AttributeModifier` с амплификатором.
    pub fn create(&self, amplifier: i32) -> AttributeModifier {
        AttributeModifier::new(
            self.id.clone(),
            self.amount * (amplifier as f64 + 1.0),
            self.operation,
        )
    }
}

/// Поведенческий вариант эффекта — соответствует ванильным
/// подклассам `MobEffect`.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum MobEffectKind {
    /// Базовый `MobEffect` — только атрибуты/звук.
    #[default]
    Base,
    /// `HealOrHarmMobEffect` — `isHarm` = harmful-ветка.
    HealOrHarm {
        /// `isHarm` — instant_damage (true) или instant_health (false).
        is_harm: bool,
    },
    /// `InstantenousEffectMobEffect` — `isInstantenous` = true.
    Instantaneous,
    /// `RegenerationMobEffect`.
    Regeneration,
    /// `HungerMobEffect`.
    Hunger,
    /// `PoisonMobEffect`.
    Poison,
    /// `WitherMobEffect`.
    Wither,
    /// `SaturationMobEffect`.
    Saturation,
    /// `AbsorptionMobEffect`.
    Absorption,
    /// `BadOmenMobEffect`.
    BadOmen,
    /// `RaidOmenMobEffect`.
    RaidOmen,
    /// `WindChargedMobEffect`.
    WindCharged,
    /// `WeavingMobEffect` — onMobRemoved → шипы (2..=3).
    Weaving {
        /// `min_max` — диапазон количества.
        min_max: (i32, i32),
    },
    /// `OozingMobEffect` — onMobRemoved → slime (фикс. 2).
    Oozing {
        /// количество.
        count: i32,
    },
    /// `InfestedMobEffect` — onMobRemoved → silverfish с шансом.
    Infested {
        /// `chance` — 0.1F.
        chance: f32,
        /// `min_max` — 1..=2.
        min_max: (i32, i32),
    },
    /// Атрибутный/косметический эффект без дополнительного тика
    /// (Speed/Slowness/Haste/MiningFatigue/Strength/Weakness/Luck/
    /// Unluck/JumpBoost/HealthBoost/SlowFalling/FireResistance/
    /// WaterBreathing/Invisibility/NightVision/Glowing/Nausea/
    /// Blindness/Levitation/Luck-ветки/DolphinsGrace/
    /// ConduitPower/HeroOfTheVillage/Darkness/TrialOmen/
    /// BreathOfTheNautilus/Resistance — модификаторы и
    /// state-флаги покрываются LivingEntity через атрибуты).
    AttributeOnly,
}

/// `MobEffect` — полная модель эффекта.
#[derive(Clone, Debug, PartialEq)]
pub struct MobEffect {
    /// registry-ключ (`minecraft:speed`).
    pub id: Identifier,
    /// `category`.
    pub category: MobEffectCategory,
    /// `color`.
    pub color: i32,
    /// `attributeModifiers` — (attribute id, template).
    pub attribute_modifiers: Vec<(Identifier, AttributeTemplate)>,
    /// `blendInDurationTicks`.
    pub blend_in_ticks: i32,
    /// `blendOutDurationTicks`.
    pub blend_out_ticks: i32,
    /// `blendOutAdvanceTicks`.
    pub blend_out_advance_ticks: i32,
    /// `soundOnAdded`.
    pub sound_on_added: Option<Holder<SoundEvent>>,
    /// `kind` — поведение.
    pub kind: MobEffectKind,
}

impl MobEffect {
    /// `new` — базовый конструктор.
    pub fn new(id: Identifier) -> Self {
        Self {
            id,
            category: MobEffectCategory::Beneficial,
            color: 0,
            attribute_modifiers: Vec::new(),
            blend_in_ticks: 0,
            blend_out_ticks: 0,
            blend_out_advance_ticks: 0,
            sound_on_added: None,
            kind: MobEffectKind::Base,
        }
    }

    /// `MobEffect(category, color)`.
    pub fn with_category_color(id: Identifier, category: MobEffectCategory, color: i32) -> Self {
        Self { category, color, ..Self::new(id) }
    }

    /// `addAttributeModifier` — fluent.
    pub fn add_attribute_modifier(
        mut self,
        attribute: Identifier,
        id: Identifier,
        amount: f64,
        operation: AttributeModifierOperation,
    ) -> Self {
        self.attribute_modifiers.push((attribute, AttributeTemplate { id, amount, operation }));
        self
    }

    /// `setBlendDuration(in, out, advance)`.
    pub fn set_blend_duration(mut self, blend_in: i32, blend_out: i32, blend_out_advance: i32) -> Self {
        self.blend_in_ticks = blend_in;
        self.blend_out_ticks = blend_out;
        self.blend_out_advance_ticks = blend_out_advance;
        self
    }

    /// `withSoundOnAdded`.
    pub fn with_sound_on_added(mut self, sound: Holder<SoundEvent>) -> Self {
        self.sound_on_added = Some(sound);
        self
    }

    /// `with_kind`.
    pub fn with_kind(mut self, kind: MobEffectKind) -> Self {
        self.kind = kind;
        self
    }

    /// `isInstantenous()`.
    pub fn is_instantaneous(&self) -> bool {
        matches!(
            self.kind,
            MobEffectKind::Instantaneous | MobEffectKind::HealOrHarm { .. } | MobEffectKind::Saturation
        )
    }

    /// `applyEffectTick(level, entity, amplifier)` —
    /// `false` = эффект должен быть снят.
    pub fn apply_effect_tick(
        &self,
        _level: Option<&dyn LevelReader>,
        entity: &mut LivingEntity,
        amplifier: i32,
    ) -> bool {
        match &self.kind {
            MobEffectKind::Regeneration => {
                if entity.get_health() < entity.get_max_health() {
                    entity.heal(1.0);
                }
                true
            }
            MobEffectKind::Poison => {
                if entity.get_health() > 1.0 {
                    entity.hurt_server(effect_damage_source(damage_types::magic()), 1.0);
                }
                true
            }
            MobEffectKind::Wither => {
                entity.hurt_server(effect_damage_source(damage_types::wither()), 1.0);
                true
            }
            MobEffectKind::Hunger => {
                // HungerMobEffect: exhaustion += 0.005 * (amplifier+1) per tick
                entity.cause_food_exhaustion(0.005 * (amplifier + 1) as f32);
                true
            }
            _ => true,
        }

    }

    /// `shouldApplyEffectTickThisTick(duration, amplifier)`.
    /// Vanilla: `period <= 0 || duration % period == 0`,
    /// period = `BASE >> amplifier` (Java int-shift).
    pub fn should_apply_effect_tick_this_tick(&self, duration: i32, amplifier: i32) -> bool {
        let period: i32 = match &self.kind {
            MobEffectKind::Regeneration => 50i32 >> (amplifier & 31),
            MobEffectKind::Poison => 25i32 >> (amplifier & 31),
            MobEffectKind::Wither => 40i32 >> (amplifier & 31),
            _ => return false,
        };
        period <= 0 || duration % period == 0
    }

    /// `onEffectStarted(entity, amplifier)`.
    pub fn on_effect_started(&self, entity: &mut LivingEntity, amplifier: i32) {
        match &self.kind {
            MobEffectKind::Absorption => {
                let amount = (amplifier + 1) * 4;
                let current = entity.get_absorption_amount() as i32;
                entity.set_absorption_amount((current + amount).min(16) as f32);
            }
            _ => {}
        }
    }

    /// `onEffectAdded(entity, amplifier)` — звук.
    pub fn on_effect_added(&self, entity: &mut LivingEntity, _amplifier: i32) {
        if let Some(sound) = &self.sound_on_added {
            entity.entity.play_sound(Some(sound), 1.0, 1.0);
        }
    }

    /// `onMobRemoved(level, entity, amplifier, reason)`.
    pub fn on_mob_removed(
        &self,
        level: Option<&dyn LevelReader>,
        entity: &mut LivingEntity,
        _amplifier: i32,
        reason: crate::world::entity::entity::RemovalReason,
    ) {
        match &self.kind {
            MobEffectKind::Absorption => {
                entity.set_absorption_amount(0.0);
            }
            MobEffectKind::Oozing { count } => {
                self.oozing_on_removed(level, entity, *count, reason);
            }
            MobEffectKind::Weaving { min_max } => {
                self.weaving_on_removed(level, entity, *min_max, reason);
            }
            MobEffectKind::Infested { .. } => {
                // InfestedMobEffect срабатывает на onMobHurt, не здесь.
            }
            _ => {}
        }
    }

    /// `OozingMobEffect.onMobRemoved` — спавн `count` слизней
    /// (size 2) при KILLED, ограниченный MAX_ENTITY_CRAMMING.
    fn oozing_on_removed(
        &self,
        level: Option<&dyn LevelReader>,
        entity: &mut LivingEntity,
        count: i32,
        reason: crate::world::entity::entity::RemovalReason,
    ) {
        use crate::world::entity::entity::RemovalReason as R;
        let Some(level) = level else { return };
        if reason != R::Killed {
            return;
        }
        let max_cramming = level.max_entity_cramming();
        let nearby = level.count_nearby_entities(
            &Identifier::with_default_namespace("slime").unwrap(),
            entity.entity.pos,
            2.0,
        );
        let spawn_count = if max_cramming < 1 {
            count
        } else {
            (max_cramming - nearby).clamp(0, count)
        };
        let level_arc = entity.entity.level.clone();
        for _ in 0..spawn_count {
            let mut slime = crate::world::entity::entity::Entity::new(
                Holder::Direct(crate::world::entity::entity_type::slime_entity_type()),
                level_arc.clone(),
            );
            let yaw = rand::random::<f32>() * 360.0;
            slime.pos = crate::world::phys::vec3::Vec3::new(
                entity.entity.pos.x,
                entity.entity.pos.y + 0.5,
                entity.entity.pos.z,
            );
            slime.rotation = crate::world::entity::entity::Vec2::new(yaw, 0.0);
            level.add_fresh_entity(slime);
        }
    }

    /// `WeavingMobEffect.onMobRemoved` — паутина в случайных
    /// заменяемых блоках вокруг (2..=3 шт.) при KILLED.
    fn weaving_on_removed(
        &self,
        level: Option<&dyn LevelReader>,
        entity: &mut LivingEntity,
        min_max: (i32, i32),
        reason: crate::world::entity::entity::RemovalReason,
    ) {
        use crate::world::entity::entity::RemovalReason as R;
        let Some(level) = level else { return };
        let is_player = entity
            .entity
            .entity_type
            .value()
            .map(|t| t.id.get_path() == "player")
            .unwrap_or(false);
        if reason != R::Killed || !(is_player || level.mob_griefing()) {
            return;
        }
        let mut random = crate::random::legacy::create_unique();
        let max_webs = crate::util::mth::random_between_inclusive(&mut *random, min_max.0, min_max.1);
        let pos = crate::core::block_pos::BlockPos::containing_vec3(&entity.entity.pos);
        let mut set = std::collections::HashSet::new();
        for cand in crate::core::block_pos::BlockPos::random_in_cube(&mut *random, 15, &pos, 1) {
            let below = cand.below();
            if !set.contains(&cand)
                && level.can_replace_at(cand)
                && level.is_face_sturdy(below, crate::core::direction::Direction::UP)
            {
                set.insert(cand);
                if set.len() as i32 >= max_webs {
                    break;
                }
            }
        }
        let cobweb = Identifier::with_default_namespace("cobweb").unwrap();
        for p in set {
            if level.set_block_by_id(p, &cobweb) {
                // LevelEvent.ANIMATION_SPAWN_COBWEB = 3018
                level.level_event(3018, p, 0);
            }
        }
    }

    /// `onMobHurt(level, entity, amplifier, source, amount)`.
    pub fn on_mob_hurt(
        &self,
        _level: Option<&dyn LevelReader>,
        entity: &mut LivingEntity,
        amplifier: i32,
        _source: &DamageSource,
        _amount: f32,
    ) {
        match &self.kind {
            MobEffectKind::WindCharged => {
                // WindChargedMobEffect: 15% + 5% за уровень (cap 100%)
                let chance = (0.15 + 0.05 * amplifier as f32).min(1.0);
                if crate::random::legacy::create_unique().next_float() < chance {
                    if let Some(level) = entity.entity.level.clone() {
                        level.spawn_lightning_bolt(entity.entity.pos);
                    }
                }
            }
            MobEffectKind::Infested { chance, min_max } => {
                // InfestedMobEffect.onMobHurt — шанс `chance`,
                // спавн min_max серебряных рыб на высоте середины hitbox.
                if crate::random::legacy::create_unique().next_float() <= *chance {
                    let count = crate::util::mth::random_between_inclusive(
                        &mut *crate::random::legacy::create_unique(),
                        min_max.0,
                        min_max.1,
                    );
                    let level_arc = entity.entity.level.clone();
                    let y = entity.entity.pos.y + entity.entity.dimensions.height as f64 / 2.0;
                    for _ in 0..count {
                        let mut fish = crate::world::entity::entity::Entity::new(
                            Holder::Direct(crate::world::entity::entity_type::silverfish_entity_type()),
                            level_arc.clone(),
                        );
                        let yaw = rand::random::<f32>() * 360.0;
                        fish.pos = crate::world::phys::vec3::Vec3::new(
                            entity.entity.pos.x,
                            y,
                            entity.entity.pos.z,
                        );
                        fish.rotation = crate::world::entity::entity::Vec2::new(yaw, 0.0);
                        if let Some(l) = &level_arc {
                            l.add_fresh_entity(fish);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    /// `applyInstantenousEffect` — для мгновенных эффектов.
    pub fn apply_instantaneous_effect(
        &self,
        _level: Option<&dyn LevelReader>,
        entity: &mut LivingEntity,
        amplifier: i32,
    ) {
        match &self.kind {
            MobEffectKind::HealOrHarm { is_harm } => {
                // HealOrHarmMobEffect: `isHarm == isInvertedHealAndHarm` → heal.
                // amount = health * (4 << amp) для heal, (6 << amp) для harm.
                // `health` у нас = 1.0 (applyInstantenousEffect вызывается
                // с health=1.0 для potion-эффектов).
                let inverted = entity.is_inverted_heal_and_harm();
                if *is_harm == inverted {
                    let i = ((4i32 << amplifier.min(4)) as f32 * 1.0 + 0.5) as i32;
                    entity.heal(i.max(0) as f32);
                } else {
                    let i = ((6i32 << amplifier.min(4)) as f32 * 1.0 + 0.5) as i32;
                    if !entity.entity.is_invulnerable() {
                        entity.hurt_server(effect_damage_source(damage_types::magic()), i as f32);
                    }
                }
            }
            MobEffectKind::Saturation => {
                entity.eat_saturation(amplifier + 1);
            }
            _ => {
                self.apply_effect_tick(_level, entity, amplifier);
            }
        }
    }

    /// `getOrCreateDescriptionId`-эквивалент.
    pub fn description_id(&self) -> String {
        format!("effect.{}", self.id.get_path())
    }

    /// `getCategory`.
    pub fn category(&self) -> MobEffectCategory {
        self.category
    }

    /// `getColor`.
    pub fn color(&self) -> i32 {
        self.color
    }

    /// `createModifiers(amplifier, output)`.
    pub fn create_modifiers<F>(&self, amplifier: i32, mut output: F)
    where
        F: FnMut(Identifier, AttributeModifier),
    {
        for (attr, template) in &self.attribute_modifiers {
            output(attr.clone(), template.create(amplifier));
        }
    }

    /// `addAttributeModifiers(attributeMap, amplifier)`.
    pub fn add_attribute_modifiers(&self, attribute_map: &mut AttributeMap, amplifier: i32) {
        self.create_modifiers(amplifier, |attr, modifier| {
            attribute_map.add_transient_attribute_modifiers(&attr, vec![modifier]);
        });
    }

    /// `removeAttributeModifiers(attributeMap)` — удаление по id
    /// модификатора (vanilla удаляет по uuid — id играет ту же роль).
    pub fn remove_attribute_modifiers(&self, attribute_map: &mut AttributeMap) {
        for (attr, template) in &self.attribute_modifiers {
            attribute_map.remove_attribute_modifier_by_id(attr, &template.id);
        }
    }

    /// `MobEffect.STREAM_CODEC` — `holderRegistry(MOB_EFFECT)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<MobEffect>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<MobEffect>()
    }
}

/// Источник урона эффекта — `DamageSource::new(Holder::Key(key))`,
/// эквивалент `level.damageSources().xxx()` без entity-привязки.
pub fn effect_damage_source(
    key: crate::resources::resource_key::ResourceKey<crate::world::damagesource::damage_type::DamageType>,
) -> DamageSource {
    DamageSource::new(Holder::Key(key))
}

/// Удобный алиас: `Arc<MobEffect>`-подобный holder value.
pub type MobEffectValue = std::sync::Arc<MobEffect>;

pub mod attribute_consts {
    pub use crate::entity::ai::attributes::attributes::*;
}


