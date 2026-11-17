//! Перенос `net.minecraft.world.item.component.ChargedProjectiles`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item_stack::ItemStack;

/// `ChargedProjectiles` — список ItemStack (≤64).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChargedProjectiles {
    /// `items`.
    pub items: Vec<ItemStack>,
}

impl ChargedProjectiles {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { items: Vec::new() }
    }

    /// `of(stack)`.
    pub fn of(stack: ItemStack) -> Self {
        Self { items: vec![stack] }
    }

    /// `of(list)`.
    pub fn of_list(items: Vec<ItemStack>) -> Self {
        Self { items }
    }

    /// `STREAM_CODEC` — list(64, ItemStack) (increaseDepth —
    /// глубина NBT контролируется accounter'ом ItemStack).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ChargedProjectiles> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list_limited(64, ItemStack::stream_codec());
        let list_dec = byte_buf_codecs::list_limited(64, ItemStack::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &ChargedProjectiles| {
                list_enc.encode(buf, &v.items);
            },
            move |buf: &mut FriendlyByteBuf| ChargedProjectiles {
                items: list_dec.decode(buf),
            },
        )
    }
}
