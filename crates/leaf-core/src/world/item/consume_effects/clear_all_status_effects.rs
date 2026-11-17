//! Перенос `net.minecraft.world.item.consume_effects.ClearAllStatusEffectsConsumeEffect`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `ClearAllStatusEffectsConsumeEffect` — unit-запись.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ClearAllStatusEffectsConsumeEffect;

impl ClearAllStatusEffectsConsumeEffect {
    /// `INSTANCE`.
    pub const INSTANCE: Self = Self;

    /// `STREAM_CODEC` — `StreamCodec.unit`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        sync_fn(|_, _| {}, |_| Self)
    }
}
