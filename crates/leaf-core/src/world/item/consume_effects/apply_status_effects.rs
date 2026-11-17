//! Перенос `net.minecraft.world.item.consume_effects.ApplyStatusEffectsConsumeEffect`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::effect::mob_effect_instance::MobEffectInstance;

/// `ApplyStatusEffectsConsumeEffect` — список эффектов + вероятность.
#[derive(Clone, Debug, PartialEq)]
pub struct ApplyStatusEffectsConsumeEffect {
    /// `effects`.
    pub effects: Vec<MobEffectInstance>,
    /// `probability`.
    pub probability: f32,
}

impl ApplyStatusEffectsConsumeEffect {
    /// `new`.
    pub fn new(effects: Vec<MobEffectInstance>, probability: f32) -> Self {
        Self { effects, probability }
    }

    /// `STREAM_CODEC` — list(MobEffectInstance) + FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(MobEffectInstance::stream_codec());
        let list_dec = byte_buf_codecs::list(MobEffectInstance::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                list_enc.encode(buf, &v.effects);
                buf.write_float(v.probability);
            },
            move |buf: &mut FriendlyByteBuf| {
                let effects = list_dec.decode(buf);
                let probability = buf.read_float();
                Self { effects, probability }
            },
        )
    }
}
