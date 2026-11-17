//! Перенос `net.minecraft.world.item.ItemStack` — минимальная модель
//! (item как registry-идентификатор; реестр Item/DataComponents
//! портируется отдельно, структура расширится).

use crate::nbt::CompoundTag;
use crate::resources::identifier::Identifier;
use std::hash::{Hash, Hasher};

/// `ItemStack` — стопка предметов.
#[derive(Clone, Debug, PartialEq)]
pub struct ItemStack {
    /// Registry-ключ предмета (`minecraft:stone` и т.п.).
    /// При порте `Item`-регистра заменится на handle.
    pub item: Identifier,
    /// `count` — количество (1..=max_stack_size).
    pub count: i32,
    /// Компоненты данных (snapshot NBT; DataComponentPatch — позже).
    pub components: Option<CompoundTag>,
}

impl ItemStack {
    pub fn new(item: Identifier, count: i32) -> Self {
        Self { item, count, components: None }
    }

    /// `isEmpty` — count <= 0.
    pub fn is_empty(&self) -> bool {
        self.count <= 0
    }

    /// `getCount`.
    pub fn get_count(&self) -> i32 {
        self.count
    }

    /// `getItem`.
    pub fn get_item(&self) -> &Identifier {
        &self.item
    }

    /// `copy`.
    pub fn copy(&self) -> ItemStack {
        self.clone()
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
        if let Some(c) = &self.components {
            format!("{:?}", c).hash(&mut h);
        }
        h.finish()
    }
}
