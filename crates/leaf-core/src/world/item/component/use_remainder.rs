//! Перенос `net.minecraft.world.item.component.UseRemainder`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item_stack::ItemStack;

/// `UseRemainder` — (convertInto: ItemStack).
#[derive(Clone, Debug, PartialEq)]
pub struct UseRemainder {
    /// `convertInto`.
    pub convert_into: ItemStack,
}

impl UseRemainder {
    /// `new`.
    pub fn new(convert_into: ItemStack) -> Self {
        Self { convert_into }
    }

    /// `STREAM_CODEC` — ItemStack.STREAM_CODEC (increaseDepth
    /// отражён ограничением глубины NBT внутри ItemStack).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, UseRemainder> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &UseRemainder| {
                ItemStack::stream_codec().encode(buf, &v.convert_into);
            },
            |buf: &mut FriendlyByteBuf| {
                let convert_into = ItemStack::stream_codec().decode(buf);
                UseRemainder::new(convert_into)
            },
        )
    }
}
