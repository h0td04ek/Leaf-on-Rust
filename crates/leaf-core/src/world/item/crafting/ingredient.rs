//! Перенос `net.minecraft.world.item.crafting.Ingredient`.

use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item::Item;
use crate::world::item::item_stack::ItemStack;
use std::sync::Arc;

/// `Ingredient` — ингредиент рецепта (набор предметов).
#[derive(Clone, Debug)]
pub struct Ingredient {
    /// `values` — HolderSet<Item>.
    pub values: HolderSet<Item>,
}

impl Ingredient {
    /// `new(values)`.
    pub fn new(values: HolderSet<Item>) -> Self {
        Self { values }
    }

    /// `of(item)` — один предмет.
    pub fn of(item: Holder<Item>) -> Self {
        Self { values: HolderSet::direct_one(item) }
    }

    /// `of_items(items)` — несколько предметов.
    pub fn of_items(items: Vec<Holder<Item>>) -> Self {
        Self { values: HolderSet::Direct(items) }
    }

    /// `EMPTY` — пустой ингредиент (для optional).
    pub fn empty() -> Self {
        Self { values: HolderSet::Direct(Vec::new()) }
    }

    /// `test(stack)` — проверяет, подходит ли стек.
    pub fn test(&self, stack: &ItemStack) -> bool {
        // Упрощённо: проверяем, что item holder совпадает
        if let Some(holder) = stack.item_holder() {
            self.values.iter().any(|h| h == holder)
        } else {
            false
        }
    }

    /// `CONTENTS_STREAM_CODEC` — encode.
    pub fn encode(&self, buf: &mut FriendlyByteBuf) {
        crate::core::holder_set::holder_set_stream_codec::<Item>().encode(buf, &self.values);
    }

    /// `CONTENTS_STREAM_CODEC` — decode.
    pub fn decode(buf: &mut FriendlyByteBuf) -> Self {
        let values = crate::core::holder_set::holder_set_stream_codec::<Item>().decode(buf);
        Self { values }
    }

    /// `OPTIONAL_CONTENTS_STREAM_CODEC` — encode Option<Ingredient>.
    pub fn encode_optional(opt: &Option<Ingredient>, buf: &mut FriendlyByteBuf) {
        match opt {
            Some(ing) => {
                buf.write_bool(true);
                ing.encode(buf);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }

    /// `OPTIONAL_CONTENTS_STREAM_CODEC` — decode Option<Ingredient>.
    pub fn decode_optional(buf: &mut FriendlyByteBuf) -> Option<Ingredient> {
        if buf.read_bool() {
            Some(Self::decode(buf))
        } else {
            None
        }
    }
}
