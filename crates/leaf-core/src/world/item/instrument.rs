//! Перенос `net.minecraft.world.item.Instrument`.

use crate::core::holder::{Holder, HolderStreamCodec};
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::chat::component::Component;
use crate::sounds::sound_event::SoundEvent;

/// `Instrument` — (soundEvent, useDuration, range, description).
#[derive(Clone, Debug, PartialEq)]
pub struct Instrument {
    /// `soundEvent`.
    pub sound_event: Holder<SoundEvent>,
    /// `useDuration`.
    pub use_duration: f32,
    /// `range`.
    pub range: f32,
    /// `description`.
    pub description: Component,
}

impl Instrument {
    /// `new`.
    pub fn new(sound_event: Holder<SoundEvent>, use_duration: f32, range: f32, description: Component) -> Self {
        Self { sound_event, use_duration, range, description }
    }

    /// `DIRECT_STREAM_CODEC` — SoundEvent holder + FLOAT + FLOAT +
    /// Component.
    pub fn direct_stream_codec() -> impl StreamCodec<FriendlyByteBuf, Instrument> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Instrument| {
                SoundEvent::stream_codec().encode(buf, &v.sound_event);
                buf.write_float(v.use_duration);
                buf.write_float(v.range);
                component_stream_codec().encode(buf, &v.description);
            },
            |buf: &mut FriendlyByteBuf| {
                let sound_event = SoundEvent::stream_codec().decode(buf);
                let use_duration = buf.read_float();
                let range = buf.read_float();
                let description = component_stream_codec().decode(buf);
                Instrument::new(sound_event, use_duration, range, description)
            },
        )
    }

    /// `Instrument.STREAM_CODEC` — `holder(INSTRUMENT, direct)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<Instrument>> + Send + Sync + 'static {
        crate::core::holder::HolderStreamCodec::new(Instrument::direct_stream_codec())
    }
}
