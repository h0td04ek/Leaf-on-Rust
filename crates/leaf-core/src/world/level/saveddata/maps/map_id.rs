//! Перенос `net.minecraft.world.level.saveddata.maps.MapId`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `MapId` — id карты мира (как `MapItemSavedData` counter).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct MapId(pub i32);

impl MapId {
    /// `STREAM_CODEC` — `ByteBufCodecs.VAR_INT`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MapId> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &MapId| {
                buf.write_var_int(v.0);
            },
            |buf: &mut FriendlyByteBuf| MapId(buf.read_var_int()),
        )
    }
}
