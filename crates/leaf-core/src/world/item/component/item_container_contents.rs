//! Перенос `net.minecraft.world.item.component.ItemContainerContents`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item_stack::ItemStack;

/// `ItemContainerContents` — `NonNullList<ItemStack>` (≤256).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ItemContainerContents {
    /// `items`.
    pub items: Vec<ItemStack>,
}

impl ItemContainerContents {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { items: Vec::new() }
    }

    /// `MAX_SIZE`.
    pub const MAX_SIZE: usize = 256;

    /// `NO_SLOT`.
    pub const NO_SLOT: i32 = -1;

    /// `new(items)`.
    pub fn new(items: Vec<ItemStack>) -> Self {
        Self { items }
    }

    /// `nonEmptyItems` — непустые стеки.
    pub fn non_empty_items(&self) -> impl Iterator<Item = &ItemStack> {
        self.items.iter().filter(|s| !s.is_empty())
    }

    /// `STREAM_CODEC` — list(256, ItemStack.OPTIONAL_STREAM_CODEC).
    /// Пустой стак кодируется optional-кодеком (count<=0 маркер).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemContainerContents> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list_limited(256, ItemStack::optional_stream_codec());
        let list_dec = byte_buf_codecs::list_limited(256, ItemStack::optional_stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &ItemContainerContents| {
                list_enc.encode(buf, &v.items);
            },
            move |buf: &mut FriendlyByteBuf| ItemContainerContents {
                items: list_dec.decode(buf),
            },
        )
    }
}
