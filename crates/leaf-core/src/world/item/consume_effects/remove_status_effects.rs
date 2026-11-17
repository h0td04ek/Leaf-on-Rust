//! Перенос `net.minecraft.world.item.consume_effects.RemoveStatusEffectsConsumeEffect`.

use crate::core::holder_set::{holder_set_stream_codec, HolderSet};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::effect::mob_effect::MobEffect;

/// `RemoveStatusEffectsConsumeEffect` — holderSet эффектов для снятия.
#[derive(Clone, Debug, PartialEq)]
pub struct RemoveStatusEffectsConsumeEffect {
    /// `effects`.
    pub effects: HolderSet<MobEffect>,
}

impl RemoveStatusEffectsConsumeEffect {
    /// `new`.
    pub fn new(effects: HolderSet<MobEffect>) -> Self {
        Self { effects }
    }

    /// `STREAM_CODEC` — holderSet(MOB_EFFECT).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let effects_enc = holder_set_stream_codec::<MobEffect>();
        let effects_dec = holder_set_stream_codec::<MobEffect>();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                effects_enc.encode(buf, &v.effects);
            },
            move |buf: &mut FriendlyByteBuf| {
                Self { effects: effects_dec.decode(buf) }
            },
        )
    }
}
