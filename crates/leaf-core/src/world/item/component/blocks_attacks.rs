//! Перенос `net.minecraft.world.item.component.BlocksAttacks`.

use crate::core::holder::Holder;
use crate::core::holder_set::{holder_set_stream_codec, HolderSet};
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::tags::TagKey;
use crate::world::damagesource::damage_type::DamageType;
use crate::sounds::sound_event::SoundEvent;

/// `BlocksAttacks.DamageReduction` — (horizontalBlockingAngle,
/// type?, base, factor).
#[derive(Clone, Debug, PartialEq)]
pub struct BlocksAttacksDamageReduction {
    /// `horizontalBlockingAngle`.
    pub horizontal_blocking_angle: f32,
    /// `type` — `HolderSet<DamageType>`.
    pub type_: Option<HolderSet<DamageType>>,
    /// `base`.
    pub base: f32,
    /// `factor`.
    pub factor: f32,
}

impl BlocksAttacksDamageReduction {
    /// `new`.
    pub fn new(
        horizontal_blocking_angle: f32,
        type_: Option<HolderSet<DamageType>>,
        base: f32,
        factor: f32,
    ) -> Self {
        Self { horizontal_blocking_angle, type_, base, factor }
    }

    /// `DamageReduction.STREAM_CODEC` — FLOAT + optional(holderSet) +
    /// FLOAT + FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let ty_enc = byte_buf_codecs::optional(holder_set_stream_codec::<DamageType>());
        let ty_dec = byte_buf_codecs::optional(holder_set_stream_codec::<DamageType>());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                buf.write_float(v.horizontal_blocking_angle);
                ty_enc.encode(buf, &v.type_);
                buf.write_float(v.base);
                buf.write_float(v.factor);
            },
            move |buf: &mut FriendlyByteBuf| {
                let horizontal_blocking_angle = buf.read_float();
                let type_ = ty_dec.decode(buf);
                let base = buf.read_float();
                let factor = buf.read_float();
                BlocksAttacksDamageReduction::new(horizontal_blocking_angle, type_, base, factor)
            },
        )
    }
}

impl BlocksAttacksDamageReduction {
    /// `resolve(damageSource, damageAmount, horizontalAngle)`.
    pub fn resolve(
        &self,
        damage_source: &crate::world::damagesource::damage_source::DamageSource,
        damage_amount: f32,
        horizontal_angle: f64,
    ) -> f32 {
        if horizontal_angle > (std::f64::consts::PI / 180.0) * self.horizontal_blocking_angle as f64 {
            return 0.0;
        }
        if let Some(type_) = &self.type_ {
            let matched = match damage_source.get_type().value() {
                Some(dt) => match type_ {
                    HolderSet::Direct(holders) => holders.iter().any(|h| {
                        matches!(h, Holder::Direct(i) if i.id == dt.id)
                            || h.value().map(|v| v.id == dt.id).unwrap_or(false)
                    }),
                    HolderSet::Named(tag_loc) => {
                        crate::tags::damage_type_tags::is_member_of_tag(tag_loc, &dt.id)
                    }
                },
                None => false,
            };
            if !matched {
                return 0.0;
            }
        }
        crate::util::mth::clamp_f32(self.base + self.factor * damage_amount, 0.0, damage_amount)
    }
}

/// `BlocksAttacks.ItemDamageFunction` — (threshold, base, factor).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlocksAttacksItemDamageFunction {
    /// `threshold`.
    pub threshold: f32,
    /// `base`.
    pub base: f32,
    /// `factor`.
    pub factor: f32,
}

impl BlocksAttacksItemDamageFunction {
    /// `DEFAULT`.
    pub const DEFAULT: Self = Self { threshold: 1.0, base: 0.0, factor: 1.0 };

    /// `new`.
    pub fn new(threshold: f32, base: f32, factor: f32) -> Self {
        Self { threshold, base, factor }
    }

    /// `apply(damageAmount)` — `floor(base + factor*damage)` если
    /// `damageAmount >= threshold`.
    pub fn apply(&self, damage_amount: f32) -> i32 {
        if damage_amount < self.threshold {
            0
        } else {
            (self.base + self.factor * damage_amount).floor() as i32
        }
    }

    /// `ItemDamageFunction.STREAM_CODEC` — 3× FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Self| {
                buf.write_float(v.threshold);
                buf.write_float(v.base);
                buf.write_float(v.factor);
            },
            |buf: &mut FriendlyByteBuf| {
                let threshold = buf.read_float();
                let base = buf.read_float();
                let factor = buf.read_float();
                BlocksAttacksItemDamageFunction::new(threshold, base, factor)
            },
        )
    }
}

/// `BlocksAttacks` — 7 полей.
#[derive(Clone, Debug, PartialEq)]
pub struct BlocksAttacks {
    /// `blockDelaySeconds`.
    pub block_delay_seconds: f32,
    /// `disableCooldownScale`.
    pub disable_cooldown_scale: f32,
    /// `damageReductions`.
    pub damage_reductions: Vec<BlocksAttacksDamageReduction>,
    /// `itemDamage`.
    pub item_damage: BlocksAttacksItemDamageFunction,
    /// `bypassedBy` — `TagKey<DamageType>`.
    pub bypassed_by: Option<TagKey<DamageType>>,
    /// `blockSound`.
    pub block_sound: Option<Holder<SoundEvent>>,
    /// `disableSound`.
    pub disable_sound: Option<Holder<SoundEvent>>,
}

impl BlocksAttacks {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_delay_seconds: f32,
        disable_cooldown_scale: f32,
        damage_reductions: Vec<BlocksAttacksDamageReduction>,
        item_damage: BlocksAttacksItemDamageFunction,
        bypassed_by: Option<TagKey<DamageType>>,
        block_sound: Option<Holder<SoundEvent>>,
        disable_sound: Option<Holder<SoundEvent>>,
    ) -> Self {
        Self {
            block_delay_seconds,
            disable_cooldown_scale,
            damage_reductions,
            item_damage,
            bypassed_by,
            block_sound,
            disable_sound,
        }
    }

    /// `STREAM_CODEC` — порядок полей Java.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, BlocksAttacks> + Send + Sync + 'static {
        let reductions_enc = byte_buf_codecs::list(BlocksAttacksDamageReduction::stream_codec());
        let reductions_dec = byte_buf_codecs::list(BlocksAttacksDamageReduction::stream_codec());
        let bypass_enc = byte_buf_codecs::optional(TagKey::stream_codec(
            crate::resources::resource_key::ResourceKey::create_registry_key(
                crate::resources::identifier::Identifier::parse("minecraft:damage_type")
                    .expect("damage_type registry"),
            ),
        ));
        let bypass_dec = byte_buf_codecs::optional(TagKey::stream_codec(
            crate::resources::resource_key::ResourceKey::create_registry_key(
                crate::resources::identifier::Identifier::parse("minecraft:damage_type")
                    .expect("damage_type registry"),
            ),
        ));
        let block_snd_enc = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let block_snd_dec = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let dis_snd_enc = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let dis_snd_dec = byte_buf_codecs::optional(SoundEvent::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &BlocksAttacks| {
                buf.write_float(v.block_delay_seconds);
                buf.write_float(v.disable_cooldown_scale);
                reductions_enc.encode(buf, &v.damage_reductions);
                BlocksAttacksItemDamageFunction::stream_codec().encode(buf, &v.item_damage);
                bypass_enc.encode(buf, &v.bypassed_by);
                block_snd_enc.encode(buf, &v.block_sound);
                dis_snd_enc.encode(buf, &v.disable_sound);
            },
            move |buf: &mut FriendlyByteBuf| {
                let block_delay_seconds = buf.read_float();
                let disable_cooldown_scale = buf.read_float();
                let damage_reductions = reductions_dec.decode(buf);
                let item_damage = BlocksAttacksItemDamageFunction::stream_codec().decode(buf);
                let bypassed_by = bypass_dec.decode(buf);
                let block_sound = block_snd_dec.decode(buf);
                let disable_sound = dis_snd_dec.decode(buf);
                BlocksAttacks::new(
                    block_delay_seconds,
                    disable_cooldown_scale,
                    damage_reductions,
                    item_damage,
                    bypassed_by,
                    block_sound,
                    disable_sound,
                )
            },
        )
    }

    /// `resolveBlockedDamage(damageSource, damageAmount, horizontalAngle)`.
    pub fn resolve_blocked_damage(
        &self,
        damage_source: &crate::world::damagesource::damage_source::DamageSource,
        damage_amount: f32,
        horizontal_angle: f64,
    ) -> f32 {
        let mut f = 0.0f32;
        for reduction in &self.damage_reductions {
            f += reduction.resolve(damage_source, damage_amount, horizontal_angle);
        }
        crate::util::mth::clamp_f32(f, 0.0, damage_amount)
    }

    /// `blockDelayTicks()`.
    pub fn block_delay_ticks(&self) -> i32 {
        (self.block_delay_seconds * 20.0).round() as i32
    }

    /// `onBlocked(level, entity)` — звук блокировки (вызывающий код
    /// проигрывает его через level).
    pub fn on_blocked(&self) -> Option<&Holder<SoundEvent>> {
        self.block_sound.as_ref()
    }
}
