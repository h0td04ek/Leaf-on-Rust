//! `net.minecraft.world.attribute` — система EnvironmentAttribute:
//! типы атрибутов, модификаторы, карты значений, слои и пробники.

pub mod attribute_range;
pub mod attribute_type;
pub mod attribute_modifier;
pub mod lerp_function;
pub mod environment_attribute;
pub mod environment_attribute_map;
pub mod environment_attributes;
pub mod environment_attribute_layer;
pub mod environment_attribute_probe;
pub mod attribute_value;
pub mod bed_rule;

pub use attribute_range::AttributeRange;
pub use attribute_type::{AttributeType, AttributeTypeKind};
pub use attribute_modifier::AttributeModifier;
pub use lerp_function::LerpFunction;
pub use environment_attribute::EnvironmentAttribute;
pub use environment_attribute_map::{EnvironmentAttributeMap, Entry as EnvironmentAttributeEntry};
pub use environment_attributes::environment_attribute_by_identifier;
pub use bed_rule::BedRule;
