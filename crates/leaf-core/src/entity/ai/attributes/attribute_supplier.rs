//! Перенос `net.minecraft.world.entity.ai.attributes.AttributeSupplier`.

use super::attribute::Attribute;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::Arc;

/// `AttributeSupplier` — набор атрибутов для типа сущности (singleton/builder).
#[derive(Clone, Default)]
pub struct AttributeSupplier {
    /// `attributes` — id -> `Arc<dyn AttributeLike>`-совместимый `Attribute`.
    /// Храним `Attribute` (конкретный); ranged-атрибуты регистрируются как
    /// `Attribute`-экземпляры с min/max? — в Java все они `Attribute` с
    /// `sanitizeValue` внутри `RangedAttribute`. Здесь `Attribute` содержит
    /// `sanitize_value` — для ranged используем `RangedAttribute` обёртку.
    pub attributes: HashMap<Identifier, Attribute>,
}

impl AttributeSupplier {
    /// `empty()`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `hasAttribute(attribute)`.
    pub fn has_attribute(&self, attribute: &Identifier) -> bool {
        self.attributes.contains_key(attribute)
    }

    /// `getAttribute(id)`.
    pub fn get_attribute(&self, id: &Identifier) -> Option<&Attribute> {
        self.attributes.get(id)
    }

    /// `size()`.
    pub fn size(&self) -> usize {
        self.attributes.len()
    }

    /// `keySet()`.
    pub fn key_set(&self) -> impl Iterator<Item = &Identifier> {
        self.attributes.keys()
    }

    /// `values()`.
    pub fn values(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.values()
    }

    /// `builder()` — builder для создания.
    pub fn builder() -> AttributeSupplierBuilder {
        AttributeSupplierBuilder::default()
    }

    /// `createBuilder()` (в Java `builder()` — нестатический; здесь builder от типа).
    pub fn create_builder_from(&self) -> AttributeSupplierBuilder {
        AttributeSupplierBuilder {
            supplier: self.clone(),
        }
    }
}

impl std::fmt::Debug for AttributeSupplier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttributeSupplier")
            .field("attributes", &self.attributes.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// `AttributeSupplier.Builder`.
#[derive(Clone, Default)]
pub struct AttributeSupplierBuilder {
    /// Накопленный supplier.
    pub supplier: AttributeSupplier,
}

impl AttributeSupplierBuilder {
    /// `add(attribute)` — вставка/переопределение (Java builder кладёт в map
    /// по объекту атрибута — повторный add того же атрибута перезаписывает).
    pub fn add(mut self, attribute: Attribute) -> Self {
        self.supplier.attributes.insert(attribute.id.clone(), attribute);
        self
    }

    /// `add(AttributeSupplier)` — merge.
    pub fn add_supplier(mut self, other: &AttributeSupplier) -> Self {
        for (id, attr) in &other.attributes {
            self.supplier.attributes.insert(id.clone(), attr.clone());
        }
        self
    }

    /// `build()`.
    pub fn build(self) -> AttributeSupplier {
        self.supplier
    }
}

/// `AttributeSupplier` в `Arc` для шаринга между сущностями.
pub type SharedAttributeSupplier = Arc<AttributeSupplier>;

/// Хелпер — создание builder'а с типичным набором.
impl AttributeSupplier {
    /// `create()` — пустой builder.
    pub fn create() -> AttributeSupplierBuilder {
        AttributeSupplierBuilder::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::attribute::Attribute;

    #[test]
    fn test_builder() {
        let supplier = AttributeSupplier::builder()
            .add(Attribute::new(
                Identifier::minecraft("generic.max_health"),
                "attribute.name.generic.max_health",
                20.0,
            ))
            .add(Attribute::new(
                Identifier::minecraft("generic.movement_speed"),
                "attribute.name.generic.movement_speed",
                0.7,
            ))
            .build();
        assert_eq!(supplier.size(), 2);
        assert!(supplier.has_attribute(&Identifier::minecraft("generic.max_health")));
    }
}
