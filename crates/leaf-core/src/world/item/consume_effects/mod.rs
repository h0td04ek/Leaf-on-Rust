//! Перенос `net.minecraft.world.item.consume_effects` — эффекты
//! при потреблении предмета.

pub mod apply_status_effects;
pub mod clear_all_status_effects;
pub mod play_sound;
pub mod remove_status_effects;
pub mod teleport_randomly;

pub use apply_status_effects::ApplyStatusEffectsConsumeEffect;
pub use clear_all_status_effects::ClearAllStatusEffectsConsumeEffect;
pub use play_sound::PlaySoundConsumeEffect;
pub use remove_status_effects::RemoveStatusEffectsConsumeEffect;
pub use teleport_randomly::TeleportRandomlyConsumeEffect;

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ConsumeEffect.Type` id — порядок регистрации в
/// `BuiltInRegistries.CONSUME_EFFECT_TYPE`.
pub const TYPE_APPLY_EFFECTS: i32 = 0;
pub const TYPE_REMOVE_EFFECTS: i32 = 1;
pub const TYPE_CLEAR_ALL_EFFECTS: i32 = 2;
pub const TYPE_TELEPORT_RANDOMLY: i32 = 3;
pub const TYPE_PLAY_SOUND: i32 = 4;

/// `ConsumeEffect` — tagged union по `Type`
/// (`ByteBufCodecs.registry(CONSUME_EFFECT_TYPE).dispatch`).
/// На проводе: VarInt type id, затем payload типа.
#[derive(Clone, Debug, PartialEq)]
pub enum ConsumeEffect {
    /// `apply_effects`.
    ApplyStatusEffects(ApplyStatusEffectsConsumeEffect),
    /// `remove_effects`.
    RemoveStatusEffects(RemoveStatusEffectsConsumeEffect),
    /// `clear_all_effects`.
    ClearAllStatusEffects(ClearAllStatusEffectsConsumeEffect),
    /// `teleport_randomly`.
    TeleportRandomly(TeleportRandomlyConsumeEffect),
    /// `play_sound`.
    PlaySound(PlaySoundConsumeEffect),
}

impl ConsumeEffect {
    /// `getType` id.
    pub fn type_id(&self) -> i32 {
        match self {
            ConsumeEffect::ApplyStatusEffects(_) => TYPE_APPLY_EFFECTS,
            ConsumeEffect::RemoveStatusEffects(_) => TYPE_REMOVE_EFFECTS,
            ConsumeEffect::ClearAllStatusEffects(_) => TYPE_CLEAR_ALL_EFFECTS,
            ConsumeEffect::TeleportRandomly(_) => TYPE_TELEPORT_RANDOMLY,
            ConsumeEffect::PlaySound(_) => TYPE_PLAY_SOUND,
        }
    }

    /// `STREAM_CODEC` — registry dispatch.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ConsumeEffect> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &ConsumeEffect| {
                buf.write_var_int(v.type_id());
                match v {
                    ConsumeEffect::ApplyStatusEffects(x) => {
                        ApplyStatusEffectsConsumeEffect::stream_codec().encode(buf, x)
                    }
                    ConsumeEffect::RemoveStatusEffects(x) => {
                        RemoveStatusEffectsConsumeEffect::stream_codec().encode(buf, x)
                    }
                    ConsumeEffect::ClearAllStatusEffects(x) => {
                        ClearAllStatusEffectsConsumeEffect::stream_codec().encode(buf, x)
                    }
                    ConsumeEffect::TeleportRandomly(x) => {
                        TeleportRandomlyConsumeEffect::stream_codec().encode(buf, x)
                    }
                    ConsumeEffect::PlaySound(x) => {
                        PlaySoundConsumeEffect::stream_codec().encode(buf, x)
                    }
                }
            },
            |buf: &mut FriendlyByteBuf| match buf.read_var_int() {
                TYPE_APPLY_EFFECTS => ConsumeEffect::ApplyStatusEffects(
                    ApplyStatusEffectsConsumeEffect::stream_codec().decode(buf),
                ),
                TYPE_REMOVE_EFFECTS => ConsumeEffect::RemoveStatusEffects(
                    RemoveStatusEffectsConsumeEffect::stream_codec().decode(buf),
                ),
                TYPE_CLEAR_ALL_EFFECTS => ConsumeEffect::ClearAllStatusEffects(
                    ClearAllStatusEffectsConsumeEffect::stream_codec().decode(buf),
                ),
                TYPE_TELEPORT_RANDOMLY => ConsumeEffect::TeleportRandomly(
                    TeleportRandomlyConsumeEffect::stream_codec().decode(buf),
                ),
                TYPE_PLAY_SOUND => ConsumeEffect::PlaySound(
                    PlaySoundConsumeEffect::stream_codec().decode(buf),
                ),
                id => panic!("unknown consume effect type {id}"),
            },
        )
    }
}

impl ConsumeEffect {
    /// `apply(LivingEntity)` — применение эффекта при поедании/тотеме.
    pub fn apply(&self, entity: &mut crate::world::entity::living_entity::LivingEntity) {
        match self {
            ConsumeEffect::ApplyStatusEffects(e) => {
                for inst in &e.effects {
                    if rand::random::<f32>() < e.probability {
                        entity.add_effect(inst.clone());
                    }
                }
            }
            ConsumeEffect::RemoveStatusEffects(e) => {
                for holder in e.effects.iter() {
                    entity.remove_effect(holder);
                }
            }
            ConsumeEffect::ClearAllStatusEffects(_) => {
                entity.remove_all_effects();
            }
            ConsumeEffect::TeleportRandomly(e) => {
                entity.teleport_randomly(e.diameter);
            }
            ConsumeEffect::PlaySound(e) => {
                entity.entity.play_sound(Some(&e.sound), 1.0, 1.0);
            }
        }
    }
}
