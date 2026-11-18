//! `HashedStack` — хэшированный ItemStack для `ServerboundContainerClickPacket`.

use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::core::holder::Holder;
use crate::world::item::item::Item;
use crate::world::item::item_stack::ItemStack;
use crate::network::hashed_patch_map::HashedPatchMap;

/// `HashedStack` — `EMPTY` или `ActualItem`.
#[derive(Clone, Debug)]
pub enum HashedStack {
    /// `EMPTY`.
    Empty,
    /// `ActualItem`.
    ActualItem(ActualItem),
}

/// `HashedStack.ActualItem` — `{ item, count, components }`.
#[derive(Clone, Debug)]
pub struct ActualItem {
    /// `item` — `Holder<Item>`.
    pub item: Holder<Item>,
    /// `count`.
    pub count: i32,
    /// `components` — `HashedPatchMap`.
    pub components: HashedPatchMap,
}

impl ActualItem {
    /// `new`.
    pub fn new(item: Holder<Item>, count: i32, components: HashedPatchMap) -> Self {
        Self { item, count, components }
    }

    /// `matches(stack, hashGenerator)`.
    pub fn matches(&self, stack: &ItemStack, hash_generator: &dyn Fn(&crate::core::component::typed_data_component::TypedDataComponent) -> i32) -> bool {
        self.count == stack.get_count()
            && stack.item_holder() == Some(&self.item)
            && self.components.matches(&stack.components.as_patch(), hash_generator)
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(holderRegistry(ITEM), VAR_INT, HashedPatchMap.STREAM_CODEC, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ActualItem> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, item: &ActualItem| {
                crate::world::item::item::stream_codec().encode(buf, &item.item);
                buf.write_var_int(item.count);
                HashedPatchMap::stream_codec().encode(buf, &item.components);
            },
            |buf: &mut FriendlyByteBuf| {
                let item = crate::world::item::item::stream_codec().decode(buf);
                let count = buf.read_var_int();
                let components = HashedPatchMap::stream_codec().decode(buf);
                Self::new(item, count, components)
            },
        )
    }
}

impl HashedStack {
    /// `EMPTY`.
    pub const EMPTY: HashedStack = HashedStack::Empty;

    /// `create(stack, hashGenerator)`.
    pub fn create(stack: &ItemStack, hash_generator: &dyn Fn(&crate::core::component::typed_data_component::TypedDataComponent) -> i32) -> Self {
        if stack.is_empty() {
            HashedStack::Empty
        } else {
            let components = HashedPatchMap::create(&stack.components.as_patch(), hash_generator);
            HashedStack::ActualItem(ActualItem::new(
                stack.item_holder().unwrap().clone(),
                stack.get_count(),
                components,
            ))
        }
    }

    /// `matches(stack, hashGenerator)`.
    pub fn matches(&self, stack: &ItemStack, hash_generator: &dyn Fn(&crate::core::component::typed_data_component::TypedDataComponent) -> i32) -> bool {
        match self {
            HashedStack::Empty => stack.is_empty(),
            HashedStack::ActualItem(item) => item.matches(stack, hash_generator),
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.optional(ActualItem.STREAM_CODEC).map(...)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, HashedStack> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, stack: &HashedStack| {
                match stack {
                    HashedStack::Empty => { buf.write_bool(false); },
                    HashedStack::ActualItem(item) => {
                        buf.write_bool(true);
                        ActualItem::stream_codec().encode(buf, item);
                    },
                }
            },
            |buf: &mut FriendlyByteBuf| {
                if buf.read_bool() {
                    HashedStack::ActualItem(ActualItem::stream_codec().decode(buf))
                } else {
                    HashedStack::Empty
                }
            },
        )
    }
}
