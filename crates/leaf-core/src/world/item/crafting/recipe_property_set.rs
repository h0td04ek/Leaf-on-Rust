//! Перенос `net.minecraft.world.item.crafting.RecipePropertySet`.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use crate::world::item::item::Item;
use crate::world::item::item_stack::ItemStack;
use std::collections::HashSet;

/// `RecipePropertySet` — набор предметов для свойств рецепта.
#[derive(Clone, Debug)]
pub struct RecipePropertySet {
    /// `items` — набор Holder<Item>.
    pub items: HashSet<Holder<Item>>,
}

impl RecipePropertySet {
    /// `TYPE_KEY` — ResourceKey для реестра.
    pub const TYPE_KEY: &str = "minecraft:recipe_property_set";

    /// `EMPTY`.
    pub fn empty() -> Self { Self { items: HashSet::new() } }

    /// `new(items)`.
    pub fn new(items: HashSet<Holder<Item>>) -> Self {
        Self { items }
    }

    /// `test(stack)` — проверяет, есть ли предмет в наборе.
    pub fn test(&self, stack: &ItemStack) -> bool {
        if let Some(holder) = stack.item_holder() {
            self.items.contains(holder)
        } else {
            false
        }
    }

    /// `STREAM_CODEC` — encode (список Item holders).
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        let items: Vec<_> = self.items.iter().collect();
        buf.write_collection(&items, |b, h| {
            crate::world::item::item::stream_codec().encode(b, h);
        });
    }

    /// `STREAM_CODEC` — decode.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let items = buf.read_collection(|n| Vec::with_capacity(n), |b| {
            crate::world::item::item::stream_codec().decode(b)
        });
        Self { items: items.into_iter().collect() }
    }
}
