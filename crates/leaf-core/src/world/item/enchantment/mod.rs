//! Перенос `net.minecraft.world.item.enchantment`.

pub mod enchantable;
pub mod enchantment;
pub mod item_enchantments;
pub mod repairable;

pub use enchantment::{Enchantment, EnchantmentCost, EnchantmentDefinition};
