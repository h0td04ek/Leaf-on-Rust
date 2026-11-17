//! Перенос `net.minecraft.world.item.component.PiercingWeapon`.

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::sounds::sound_event::SoundEvent;

/// `PiercingWeapon` — (dealsKnockback, dismounts, sound?, hitSound?).
#[derive(Clone, Debug, PartialEq)]
pub struct PiercingWeapon {
    /// `dealsKnockback`.
    pub deals_knockback: bool,
    /// `dismounts`.
    pub dismounts: bool,
    /// `sound`.
    pub sound: Option<Holder<SoundEvent>>,
    /// `hitSound`.
    pub hit_sound: Option<Holder<SoundEvent>>,
}

impl PiercingWeapon {
    /// `new`.
    pub fn new(
        deals_knockback: bool,
        dismounts: bool,
        sound: Option<Holder<SoundEvent>>,
        hit_sound: Option<Holder<SoundEvent>>,
    ) -> Self {
        Self { deals_knockback, dismounts, sound, hit_sound }
    }

    /// `STREAM_CODEC` — BOOL + BOOL + optional(SoundEvent)×2.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PiercingWeapon> + Send + Sync + 'static {
        let snd_enc = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let snd_dec = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let hit_enc = byte_buf_codecs::optional(SoundEvent::stream_codec());
        let hit_dec = byte_buf_codecs::optional(SoundEvent::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &PiercingWeapon| {
                buf.write_bool(v.deals_knockback);
                buf.write_bool(v.dismounts);
                snd_enc.encode(buf, &v.sound);
                hit_enc.encode(buf, &v.hit_sound);
            },
            move |buf: &mut FriendlyByteBuf| {
                let deals_knockback = buf.read_bool();
                let dismounts = buf.read_bool();
                let sound = snd_dec.decode(buf);
                let hit_sound = hit_dec.decode(buf);
                PiercingWeapon::new(deals_knockback, dismounts, sound, hit_sound)
            },
        )
    }
}
