//! Перенос `net.minecraft.world.item.JukeboxSong`.

use crate::core::holder::{Holder, HolderStreamCodec};
use crate::network::chat::component_json::component_stream_codec;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::chat::component::Component;
use crate::sounds::sound_event::SoundEvent;

/// `JukeboxSong` — (soundEvent, description, lengthInSeconds,
/// comparatorOutput).
#[derive(Clone, Debug, PartialEq)]
pub struct JukeboxSong {
    /// `soundEvent`.
    pub sound_event: Holder<SoundEvent>,
    /// `description`.
    pub description: Component,
    /// `lengthInSeconds`.
    pub length_in_seconds: f32,
    /// `comparatorOutput`.
    pub comparator_output: i32,
}

impl JukeboxSong {
    /// `SONG_END_PADDING_TICKS`.
    pub const SONG_END_PADDING_TICKS: i32 = 20;

    /// `new`.
    pub fn new(sound_event: Holder<SoundEvent>, description: Component, length_in_seconds: f32, comparator_output: i32) -> Self {
        Self { sound_event, description, length_in_seconds, comparator_output }
    }

    /// `DIRECT_STREAM_CODEC` — SoundEvent + Component + FLOAT + VAR_INT.
    pub fn direct_stream_codec() -> impl StreamCodec<FriendlyByteBuf, JukeboxSong> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &JukeboxSong| {
                SoundEvent::stream_codec().encode(buf, &v.sound_event);
                component_stream_codec().encode(buf, &v.description);
                buf.write_float(v.length_in_seconds);
                buf.write_var_int(v.comparator_output);
            },
            |buf: &mut FriendlyByteBuf| {
                let sound_event = SoundEvent::stream_codec().decode(buf);
                let description = component_stream_codec().decode(buf);
                let length_in_seconds = buf.read_float();
                let comparator_output = buf.read_var_int();
                JukeboxSong::new(sound_event, description, length_in_seconds, comparator_output)
            },
        )
    }

    /// `JukeboxSong.STREAM_CODEC` — `holder(JUKEBOX_SONG, direct)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Holder<JukeboxSong>> + Send + Sync + 'static {
        crate::core::holder::HolderStreamCodec::new(JukeboxSong::direct_stream_codec())
    }
}
