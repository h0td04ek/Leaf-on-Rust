//! `ItemCost` — стоимость в трейде.

use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::core::holder::Holder;
use crate::world::item::item::Item;
use crate::world::item::item_stack::ItemStack;
use crate::core::component::data_component_exact_predicate::DataComponentExactPredicate;

/// `ItemCost` — `{ item, count, components, itemStack }`.
#[derive(Clone, Debug)]
pub struct ItemCost {
    /// `item` — `Holder<Item>`.
    pub item: Holder<Item>,
    /// `count`.
    pub count: i32,
    /// `components`.
    pub components: DataComponentExactPredicate,
    /// `itemStack` — кэшированный `ItemStack`.
    pub item_stack: ItemStack,
}

impl ItemCost {
    /// `new(item, count, components)`.
    pub fn new(item: Holder<Item>, count: i32, components: DataComponentExactPredicate) -> Self {
        let item_stack = Self::create_stack(&item, count, &components);
        Self { item, count, components, item_stack }
    }

    /// `createStack(item, count, components)`.
    fn create_stack(item: &Holder<Item>, count: i32, components: &DataComponentExactPredicate) -> ItemStack {
        ItemStack::with_patch(item.clone(), count, components.as_patch())
    }

    /// `test(stack)` — `stack.is(item) && components.test(stack)`.
    pub fn test(&self, stack: &ItemStack) -> bool {
        stack.item_holder() == Some(&self.item) && self.components.test(stack)
    }

    /// `withComponents`.
    pub fn with_components(&self, f: impl FnOnce(&mut DataComponentExactPredicate) -> DataComponentExactPredicate) -> Self {
        let new_components = f(&mut self.components.clone());
        Self::new(self.item.clone(), self.count, new_components)
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(Item.STREAM_CODEC, VAR_INT, DataComponentExactPredicate.STREAM_CODEC, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemCost> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, cost: &ItemCost| {
                // Item.STREAM_CODEC — holder registry
                crate::world::item::item::stream_codec().encode(buf, &cost.item);
                buf.write_var_int(cost.count);
                DataComponentExactPredicate::stream_codec().encode(buf, &cost.components);
            },
            |buf: &mut FriendlyByteBuf| {
                let item = crate::world::item::item::stream_codec().decode(buf);
                let count = buf.read_var_int();
                let components = DataComponentExactPredicate::stream_codec().decode(buf);
                Self::new(item, count, components)
            },
        )
    }

    /// `OPTIONAL_STREAM_CODEC` — `STREAM_CODEC.apply(ByteBufCodecs::optional)`.
    pub fn optional_stream_codec() -> impl StreamCodec<FriendlyByteBuf, Option<ItemCost>> + Send + Sync + 'static {
        crate::network::codec::byte_buf_codecs::optional(Self::stream_codec())
    }
}
