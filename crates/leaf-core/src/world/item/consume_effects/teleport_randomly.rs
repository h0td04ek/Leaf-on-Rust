//! Перенос `net.minecraft.world.item.consume_effects.TeleportRandomlyConsumeEffect`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `TeleportRandomlyConsumeEffect` — `diameter` (default 16.0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TeleportRandomlyConsumeEffect {
    /// `diameter`.
    pub diameter: f32,
}

impl TeleportRandomlyConsumeEffect {
    /// `DEFAULT_DIAMETER`.
    pub const DEFAULT_DIAMETER: f32 = 16.0;

    /// `new`.
    pub fn new(diameter: f32) -> Self {
        Self { diameter }
    }

    /// `STREAM_CODEC` — FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Self> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Self| {
                buf.write_float(v.diameter);
            },
            |buf: &mut FriendlyByteBuf| Self { diameter: buf.read_float() },
        )
    }
}
