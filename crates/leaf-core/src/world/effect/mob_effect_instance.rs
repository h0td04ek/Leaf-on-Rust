//! Перенос `net.minecraft.world.effect.MobEffectInstance` —
//! активный эффект на сущности.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::effect::mob_effect::MobEffect;

/// `MobEffectInstance.Details` — amplifier/duration/флаги +
/// рекурсивный hiddenEffect.
#[derive(Clone, Debug, PartialEq)]
pub struct MobEffectDetails {
    /// `amplifier`.
    pub amplifier: i32,
    /// `duration`.
    pub duration: i32,
    /// `ambient`.
    pub ambient: bool,
    /// `showParticles`.
    pub show_particles: bool,
    /// `showIcon`.
    pub show_icon: bool,
    /// `hiddenEffect` (optional, рекурсивный).
    pub hidden_effect: Option<Box<MobEffectDetails>>,
}

impl MobEffectDetails {
    /// `new`.
    pub fn new(
        amplifier: i32,
        duration: i32,
        ambient: bool,
        show_particles: bool,
        show_icon: bool,
        hidden_effect: Option<Box<MobEffectDetails>>,
    ) -> Self {
        Self { amplifier, duration, ambient, show_particles, show_icon, hidden_effect }
    }

    fn encode(&self, buf: &mut FriendlyByteBuf, depth: usize) {
        assert!(depth < 64, "mob effect details recursion too deep");
        buf.write_var_int(self.amplifier);
        buf.write_var_int(self.duration);
        buf.write_bool(self.ambient);
        buf.write_bool(self.show_particles);
        buf.write_bool(self.show_icon);
        match &self.hidden_effect {
            Some(h) => {
                buf.write_bool(true);
                h.encode(buf, depth + 1);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }

    /// `updateDetailsFrom(other)` — копия полей деталей
    /// (без hiddenEffect — vanilla Details record его не содержит).
    pub fn update_details_from(&mut self, other: &MobEffectDetails) {
        self.amplifier = other.amplifier;
        self.duration = other.duration;
        self.ambient = other.ambient;
        self.show_particles = other.show_particles;
        self.show_icon = other.show_icon;
    }

    fn decode(buf: &mut FriendlyByteBuf, depth: usize) -> Self {
        assert!(depth < 64, "mob effect details recursion too deep");
        let amplifier = buf.read_var_int();
        let duration = buf.read_var_int();
        let ambient = buf.read_bool();
        let show_particles = buf.read_bool();
        let show_icon = buf.read_bool();
        let hidden_effect = if buf.read_bool() {
            Some(Box::new(Self::decode(buf, depth + 1)))
        } else {
            None
        };
        Self { amplifier, duration, ambient, show_particles, show_icon, hidden_effect }
    }

    /// `STREAM_CODEC` (`StreamCodec.recursive` + increaseDepth —
    /// depth-учёт реализован счётчиком аргумента).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MobEffectDetails> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &MobEffectDetails| v.encode(buf, 0),
            |buf: &mut FriendlyByteBuf| MobEffectDetails::decode(buf, 0),
        )
    }
}

/// `MobEffectInstance` — (effect holder, details).
#[derive(Clone, Debug, PartialEq)]
pub struct MobEffectInstance {
    /// `effect` — holder эффекта.
    pub effect: Holder<MobEffect>,
    /// `details`.
    pub details: MobEffectDetails,
}

impl MobEffectInstance {
    /// `new`.
    pub fn new(effect: Holder<MobEffect>, details: MobEffectDetails) -> Self {
        Self { effect, details }
    }

    /// `getAmplifier`, `getDuration` и т.п. — делегаты.
    pub fn amplifier(&self) -> i32 {
        self.details.amplifier
    }
    pub fn duration(&self) -> i32 {
        self.details.duration
    }

    /// `tick()` — уменьшается ли длительность.
    pub fn tick(&mut self) {
        if self.details.duration > 0 {
            self.details.duration -= 1;
        }
    }

    /// `isExpired()`.
    pub fn is_expired(&self) -> bool {
        self.details.duration <= 0
    }

    /// `toNbt()` — сериализация в NBT.
    pub fn to_nbt(&self) -> crate::nbt::tag::Tag {
        let mut c = crate::nbt::compound::CompoundTag::new();
        if let Some(id) = self.effect.key().map(|k| k.id().to_string()) {
            c.put("Id", crate::nbt::tag::Tag::String(id));
        }
        c.put("Amplifier", crate::nbt::tag::Tag::Int(self.details.amplifier));
        c.put("Duration", crate::nbt::tag::Tag::Int(self.details.duration));
        crate::nbt::tag::Tag::Compound(c)
    }

    /// `fromNbt(tag)`.
    pub fn from_nbt(tag: &crate::nbt::compound::CompoundTag) -> Option<Self> {
        let id = tag.get_string("Id")?.to_string();
        let amplifier = tag.get_int("Amplifier").unwrap_or(0);
        let duration = tag.get_int("Duration").unwrap_or(0);
        // Holder::Key по identifier — реестр резолвит позже
        let ident = crate::resources::identifier::Identifier::parse(&id).ok()?;
        let key = crate::resources::resource_key::ResourceKey::create_with_registry_name(
            crate::core::registry::registries::mob_effect(),
            ident,
        );
        Some(Self::new(
            crate::core::holder::Holder::Key(key),
            MobEffectDetails::new(amplifier, duration, false, false, false, None),
        ))
    }

    /// `STREAM_CODEC` — MobEffect holder + Details.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MobEffectInstance> + Send + Sync + 'static {
        let effect_codec = crate::core::holder::holder_registry_stream_codec::<MobEffect>();
        let details_codec = MobEffectDetails::stream_codec();
        let effect_codec2 = crate::core::holder::holder_registry_stream_codec::<MobEffect>();
        let details_codec2 = MobEffectDetails::stream_codec();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &MobEffectInstance| {
                effect_codec.encode(buf, &v.effect);
                details_codec.encode(buf, &v.details);
            },
            move |buf: &mut FriendlyByteBuf| {
                let effect = effect_codec2.decode(buf);
                let details = details_codec2.decode(buf);
                MobEffectInstance::new(effect, details)
            },
        )
    }
}

impl MobEffectInstance {
    /// `isInfiniteDuration()` — `duration == -1`.
    pub fn is_infinite_duration(&self) -> bool {
        self.details.duration == -1
    }

    /// `hasRemainingDuration()`.
    fn has_remaining_duration(&self) -> bool {
        self.is_infinite_duration() || self.details.duration > 0
    }

    /// `mapDuration(op)` — для бесконечных возвращает -1.
    fn map_duration(&mut self, f: impl FnOnce(i32) -> i32) {
        self.details.duration = if self.is_infinite_duration() {
            -1
        } else {
            f(self.details.duration)
        };
    }

    /// `isShorterDurationThan(other)`.
    pub fn is_shorter_duration_than(&self, other: &MobEffectInstance) -> bool {
        !self.is_infinite_duration()
            && (self.details.duration < other.details.duration || other.is_infinite_duration())
    }

    /// `tickServer(level, entity, onEffectUpdated)` —
    /// `false` = эффект должен быть снят.
    ///
    /// `level` — `Option<&dyn LevelReader>` (entity.level).
    pub fn tick_server(
        &mut self,
        level: Option<&dyn crate::world::level::level_reader::LevelReader>,
        entity: &mut crate::world::entity::living_entity::LivingEntity,
        on_effect_updated: &mut dyn FnMut(),
    ) -> bool {
        if !self.has_remaining_duration() {
            return false;
        }
        let duration = if self.is_infinite_duration() {
            entity.entity.tick_count
        } else {
            self.details.duration
        };
        let should_apply = self.effect.value().map(|e| {
            e.should_apply_effect_tick_this_tick(duration, self.details.amplifier)
        }).unwrap_or(false);
        if should_apply {
            let applied = self.effect.value().map(|e| {
                e.apply_effect_tick(level, entity, self.details.amplifier)
            }).unwrap_or(true);
            if !applied {
                return false;
            }
        }
        self.tick_down_duration();
        if self.downgrade_to_hidden_effect() {
            on_effect_updated();
        }
        self.has_remaining_duration()
    }

    /// `tickClient()` — клиентский тик (duration-- + downgrade +
    /// blend tick — blend-стейт не портирован).
    pub fn tick_client(&mut self) {
        if self.has_remaining_duration() {
            self.tick_down_duration();
            self.downgrade_to_hidden_effect();
        }
    }

    /// `tickDownDuration()`.
    fn tick_down_duration(&mut self) {
        if let Some(hidden) = self.details.hidden_effect.as_mut() {
            hidden.duration = if hidden.duration == -1 { -1 } else { hidden.duration - 1 };
        }
        self.map_duration(|d| d - 1);
    }

    /// `downgradeToHiddenEffect()`.
    fn downgrade_to_hidden_effect(&mut self) -> bool {
        if self.details.duration == 0 {
            if let Some(hidden) = self.details.hidden_effect.take() {
                let mut new_details = (*hidden).clone();
                new_details.hidden_effect = hidden.hidden_effect.clone();
                self.details = new_details;
                return true;
            }
        }
        false
    }

    /// `onEffectStarted(entity)` — делегат в `MobEffect`.
    pub fn on_effect_started(&self, entity: &mut crate::world::entity::living_entity::LivingEntity) {
        if let Some(e) = self.effect.value() {
            e.on_effect_started(entity, self.details.amplifier);
        }
    }

    /// `onEffectAdded(entity)` — звук (vanilla `MobEffect.onEffectAdded`).
    pub fn on_effect_added(&self, entity: &mut crate::world::entity::living_entity::LivingEntity) {
        if let Some(e) = self.effect.value() {
            e.on_effect_added(entity, self.details.amplifier);
        }
    }

    /// `onMobRemoved(level, entity, reason)`.
    pub fn on_mob_removed(
        &self,
        level: Option<&dyn crate::world::level::level_reader::LevelReader>,
        entity: &mut crate::world::entity::living_entity::LivingEntity,
        reason: crate::world::entity::entity::RemovalReason,
    ) {
        if let Some(e) = self.effect.value() {
            e.on_mob_removed(level, entity, self.details.amplifier, reason);
        }
    }

    /// `onMobHurt(level, entity, amplifier, source, amount)` —
    /// делегат в `MobEffect.onMobHurt`.
    pub fn on_mob_hurt(
        &self,
        level: Option<&dyn crate::world::level::level_reader::LevelReader>,
        entity: &mut crate::world::entity::living_entity::LivingEntity,
        source: &crate::world::damagesource::damage_source::DamageSource,
        amount: f32,
    ) {
        if let Some(e) = self.effect.value() {
            e.on_mob_hurt(level, entity, self.details.amplifier, source, amount);
        }
    }

    /// `addAttributeModifiers(attributeMap, amplifier)` —
    /// через `MobEffect.createModifiers`.
    pub fn add_attribute_modifiers(
        &self,
        attribute_map: &mut crate::entity::ai::attributes::attribute_map::AttributeMap,
    ) {
        if let Some(e) = self.effect.value() {
            e.add_attribute_modifiers(attribute_map, self.details.amplifier);
        }
    }

    /// `removeAttributeModifiers(attributeMap)`.
    pub fn remove_attribute_modifiers(
        &self,
        attribute_map: &mut crate::entity::ai::attributes::attribute_map::AttributeMap,
    ) {
        if let Some(e) = self.effect.value() {
            e.remove_attribute_modifiers(attribute_map);
        }
    }

    /// `update(other)` — слияние нового инстанса в текущий
    /// (vanilla `MobEffectInstance.update`).
    pub fn update(&mut self, other: &MobEffectInstance) -> bool {
        let mut flag = false;
        if other.details.amplifier > self.details.amplifier {
            if other.is_shorter_duration_than(self) {
                // hiddenEffect = копия this; hiddenEffect.hiddenEffect = this.hiddenEffect
                let copy = self.clone();
                let mut hidden = copy.details;
                hidden.hidden_effect = self.details.hidden_effect.clone();
                self.details.hidden_effect = Some(Box::new(hidden));
            }
            self.details.amplifier = other.details.amplifier;
            self.details.duration = other.details.duration;
            flag = true;
        } else if self.is_shorter_duration_than(other) {
            if other.details.amplifier == self.details.amplifier {
                self.details.duration = other.details.duration;
                flag = true;
            } else if self.details.hidden_effect.is_none() {
                self.details.hidden_effect = Some(Box::new(other.details.clone()));
            } else if let Some(hidden) = self.details.hidden_effect.as_mut() {
                // рекурсивный update деталей hidden-эффекта
                hidden.update_details_from(&other.details);
            }
        }

        if (!other.details.ambient && self.details.ambient) || flag {
            self.details.ambient = other.details.ambient;
            flag = true;
        }
        if other.details.show_particles != self.details.show_particles {
            self.details.show_particles = other.details.show_particles;
            flag = true;
        }
        if other.details.show_icon != self.details.show_icon {
            self.details.show_icon = other.details.show_icon;
            flag = true;
        }
        flag
    }

    /// `updateDetailsFrom(other)` — копия деталей без hiddenEffect.
    fn update_details_from(&mut self, other: &MobEffectInstance) {
        self.details.amplifier = other.details.amplifier;
        self.details.duration = other.details.duration;
        self.details.ambient = other.details.ambient;
        self.details.show_particles = other.details.show_particles;
        self.details.show_icon = other.details.show_icon;
    }

    /// `getBlendFactor(entity, delta)` — blend-state не портирован;
    /// возвращаем видимость как бинарный фактор.
    pub fn get_blend_factor(&self, _entity: &crate::world::entity::living_entity::LivingEntity, _delta: f32) -> f32 {
        if self.details.show_particles { 1.0 } else { 0.0 }
    }

    /// `copyBlendState(other)` — blend-state не портирован (no-op).
    pub fn copy_blend_state(&mut self, _other: &MobEffectInstance) {}

    /// `is(Holder<MobEffect>)` — сравнение по registry-ключу.
    pub fn is(&self, effect: &Holder<MobEffect>) -> bool {
        match (&self.effect, effect) {
            (Holder::Direct(a), Holder::Direct(b)) => a.id == b.id,
            (Holder::Key(a), Holder::Key(b)) => a == b,
            _ => self
                .effect
                .value()
                .zip(effect.value())
                .map(|(a, b)| a.id == b.id)
                .unwrap_or(false),
        }
    }
}
