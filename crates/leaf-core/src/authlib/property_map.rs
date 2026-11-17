//! Перенос `com.mojang.authlib.properties.PropertyMap` —
//! multimap name → Vec<Property>.

use super::property::Property;
use std::collections::HashMap;

/// `PropertyMap` — свойства профиля (как Guava Multimap).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PropertyMap {
    /// name → свойства с этим именем.
    pub map: HashMap<String, Vec<Property>>,
}

impl PropertyMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// `size()` — количество Property (не ключей).
    pub fn size(&self) -> usize {
        self.map.values().map(|v| v.len()).sum()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// `values()` — все Property.
    pub fn values(&self) -> impl Iterator<Item = &Property> {
        self.map.values().flatten()
    }

    /// `get(name)` — свойства по имени.
    pub fn get(&self, name: &str) -> &[Property] {
        self.map.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// `put` — добавить свойство (создаёт ключ при необходимости).
    pub fn put(&mut self, property: Property) {
        self.map.entry(property.name.clone()).or_default().push(property);
    }

    /// `containsKey`.
    pub fn contains_key(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }
}
