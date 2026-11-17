//! Перенос `net.minecraft.sounds.SoundEvent`.

use crate::core::holder::{Holder, HolderStreamCodec};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `SoundEvent` — запись реестра `Registries.SOUND_EVENT`.
#[derive(Clone, Debug, PartialEq)]
pub struct SoundEvent {
    /// `location` — id звука (`minecraft:entity.pig.ambient` и т.п.).
    pub location: Identifier,
    /// `fixedRange` — фиксированная дальность слышимости (optional).
    pub fixed_range: Option<f32>,
}

impl SoundEvent {
    /// `SoundEvent::create`-эквивалент.
    pub fn new(location: Identifier, fixed_range: Option<f32>) -> Self {
        Self { location, fixed_range }
    }

    /// `location`.
    pub fn location(&self) -> &Identifier {
        &self.location
    }

    /// `fixedRange`.
    pub fn fixed_range(&self) -> Option<f32> {
        self.fixed_range
    }

    /// `DIRECT_STREAM_CODEC` — Identifier + optional FLOAT.
    pub fn direct_stream_codec() -> impl StreamCodec<FriendlyByteBuf, SoundEvent> {
        sync_fn(
            |buf: &mut FriendlyByteBuf, s: &SoundEvent| {
                buf.write_identifier(&s.location).expect("sound event id");
                buf.write_optional(&s.fixed_range, |b, v| {
                    b.write_float(*v);
                });
            },
            |buf: &mut FriendlyByteBuf| {
                let location = buf.read_identifier().expect("sound event id");
                let fixed_range = buf.read_optional(|b| b.read_float());
                SoundEvent::new(location, fixed_range)
            },
        )
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.holder(SOUND_EVENT, DIRECT)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<SoundEvent>> {
        HolderStreamCodec::new(SoundEvent::direct_stream_codec())
    }
}
