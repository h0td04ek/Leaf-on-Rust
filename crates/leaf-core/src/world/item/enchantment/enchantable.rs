//! Перенос `net.minecraft.world.item.enchantment.Enchantable`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `Enchantable` — (value > 0).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Enchantable {
    /// `value`.
    pub value: i32,
}

impl Enchantable {
    /// `new`.
    pub fn new(value: i32) -> Self {
        assert!(value > 0, "Enchantment value must be positive, but was {value}");
        Self { value }
    }

    /// `STREAM_CODEC` — VAR_INT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Enchantable> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Enchantable| {
                buf.write_var_int(v.value);
            },
            |buf: &mut FriendlyByteBuf| Enchantable::new(buf.read_var_int()),
        )
    }
}
