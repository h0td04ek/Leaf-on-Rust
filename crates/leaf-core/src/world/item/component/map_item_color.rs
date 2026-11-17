//! Перенос `net.minecraft.world.item.component.MapItemColor`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `MapItemColor` — (rgb).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MapItemColor {
    /// `rgb`.
    pub rgb: i32,
}

impl MapItemColor {
    /// `DEFAULT`.
    pub const DEFAULT: MapItemColor = MapItemColor { rgb: 4603950 };

    /// `new`.
    pub fn new(rgb: i32) -> Self {
        Self { rgb }
    }

    /// `STREAM_CODEC` — INT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MapItemColor> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &MapItemColor| {
                buf.write_int(v.rgb);
            },
            |buf: &mut FriendlyByteBuf| MapItemColor::new(buf.read_int()),
        )
    }
}
