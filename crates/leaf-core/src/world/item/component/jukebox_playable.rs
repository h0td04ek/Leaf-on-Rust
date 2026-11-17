//! Перенос `net.minecraft.world.item.JukeboxPlayable`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::either_holder::EitherHolder;
use crate::world::item::jukebox_song::JukeboxSong;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;

/// `JukeboxPlayable` — `EitherHolder<JukeboxSong>`.
#[derive(Clone, Debug, PartialEq)]
pub struct JukeboxPlayable {
    /// `song`.
    pub song: EitherHolder<JukeboxSong>,
}

impl JukeboxPlayable {
    /// `new`.
    pub fn new(song: EitherHolder<JukeboxSong>) -> Self {
        Self { song }
    }

    /// `of_key`.
    pub fn of_key(key: ResourceKey<JukeboxSong>) -> Self {
        Self { song: EitherHolder::of_key(key) }
    }

    /// `STREAM_CODEC` — `EitherHolder.streamCodec(JUKEBOX_SONG,
    /// JukeboxSong.DIRECT_STREAM_CODEC)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, JukeboxPlayable> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &JukeboxPlayable| {
                EitherHolder::stream_codec(
                    Identifier::minecraft("jukebox_song"),
                    JukeboxSong::stream_codec(),
                )
                .encode(buf, &v.song);
            },
            |buf: &mut FriendlyByteBuf| {
                let song = EitherHolder::stream_codec(
                    Identifier::minecraft("jukebox_song"),
                    JukeboxSong::stream_codec(),
                )
                .decode(buf);
                JukeboxPlayable::new(song)
            },
        )
    }
}
