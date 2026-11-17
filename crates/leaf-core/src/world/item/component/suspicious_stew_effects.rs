//! Перенос `net.minecraft.world.item.component.SuspiciousStewEffects`.

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::effect::mob_effect::MobEffect;

/// `SuspiciousStewEffects.Entry` — effect holder + duration.
#[derive(Clone, Debug, PartialEq)]
pub struct SuspiciousStewEntry {
    /// `effect`.
    pub effect: Holder<MobEffect>,
    /// `duration`.
    pub duration: i32,
}

impl SuspiciousStewEntry {
    /// `new`.
    pub fn new(effect: Holder<MobEffect>, duration: i32) -> Self {
        Self { effect, duration }
    }

    /// `STREAM_CODEC` — MobEffect.STREAM_CODEC + VAR_INT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Self| {
                crate::world::effect::mob_effect::MobEffect::stream_codec().encode(buf, &v.effect);
                buf.write_var_int(v.duration);
            },
            |buf: &mut FriendlyByteBuf| {
                let effect = crate::world::effect::mob_effect::MobEffect::stream_codec().decode(buf);
                let duration = buf.read_var_int();
                SuspiciousStewEntry { effect, duration }
            },
        )
    }
}

/// `SuspiciousStewEffects` — список Entry.
#[derive(Clone, Debug, PartialEq)]
pub struct SuspiciousStewEffects {
    /// `effects`.
    pub effects: Vec<SuspiciousStewEntry>,
}

impl SuspiciousStewEffects {
    /// `new`.
    pub fn new(effects: Vec<SuspiciousStewEntry>) -> Self {
        Self { effects }
    }

    /// `STREAM_CODEC` — list(Entry).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list(SuspiciousStewEntry::stream_codec());
        let list_dec = byte_buf_codecs::list(SuspiciousStewEntry::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Self| {
                list_enc.encode(buf, &v.effects);
            },
            move |buf: &mut FriendlyByteBuf| {
                SuspiciousStewEffects { effects: list_dec.decode(buf) }
            },
        )
    }
}
