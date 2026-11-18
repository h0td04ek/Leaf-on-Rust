//! Перенос `net.minecraft.world.entity.ai.attributes.AttributeMap`.

use super::attribute_instance::AttributeInstance;
use super::attribute_supplier::AttributeSupplier;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;

/// `AttributeMap` — набор `AttributeInstance` сущности.
#[derive(Clone)]
pub struct AttributeMap {
    /// `attributes` — id -> instance.
    pub attributes: HashMap<Identifier, AttributeInstance>,
    /// `supplier` — исходный supplier (для `hasAttribute`).
    pub supplier: AttributeSupplier,
}

impl std::fmt::Debug for AttributeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AttributeMap")
            .field("attributes", &self.attributes.keys().collect::<Vec<_>>())
            .finish()
    }
}

impl AttributeMap {
    /// `new(supplier)` — создаёт instance для каждого атрибута supplier'а.
    pub fn new(supplier: AttributeSupplier) -> Self {
        let mut attributes = HashMap::new();
        for (id, attr) in &supplier.attributes {
            attributes.insert(id.clone(), AttributeInstance::new(Box::new(attr.clone())));
        }
        Self { attributes, supplier }
    }

    /// `getInstance(attribute)` — `Option` (в Java бросает NPE, здесь Option).
    pub fn get_instance(&mut self, attribute: &Identifier) -> Option<&mut AttributeInstance> {
        self.attributes.get_mut(attribute)
    }

    /// `getValue(attribute)` — текущее значение (0.0 если отсутствует).
    pub fn get_value(&mut self, attribute: &Identifier) -> f64 {
        self.attributes
            .get_mut(attribute)
            .map(|i| i.get_value())
            .unwrap_or(0.0)
    }

    /// `getValue` read-only (cached_value без dirty-пересчёта).
    pub fn get_value_ro(&self, attribute: &Identifier) -> f64 {
        self.attributes
            .get(attribute)
            .map(|i| i.get_value_ro())
            .unwrap_or(0.0)
    }

    /// `getBaseValue(attribute)`.
    pub fn get_base_value(&mut self, attribute: &Identifier) -> f64 {
        self.attributes
            .get(attribute)
            .map(|i| i.get_base_value())
            .unwrap_or(0.0)
    }

    /// `hasAttribute(attribute)`.
    pub fn has_attribute(&self, attribute: &Identifier) -> bool {
        self.attributes.contains_key(attribute) || self.supplier.has_attribute(attribute)
    }

    /// `addTransientAttributeModifiers(attribute, modifiers)`.
    pub fn add_transient_attribute_modifiers(&mut self, attribute: &Identifier, modifiers: Vec<super::attribute_modifier::AttributeModifier>) {
        if let Some(instance) = self.attributes.get_mut(attribute) {
            for modifier in modifiers {
                instance.add_transient_modifier(modifier);
            }
        }
    }

    /// `removeAttributeModifiers(attribute, modifiers)`.
    pub fn remove_attribute_modifiers(&mut self, attribute: &Identifier, modifiers: &[Identifier]) {
        if let Some(instance) = self.attributes.get_mut(attribute) {
            for id in modifiers {
                instance.remove_modifier(id);
            }
        }
    }

    /// `removeAttribute(attribute, uuid)` — удаление по uuid модификатора.
    /// В этой модели uuid не хранится в modifier — удаляем по id.
    pub fn remove_attribute_modifier_by_id(&mut self, attribute: &Identifier, id: &Identifier) -> bool {
        self.attributes
            .get_mut(attribute)
            .map(|i| i.remove_modifier(id))
            .unwrap_or(false)
    }

    /// `getSyncableAttributes()` — атрибуты для синхронизации с клиентом.
    pub fn get_syncable_attributes(&self) -> Vec<(&Identifier, &AttributeInstance)> {
        self.attributes
            .iter()
            .filter(|(_, i)| i.is_client_syncable())
            .collect()
    }

    /// `save()` — NBT `attributes` список.
    pub fn save_nbt(&self, tag: &mut CompoundTag) {
        let mut list: Vec<Tag> = Vec::new();
        for (id, instance) in &self.attributes {
            let mut m = CompoundTag::new();
            m.put("Name", Tag::String(id.to_string()));
            instance.save_nbt(&mut m);
            list.push(Tag::Compound(m));
        }
        tag.put("attributes", Tag::List(list));
    }

    /// `load(tag)`.
    pub fn load_nbt(&mut self, tag: &CompoundTag) {
        if let Some(list) = tag.get_list("attributes") {
            for item in list {
                if let Tag::Compound(m) = item {
                    let name = m.get_string("Name").unwrap_or_default().to_string();
                    if let Ok(id) = Identifier::parse(&name) {
                        if let Some(instance) = self.attributes.get_mut(&id) {
                            instance.load_nbt(m);
                        }
                    }
                }
            }
        }
    }

    /// `size()`.
    pub fn size(&self) -> usize {
        self.attributes.len()
    }
}

impl AttributeInstance {
    /// `isClientSyncable()` — делегат.
    pub fn is_client_syncable(&self) -> bool {
        self.attribute.is_client_syncable()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::attribute::Attribute;
    use super::super::attribute_modifier::{AttributeModifier, AttributeModifierOperation};

    fn supplier() -> AttributeSupplier {
        AttributeSupplier::builder()
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
            .build()
    }

    #[test]
    fn test_map() {
        let mut map = AttributeMap::new(supplier());
        assert_eq!(map.size(), 2);
        assert_eq!(map.get_value(&Identifier::minecraft("generic.max_health")), 20.0);
        let id = Identifier::minecraft("generic.max_health");
        map.get_instance(&id).unwrap().add_modifier(AttributeModifier::new(
            Identifier::minecraft("test"),
            10.0,
            AttributeModifierOperation::AddValue,
        ));
        assert_eq!(map.get_value(&id), 30.0);
    }
}
