//! Перенос `net.minecraft.world.item.ItemStack`.
//!
//! Модель: `item: Option<Holder<Item>>` (None = EMPTY-item),
//! `count`, `components: PatchedDataComponentMap` (prototype + patch).
//!
//! Wire-кодеки (`ItemStack.java`):
//! - `OPTIONAL_STREAM_CODEC`: VarInt count (<=0 → EMPTY), затем
//!   `Item.STREAM_CODEC` (holderRegistry) + `DataComponentPatch.STREAM_CODEC`.
//! - `OPTIONAL_UNTRUSTED_STREAM_CODEC`: то же, но patch —
//!   `DELIMITED_STREAM_CODEC` (length-prefixed значения).
//! - `STREAM_CODEC`: optional + запрет пустого стека.

use crate::core::component::{DataComponentPatch, PatchedDataComponentMap};
use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::item;
use std::hash::{Hash, Hasher};

/// `ItemStack` — стопка предметов.
#[derive(Clone, Debug)]
pub struct ItemStack {
    /// `item` — holder предмета (`None` только у EMPTY-подобных).
    pub item: Option<Holder<item::Item>>,
    /// `count`.
    pub count: i32,
    /// `components` — `PatchedDataComponentMap` (prototype + patch).
    pub components: PatchedDataComponentMap,
}

impl ItemStack {
    /// `ItemStack.EMPTY`.
    pub fn empty() -> Self {
        Self { item: None, count: 0, components: PatchedDataComponentMap::empty() }
    }

    /// `new(itemHolder, count)` — с пустым prototype компонентов.
    pub fn new(item: Holder<item::Item>, count: i32) -> Self {
        Self {
            item: Some(item),
            count,
            components: PatchedDataComponentMap::empty(),
        }
    }

    /// `new(item, count, patch)` — Java-конструктор из optional-кодека.
    pub fn with_patch(item: Holder<item::Item>, count: i32, patch: DataComponentPatch) -> Self {
        Self {
            item: Some(item),
            count,
            components: PatchedDataComponentMap::from_patch(
                Default::default(),
                &patch,
            ),
        }
    }

    /// `isEmpty` — `count <= 0` или нет предмета.
    pub fn is_empty(&self) -> bool {
        self.count <= 0 || self.item.is_none()
    }

    /// `getCount`.
    pub fn get_count(&self) -> i32 {
        self.count
    }

    /// `getItem` — registry-ключ предмета (для Direct — его id,
    /// для Reference — по реестру).
    pub fn item_identifier(&self) -> Option<crate::resources::identifier::Identifier> {
        match self.item.as_ref()? {
            Holder::Direct(i) => Some(i.id.clone()),
            Holder::Reference(id) => item::item_by_id(*id).map(|i| i.id.clone()),
            Holder::Key(k) => Some(k.identifier.clone()),
        }
    }

    /// `getItemHolder`.
    pub fn item_holder(&self) -> Option<&Holder<item::Item>> {
        self.item.as_ref()
    }

    /// `copy`.
    pub fn copy(&self) -> ItemStack {
        self.clone()
    }

    /// `getHoverName()` — имя для подсказки (custom_name компонент или item id).
    pub fn get_hover_name(&self) -> crate::network::chat::component::Component {
        let id = crate::resources::identifier::Identifier::parse("custom_name")
            .expect("custom_name id");
        if let Some(c) = self.components.get_by_id::<crate::network::chat::component::Component>(&id) {
            return c.clone();
        }
        match self.item_holder().and_then(|h| h.key().map(|k| k.id().clone())) {
            Some(id) => crate::network::chat::component::Component::translatable(
                format!("item.{}.{}", id.get_namespace(), id.get_path()),
                vec![],
            ),
            None => crate::network::chat::component::Component::literal("item"),
        }
    }

    /// `getDisplayName()` — синоним `getHoverName`.
    pub fn get_display_name(&self) -> crate::network::chat::component::Component {
        self.get_hover_name()
    }

    /// `getMaxStackSize()` — `item.getMaxStackSize()`:
    /// `MAX_STACK_SIZE` компонент предмета, default 64.
    pub fn get_max_stack_size(&self) -> i32 {
        let id = crate::resources::identifier::Identifier::parse("max_stack_size")
            .expect("max_stack_size id");
        if let Some(max) = self.components.get_by_id::<i32>(&id) {
            return *max;
        }
        64
    }

    /// `matches(a, b)` — одинаковый item и count (без компонентов).
    pub fn matches(a: &ItemStack, b: &ItemStack) -> bool {
        a.item == b.item && a.count == b.count
    }

    /// `hashItemAndComponents`.
    pub fn hash_item_and_components(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.item.hash(&mut h);
        self.count.hash(&mut h);
        for c in self.components.iter() {
            c.component_type.id.hash(&mut h);
            c.value.type_id().hash(&mut h);
        }
        h.finish()
    }

    /// Общий decode optional-кодека.
    fn decode_optional(buf: &mut FriendlyByteBuf, delimited: bool) -> ItemStack {
        let count = buf.read_var_int();
        if count <= 0 {
            return ItemStack::empty();
        }
        let item_holder = item::stream_codec().decode(buf);
        let patch = if delimited {
            DataComponentPatch::delimited_stream_codec().decode(buf)
        } else {
            DataComponentPatch::stream_codec().decode(buf)
        };
        ItemStack::with_patch(item_holder, count, patch)
    }

    /// Общий encode optional-кодека.
    fn encode_optional(stack: &ItemStack, buf: &mut FriendlyByteBuf, delimited: bool) {
        if stack.is_empty() {
            buf.write_var_int(0);
            return;
        }
        buf.write_var_int(stack.count);
        item::stream_codec().encode(buf, stack.item.as_ref().expect("item"));
        let patch = stack.components.as_patch();
        if delimited {
            DataComponentPatch::delimited_stream_codec().encode(buf, &patch);
        } else {
            DataComponentPatch::stream_codec().encode(buf, &patch);
        }
    }

    /// `OPTIONAL_STREAM_CODEC`.
    pub fn optional_stream_codec(
    ) -> impl StreamCodec<FriendlyByteBuf, ItemStack> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, s: &ItemStack| {
                Self::encode_optional(s, buf, false);
            },
            |buf: &mut FriendlyByteBuf| Self::decode_optional(buf, false),
        )
    }

    /// `OPTIONAL_UNTRUSTED_STREAM_CODEC` — delimited patch.
    pub fn optional_untrusted_stream_codec(
    ) -> impl StreamCodec<FriendlyByteBuf, ItemStack> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, s: &ItemStack| {
                Self::encode_optional(s, buf, true);
            },
            |buf: &mut FriendlyByteBuf| Self::decode_optional(buf, true),
        )
    }

    /// `STREAM_CODEC` — optional + non-empty.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ItemStack> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, s: &ItemStack| {
                if s.is_empty() {
                    panic!("Empty ItemStack not allowed");
                }
                Self::encode_optional(s, buf, false);
            },
            |buf: &mut FriendlyByteBuf| {
                let s = Self::decode_optional(buf, false);
                if s.is_empty() {
                    panic!("Empty ItemStack not allowed");
                }
                s
            },
        )
    }

    /// `OPTIONAL_LIST_STREAM_CODEC`.
    pub fn optional_list_stream_codec(
    ) -> impl StreamCodec<FriendlyByteBuf, Vec<ItemStack>> + Send + Sync + 'static {
        crate::network::codec::byte_buf_codecs::list(Self::optional_stream_codec())
    }
}

impl PartialEq for ItemStack {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item && self.count == other.count
    }
}

impl crate::core::component::data_component_getter::DataComponentGetter for ItemStack {
    fn get_raw(&self, component: &crate::core::component::data_component_type::DataComponentType) -> Option<crate::core::component::data_component_type::ComponentValue> {
        self.components.effective_raw(&component.id)
    }

    fn has_component(&self, component: &crate::core::component::data_component_type::DataComponentType) -> bool {
        self.components.has(component)
    }

    fn iter_raw(&self) -> Vec<(crate::resources::identifier::Identifier, Option<crate::core::component::data_component_type::ComponentValue>)> {
        self.components
            .iter()
            .into_iter()
            .map(|t| (t.component_type.id.clone(), Some(t.value)))
            .collect()
    }
}
