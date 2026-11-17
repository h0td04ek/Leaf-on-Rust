//! Перенос `net.minecraft.world.item.component.MapPostProcessing`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `MapPostProcessing` — LOCK(0), SCALE(1).
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MapPostProcessing {
    /// `LOCK`.
    LOCK,
    /// `SCALE`.
    SCALE,
}

impl MapPostProcessing {
    /// `ID_MAP` — `OutOfBoundsStrategy.ZERO`.
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// `byId`.
    pub fn by_id(id: i32) -> MapPostProcessing {
        match id {
            0 => MapPostProcessing::LOCK,
            1 => MapPostProcessing::SCALE,
            _ => MapPostProcessing::LOCK,
        }
    }

    /// `STREAM_CODEC` — idMapper.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MapPostProcessing> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &MapPostProcessing| {
                buf.write_var_int(v.id());
            },
            |buf: &mut FriendlyByteBuf| MapPostProcessing::by_id(buf.read_var_int()),
        )
    }
}
