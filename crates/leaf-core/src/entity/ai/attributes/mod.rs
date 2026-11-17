//! Перенос `net.minecraft.world.entity.ai.attributes`.

pub mod attribute;
pub mod attribute_modifier;

pub use attribute::Attribute;
pub use attribute_modifier::{AttributeModifier, AttributeModifierOperation};
