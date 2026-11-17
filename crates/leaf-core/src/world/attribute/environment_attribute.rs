//! `net.minecraft.world.attribute.EnvironmentAttribute` —
//! атрибут с типом, дефолтом, диапазоном и флагами.

use super::attribute_type::{attribute_type_by_identifier, AttributeType, AttributeTypeKind};
use super::attribute_value::AttributeValue;
use super::attribute_range::AttributeRange;
use super::attribute_modifier::AttributeModifier;
use crate::resources::identifier::Identifier;
use std::sync::Arc;

/// `EnvironmentAttribute<Value>` — type-erased (`Value` → `AttributeValue`).
#[derive(Clone)]
pub struct EnvironmentAttribute {
    /// Registry-ключ (`minecraft:visual/fog_color`).
    pub id: Identifier,
    /// Тип значения (ссылка на `AttributeType`).
    pub type_: Arc<AttributeType>,
    /// `defaultValue`.
    pub default_value: AttributeValue,
    /// `valueRange` (для float-типов).
    pub value_range: AttributeRange,
    /// `isSyncable`.
    pub is_syncable: bool,
    /// `isPositional`.
    pub is_positional: bool,
    /// `isSpatiallyInterpolated`.
    pub is_spatially_interpolated: bool,
}

impl EnvironmentAttribute {
    /// `type()`.
    pub fn type_(&self) -> &AttributeType {
        &self.type_
    }

    /// `kind()` — вид значения.
    pub fn kind(&self) -> AttributeTypeKind {
        self.type_.kind
    }

    /// `defaultValue()`.
    pub fn default_value(&self) -> &AttributeValue {
        &self.default_value
    }

    /// `sanitizeValue` — клэмп float-значений.
    pub fn sanitize_value(&self, value: AttributeValue) -> AttributeValue {
        if let AttributeValue::Float(f) = value {
            AttributeValue::Float(self.value_range.sanitize(f))
        } else {
            value
        }
    }

    /// `isSyncable`.
    pub fn is_syncable(&self) -> bool {
        self.is_syncable
    }

    /// `isPositional`.
    pub fn is_positional(&self) -> bool {
        self.is_positional
    }

    /// `isSpatiallyInterpolated`.
    pub fn is_spatially_interpolated(&self) -> bool {
        self.is_spatially_interpolated
    }

    /// Проверка типа значения.
    pub fn check_value(&self, value: &AttributeValue) -> Result<(), String> {
        if !self.type_.kind.matches(value) {
            return Err(format!("Value {:?} does not match attribute type {:?}", value, self.type_.kind));
        }
        Ok(())
    }

    /// `checkAllowedModifier` — делегирует в `AttributeType`.
    pub fn check_allowed_modifier(&self, modifier: &AttributeModifier) -> Result<(), String> {
        self.type_.check_allowed_modifier(modifier)
    }
}

impl PartialEq for EnvironmentAttribute {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for EnvironmentAttribute {}
impl std::hash::Hash for EnvironmentAttribute {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl std::fmt::Debug for EnvironmentAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("EnvironmentAttribute").field(&self.id).finish()
    }
}

/// `EnvironmentAttribute.builder(type)` — билдер.
pub struct EnvironmentAttributeBuilder {
    type_: Arc<AttributeType>,
    default_value: Option<AttributeValue>,
    value_range: AttributeRange,
    is_syncable: bool,
    is_positional: bool,
    is_spatially_interpolated: bool,
}

impl EnvironmentAttributeBuilder {
    /// `builder(AttributeType)`.
    pub fn new(type_: Arc<AttributeType>) -> Self {
        Self {
            type_,
            default_value: None,
            value_range: AttributeRange::Any,
            is_syncable: false,
            is_positional: true,
            is_spatially_interpolated: false,
        }
    }

    /// `defaultValue`.
    pub fn default_value(mut self, value: AttributeValue) -> Self {
        self.default_value = Some(value);
        self
    }

    /// `valueRange`.
    pub fn value_range(mut self, range: AttributeRange) -> Self {
        self.value_range = range;
        self
    }

    /// `syncable`.
    pub fn syncable(mut self) -> Self {
        self.is_syncable = true;
        self
    }

    /// `notPositional`.
    pub fn not_positional(mut self) -> Self {
        self.is_positional = false;
        self
    }

    /// `spatiallyInterpolated`.
    pub fn spatially_interpolated(mut self) -> Self {
        self.is_spatially_interpolated = true;
        self
    }

    /// `build` — id проставляется при регистрации в `environment_attributes`.
    pub fn build(mut self, id: Identifier) -> EnvironmentAttribute {
        EnvironmentAttribute {
            id,
            type_: self.type_.clone(),
            default_value: self.default_value.take().expect("Missing default value"),
            value_range: self.value_range,
            is_syncable: self.is_syncable,
            is_positional: self.is_positional,
            is_spatially_interpolated: self.is_spatially_interpolated,
        }
    }
}

/// Удобный конструктор: тип по имени из `attribute_types()`.
pub fn attribute_of_type(kind_name: &str) -> Arc<AttributeType> {
    attribute_type_by_identifier(&Identifier::minecraft(kind_name))
        .unwrap_or_else(|| panic!("unknown attribute type {}", kind_name))
}
