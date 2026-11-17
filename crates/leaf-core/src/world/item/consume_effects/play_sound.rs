//! Перенос `net.minecraft.world.item.consume_effects.PlaySoundConsumeEffect`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::sounds::sound_event::SoundEvent;

/// `PlaySoundConsumeEffect` — holder звука.
#[derive(Clone, Debug, PartialEq)]
pub struct PlaySoundConsumeEffect {
    /// `sound`.
    pub sound: Holder<SoundEvent>,
}

impl PlaySoundConsumeEffect {
    /// `new`.
    pub fn new(sound: Holder<SoundEvent>) -> Self {
        Self { sound }
    }

    /// `STREAM_CODEC` — SoundEvent holder codec.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Self| {
                SoundEvent::stream_codec().encode(buf, &v.sound);
            },
            |buf: &mut FriendlyByteBuf| {
                Self { sound: SoundEvent::stream_codec().decode(buf) }
            },
        )
    }
}
