//! Перенос `net.minecraft.world.item.component.DeathProtection`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::consume_effects::ConsumeEffect;

/// `DeathProtection` — список `ConsumeEffect` при смерти.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeathProtection {
    /// `deathEffects`.
    pub death_effects: Vec<ConsumeEffect>,
}

impl DeathProtection {
    /// `new`.
    pub fn new(death_effects: Vec<ConsumeEffect>) -> Self {
        Self { death_effects }
    }

    /// `STREAM_CODEC` — list(ConsumeEffect).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DeathProtection> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(ConsumeEffect::stream_codec());
        let list_dec = byte_buf_codecs::list(ConsumeEffect::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &DeathProtection| {
                list_enc.encode(buf, &v.death_effects);
            },
            move |buf: &mut FriendlyByteBuf| DeathProtection {
                death_effects: list_dec.decode(buf),
            },
        )
    }
}
