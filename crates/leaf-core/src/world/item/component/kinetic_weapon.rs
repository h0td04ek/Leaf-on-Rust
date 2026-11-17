//! Перенос `net.minecraft.world.item.component.KineticWeapon`.

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::sounds::sound_event::SoundEvent;

/// `KineticWeapon.Condition` — (maxDurationTicks, minSpeed,
/// minRelativeSpeed).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KineticWeaponCondition {
    /// `maxDurationTicks`.
    pub max_duration_ticks: i32,
    /// `minSpeed`.
    pub min_speed: f32,
    /// `minRelativeSpeed`.
    pub min_relative_speed: f32,
}

impl KineticWeaponCondition {
    /// `new`.
    pub fn new(max_duration_ticks: i32, min_speed: f32, min_relative_speed: f32) -> Self {
        Self { max_duration_ticks, min_speed, min_relative_speed }
    }

    /// `Condition.STREAM_CODEC` — VAR_INT + FLOAT + FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, KineticWeaponCondition> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &KineticWeaponCondition| {
                buf.write_var_int(v.max_duration_ticks);
                buf.write_float(v.min_speed);
                buf.write_float(v.min_relative_speed);
            },
            |buf: &mut FriendlyByteBuf| {
                let max_duration_ticks = buf.read_var_int();
                let min_speed = buf.read_float();
                let min_relative_speed = buf.read_float();
                KineticWeaponCondition::new(max_duration_ticks, min_speed, min_relative_speed)
            },
        )
    }
}

/// `KineticWeapon` — 9 полей.
#[derive(Clone, Debug, PartialEq)]
pub struct KineticWeapon {
    /// `contactCooldownTicks`.
    pub contact_cooldown_ticks: i32,
    /// `delayTicks`.
    pub delay_ticks: i32,
    /// `dismountConditions`.
    pub dismount_conditions: Option<KineticWeaponCondition>,
    /// `knockbackConditions`.
    pub knockback_conditions: Option<KineticWeaponCondition>,
    /// `damageConditions`.
    pub damage_conditions: Option<KineticWeaponCondition>,
    /// `forwardMovement`.
    pub forward_movement: f32,
    /// `damageMultiplier`.
    pub damage_multiplier: f32,
    /// `sound`.
    pub sound: Option<Holder<SoundEvent>>,
    /// `hitSound`.
    pub hit_sound: Option<Holder<SoundEvent>>,
}

impl KineticWeapon {
    /// `HIT_FEEDBACK_TICKS`.
    pub const HIT_FEEDBACK_TICKS: i32 = 10;

    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        contact_cooldown_ticks: i32,
        delay_ticks: i32,
        dismount_conditions: Option<KineticWeaponCondition>,
        knockback_conditions: Option<KineticWeaponCondition>,
        damage_conditions: Option<KineticWeaponCondition>,
        forward_movement: f32,
        damage_multiplier: f32,
        sound: Option<Holder<SoundEvent>>,
        hit_sound: Option<Holder<SoundEvent>>,
    ) -> Self {
        Self {
            contact_cooldown_ticks,
            delay_ticks,
            dismount_conditions,
            knockback_conditions,
            damage_conditions,
            forward_movement,
            damage_multiplier,
            sound,
            hit_sound,
        }
    }

    /// `STREAM_CODEC` — порядок полей Java.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, KineticWeapon> + Send + Sync + 'static {
        let cond = KineticWeaponCondition::stream_codec();
        let dis_enc = byte_buf_codecs::optional(KineticWeaponCondition::stream_codec());
        let dis_dec = byte_buf_codecs::optional(KineticWeaponCondition::stream_codec());
        let knock_enc = byte_buf_codecs::optional(KineticWeaponCondition::stream_codec());
        let knock_dec = byte_buf_codecs::optional(KineticWeaponCondition::stream_codec());
        let dmg_enc = byte_buf_codecs::optional(KineticWeaponCondition::stream_codec());
        let dmg_dec = byte_buf_codecs::optional(KineticWeaponCondition::stream_codec());
        let snd_enc = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let snd_dec = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let hit_enc = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let hit_dec = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let _ = cond;
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &KineticWeapon| {
                buf.write_var_int(v.contact_cooldown_ticks);
                buf.write_var_int(v.delay_ticks);
                dis_enc.encode(buf, &v.dismount_conditions);
                knock_enc.encode(buf, &v.knockback_conditions);
                dmg_enc.encode(buf, &v.damage_conditions);
                buf.write_float(v.forward_movement);
                buf.write_float(v.damage_multiplier);
                snd_enc.encode(buf, &v.sound);
                hit_enc.encode(buf, &v.hit_sound);
            },
            move |buf: &mut FriendlyByteBuf| {
                let contact_cooldown_ticks = buf.read_var_int();
                let delay_ticks = buf.read_var_int();
                let dismount_conditions = dis_dec.decode(buf);
                let knockback_conditions = knock_dec.decode(buf);
                let damage_conditions = dmg_dec.decode(buf);
                let forward_movement = buf.read_float();
                let damage_multiplier = buf.read_float();
                let sound = snd_dec.decode(buf);
                let hit_sound = hit_dec.decode(buf);
                KineticWeapon::new(
                    contact_cooldown_ticks,
                    delay_ticks,
                    dismount_conditions,
                    knockback_conditions,
                    damage_conditions,
                    forward_movement,
                    damage_multiplier,
                    sound,
                    hit_sound,
                )
            },
        )
    }
}
