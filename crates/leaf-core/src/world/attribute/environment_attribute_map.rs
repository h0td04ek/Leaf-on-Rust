//! `net.minecraft.world.attribute.EnvironmentAttributeMap` —
//! карта атрибут→модификатор/значение.

use super::attribute_modifier::AttributeModifier;
use super::attribute_value::AttributeValue;
use super::environment_attribute::EnvironmentAttribute;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::Arc;

/// `EnvironmentAttributeMap.Entry` — аргумент + модификатор.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// `argument`.
    pub argument: AttributeValue,
    /// `modifier`.
    pub modifier: AttributeModifier,
}

impl Entry {
    /// `applyModifier(value)`.
    pub fn apply_modifier(&self, value: AttributeValue) -> AttributeValue {
        self.modifier.apply(&value, &self.argument)
    }
}

/// `EnvironmentAttributeMap` — `Map<EnvironmentAttribute, Entry>`.
/// Ключи — по `Identifier` атрибута (реестровая идентичность).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EnvironmentAttributeMap {
    /// `entries` — id атрибута → Entry.
    pub entries: HashMap<Identifier, Entry>,
}

impl EnvironmentAttributeMap {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { entries: HashMap::new() }
    }

    /// `builder`.
    pub fn builder() -> Builder {
        Builder { entries: HashMap::new() }
    }

    /// `get(attribute)`.
    pub fn get(&self, attribute: &EnvironmentAttribute) -> Option<&Entry> {
        self.entries.get(&attribute.id)
    }

    /// `applyModifier(attribute, value)`.
    pub fn apply_modifier(&self, attribute: &EnvironmentAttribute, value: AttributeValue) -> AttributeValue {
        match self.get(attribute) {
            Some(entry) => entry.apply_modifier(value),
            None => value,
        }
    }

    /// `contains(attribute)`.
    pub fn contains(&self, attribute: &EnvironmentAttribute) -> bool {
        self.entries.contains_key(&attribute.id)
    }

    /// `keySet` — идентификаторы атрибутов.
    pub fn key_set(&self) -> impl Iterator<Item = &Identifier> {
        self.entries.keys()
    }

    /// `filterSyncable` — только syncable-атрибуты (для network codec).
    pub fn filter_syncable(&self, attributes: &[Arc<EnvironmentAttribute>]) -> Self {
        let syncable: std::collections::HashSet<&Identifier> = attributes
            .iter()
            .filter(|a| a.is_syncable())
            .map(|a| &a.id)
            .collect();
        Self {
            entries: self
                .entries
                .iter()
                .filter(|(k, _)| syncable.contains(k))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }
}

/// `EnvironmentAttributeMap.Builder`.
#[derive(Default)]
pub struct Builder {
    entries: HashMap<Identifier, Entry>,
}

impl Builder {
    /// `putAll`.
    pub fn put_all(mut self, attributes: EnvironmentAttributeMap) -> Self {
        self.entries.extend(attributes.entries);
        self
    }

    /// `modify(attribute, modifier, argument)`.
    pub fn modify(
        mut self,
        attribute: &EnvironmentAttribute,
        modifier: AttributeModifier,
        argument: AttributeValue,
    ) -> Result<Self, String> {
        attribute.check_allowed_modifier(&modifier)?;
        self.entries.insert(attribute.id.clone(), Entry { argument, modifier });
        Ok(self)
    }

    /// `set(attribute, value)` — `modify(override, value)`.
    pub fn set(self, attribute: &EnvironmentAttribute, value: AttributeValue) -> Result<Self, String> {
        self.modify(attribute, AttributeModifier::Override, value)
    }

    /// `build`.
    pub fn build(self) -> EnvironmentAttributeMap {
        EnvironmentAttributeMap { entries: self.entries }
    }
}
