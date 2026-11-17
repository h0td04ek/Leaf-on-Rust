//! Перенос `net.minecraft.world.item.component.SwingAnimation`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::swing_animation_type::SwingAnimationType;

/// `SwingAnimation` — (type, duration).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SwingAnimation {
    /// `type`.
    pub type_: SwingAnimationType,
    /// `duration`.
    pub duration: i32,
}

impl SwingAnimation {
    /// `DEFAULT` — WHACK, 6.
    pub const DEFAULT: SwingAnimation = SwingAnimation {
        type_: SwingAnimationType::WHACK,
        duration: 6,
    };

    /// `new`.
    pub fn new(type_: SwingAnimationType, duration: i32) -> Self {
        Self { type_, duration }
    }

    /// `STREAM_CODEC` — SwingAnimationType + VAR_INT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, SwingAnimation> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &SwingAnimation| {
                SwingAnimationType::stream_codec().encode(buf, &v.type_);
                buf.write_var_int(v.duration);
            },
            |buf: &mut FriendlyByteBuf| {
                let type_ = SwingAnimationType::stream_codec().decode(buf);
                let duration = buf.read_var_int();
                SwingAnimation::new(type_, duration)
            },
        )
    }
}
