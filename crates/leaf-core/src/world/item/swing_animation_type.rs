//! Перенос `net.minecraft.world.item.SwingAnimationType`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `SwingAnimationType` — тип анимации взмаха (id = wire).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SwingAnimationType {
    /// `NONE`.
    NONE,
    /// `WHACK`.
    WHACK,
    /// `STAB`.
    STAB,
}

impl SwingAnimationType {
    /// `getId`.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `BY_ID` — `OutOfBoundsStrategy.ZERO`.
    pub fn by_id(id: i32) -> SwingAnimationType {
        match id {
            0 => SwingAnimationType::NONE,
            1 => SwingAnimationType::WHACK,
            2 => SwingAnimationType::STAB,
            _ => SwingAnimationType::NONE,
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            SwingAnimationType::NONE => "none",
            SwingAnimationType::WHACK => "whack",
            SwingAnimationType::STAB => "stab",
        }
    }

    /// `STREAM_CODEC` — idMapper.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, SwingAnimationType> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &SwingAnimationType| {
                buf.write_var_int(v.get_id());
            },
            |buf: &mut FriendlyByteBuf| SwingAnimationType::by_id(buf.read_var_int()),
        )
    }
}
