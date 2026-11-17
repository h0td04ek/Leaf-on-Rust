//! Перенос `net.minecraft.world.item.component.OminousBottleAmplifier`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `OminousBottleAmplifier` — уровень зелья в зловещей бутыли (0..4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OminousBottleAmplifier {
    /// `value`.
    pub value: i32,
}

impl OminousBottleAmplifier {
    /// `STREAM_CODEC` — `ByteBufCodecs.VAR_INT`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, OminousBottleAmplifier> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &OminousBottleAmplifier| {
                buf.write_var_int(v.value);
            },
            |buf: &mut FriendlyByteBuf| OminousBottleAmplifier { value: buf.read_var_int() },
        )
    }
}
