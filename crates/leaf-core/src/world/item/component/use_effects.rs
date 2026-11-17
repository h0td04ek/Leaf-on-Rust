//! Перенос `net.minecraft.world.item.component.UseEffects`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `UseEffects` — эффекты при использовании предмета.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UseEffects {
    /// `canSprint`.
    pub can_sprint: bool,
    /// `interactVibrations`.
    pub interact_vibrations: bool,
    /// `speedMultiplier`.
    pub speed_multiplier: f32,
}

impl UseEffects {
    /// `DEFAULT`.
    pub const DEFAULT: UseEffects = UseEffects {
        can_sprint: false,
        interact_vibrations: true,
        speed_multiplier: 0.2,
    };

    /// `STREAM_CODEC` — BOOL + BOOL + FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, UseEffects> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &UseEffects| {
                buf.write_bool(v.can_sprint);
                buf.write_bool(v.interact_vibrations);
                buf.write_float(v.speed_multiplier);
            },
            |buf: &mut FriendlyByteBuf| UseEffects {
                can_sprint: buf.read_bool(),
                interact_vibrations: buf.read_bool(),
                speed_multiplier: buf.read_float(),
            },
        )
    }
}
