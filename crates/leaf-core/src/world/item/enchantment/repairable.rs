//! Перенос `net.minecraft.world.item.enchantment.Repairable`.

use crate::core::holder_set::{holder_set_stream_codec, HolderSet};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item::Item;

/// `Repairable` — `HolderSet<Item>` допустимых материалов ремонта.
#[derive(Clone, Debug, PartialEq)]
pub struct Repairable {
    /// `items`.
    pub items: HolderSet<Item>,
}

impl Repairable {
    /// `new`.
    pub fn new(items: HolderSet<Item>) -> Self {
        Self { items }
    }

    /// `STREAM_CODEC` — holderSet(ITEM).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Repairable> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &Repairable| {
                holder_set_stream_codec::<Item>().encode(buf, &v.items);
            },
            |buf: &mut FriendlyByteBuf| {
                let items = holder_set_stream_codec::<Item>().decode(buf);
                Repairable::new(items)
            },
        )
    }
}
