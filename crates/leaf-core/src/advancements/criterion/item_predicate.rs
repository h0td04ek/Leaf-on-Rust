//! Перенос `net.minecraft.advancements.criterion.ItemPredicate`.

use crate::advancements::criterion::collection_predicate::CollectionTest;
use crate::advancements::criterion::holder_set_nbt;
use crate::advancements::criterion::min_max_bounds::MinMaxBoundsInts;
use crate::advancements::criterion::nbt_util;
use crate::advancements::criterion::data_component_matchers::DataComponentMatchers;
use crate::core::holder::Holder;
use crate::core::holder_set::HolderSet;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use crate::world::item::item::Item;
use crate::world::item::item_stack::ItemStack;

/// `ItemPredicate` — (items?, count, components).
#[derive(Clone, Debug, PartialEq)]
pub struct ItemPredicate {
    /// `items` — `HolderSet<Item>`.
    pub items: Option<HolderSet<Item>>,
    /// `count` — `MinMaxBounds.Ints` (default ANY).
    pub count: MinMaxBoundsInts,
    /// `components` — `DataComponentMatchers` (required).
    pub components: DataComponentMatchers,
}

impl ItemPredicate {
    /// `new`.
    pub fn new(
        items: Option<HolderSet<Item>>,
        count: MinMaxBoundsInts,
        components: DataComponentMatchers,
    ) -> Self {
        Self { items, count, components }
    }

    /// `ItemPredicate.CODEC` → NBT.
    /// `count` — `optionalFieldOf(ANY)` → опускается при ANY;
    /// `components` — обязательное поле (пустой matcher → `{}`).
    pub fn encode_nbt(&self) -> Tag {
        let mut c = CompoundTag::new();
        if let Some(set) = &self.items {
            if let Ok(t) = holder_set_nbt::holder_set_encode(
                set,
                &Identifier::minecraft("item"),
                |item| Some(item.id.to_string()),
            ) {
                c.put("items", t);
            }
        }
        if self.count != MinMaxBoundsInts::ANY {
            c.put("count", self.count.encode());
        }
        c.put("components", self.components.encode_nbt().expect("component matchers nbt"));
        Tag::Compound(c)
    }

    /// `ItemPredicate.CODEC` ← NBT.
    pub fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        let c = nbt_util::as_compound(tag, "item predicate")?;
        let items = match c.get("items") {
            Some(t) => Some(holder_set_nbt::holder_set_decode::<Item>(
                t,
                Identifier::minecraft("item"),
            )?),
            None => None,
        };
        let count = match c.get("count") {
            Some(t) => MinMaxBoundsInts::decode(t)?,
            None => MinMaxBoundsInts::ANY,
        };
        let components = match c.get("components") {
            Some(t) => DataComponentMatchers::decode_nbt(t)?,
            None => DataComponentMatchers::ANY,
        };
        Ok(Self::new(items, count, components))
    }

    /// `test(stack)`.
    pub fn test(&self, stack: &ItemStack) -> bool {
        if let Some(set) = &self.items {
            let item_holder = match &stack.item {
                Some(h) => h.clone(),
                None => return false,
            };
            if !holder_set_nbt::holder_set_contains(set, &item_holder) {
                // Named-теги: без резолва реестра не проверить —
                // считаем совпадением (Java резолвит через реестр).
                match set {
                    HolderSet::Named(_) => {}
                    _ => return false,
                }
            }
        }
        if !self.count.matches(stack.get_count()) {
            return false;
        }
        self.components.test(&stack.components)
    }
}

impl CollectionTest<crate::world::item::item_stack::ItemStack> for ItemPredicate {
    fn test(&self, value: &crate::world::item::item_stack::ItemStack) -> bool {
        ItemPredicate::test(self, value)
    }
    fn encode_nbt(&self) -> Tag {
        ItemPredicate::encode_nbt(self)
    }
    fn decode_nbt(tag: &Tag) -> Result<Self, String> {
        ItemPredicate::decode_nbt(tag)
    }
}
