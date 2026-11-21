//! `net.minecraft.world.level.ItemLike`.

use std::sync::Arc;

use crate::world::item::item::Item;

/// `ItemLike` — объект, у которого есть `Item` (Block, ItemStack...).
pub trait ItemLike {
    /// `asItem()`.
    fn as_item(&self) -> Arc<Item>;
}
