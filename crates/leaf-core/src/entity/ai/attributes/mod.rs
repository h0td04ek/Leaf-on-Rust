//! Перенос `net.minecraft.world.entity.ai.attributes`.

pub mod attribute;
pub mod attribute_instance;
pub mod attribute_map;
pub mod attribute_modifier;
pub mod attribute_supplier;
pub mod attributes;
pub mod default_attributes;
pub mod ranged_attribute;

pub use attribute::Attribute;
pub use attribute_instance::AttributeInstance;
pub use attribute_map::AttributeMap;
pub use attribute_modifier::{AttributeModifier, AttributeModifierOperation};
pub use attribute_supplier::{AttributeSupplier, AttributeSupplierBuilder};
pub use attributes as attributes_const;
pub use ranged_attribute::RangedAttribute;
