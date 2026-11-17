//! Перенос `net.minecraft.world.item.component.DyedItemColor`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `DyedItemColor` — RGB-цвет окрашиваемого предмета.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DyedItemColor {
    /// `rgb`.
    pub rgb: i32,
}

impl DyedItemColor {
    /// `LEATHER_COLOR`.
    pub const LEATHER_COLOR: i32 = -6265536;

    /// `STREAM_CODEC` — `ByteBufCodecs.INT`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DyedItemColor> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &DyedItemColor| {
                buf.write_int(v.rgb);
            },
            |buf: &mut FriendlyByteBuf| DyedItemColor { rgb: buf.read_int() },
        )
    }
}
