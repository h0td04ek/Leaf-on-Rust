//! Перенос `net.minecraft.core.component.DataComponentMap`.
//!
//! Типизированная карта `DataComponentType -> value`; значения
//! type-erased (`ComponentValue`), доступ — через `get::<T>`.
//! Порядок вставки сохраняется (как `Reference2ObjectArrayMap`).

use crate::core::component::data_component_type::{
    data_component_type_by_identifier, ComponentValue, DataComponentType,
};
use crate::resources::identifier::Identifier;
use std::any::Any;

/// `DataComponentMap` — упорядоченная map компонентов.
#[derive(Clone, Default)]
pub struct DataComponentMap {
    entries: Vec<(Identifier, ComponentValue)>,
}

impl DataComponentMap {
    /// `DataComponentMap.EMPTY`/`builder`.
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// `builder().set(component, value)`.
    pub fn set<T>(&mut self, component: &DataComponentType, value: T)
    where
        T: crate::core::component::data_component_type::ComponentValueData + 'static,
    {
        self.set_boxed(&component.id, Box::new(value));
    }

    /// `setUnchecked` — по registry-ключу типа.
    pub fn set_boxed(&mut self, component_id: &Identifier, value: ComponentValue) {
        if let Some(entry) = self.entries.iter_mut().find(|(id, _)| id == component_id) {
            entry.1 = value;
        } else {
            self.entries.push((component_id.clone(), value));
        }
    }

    /// `get(component)` — типизированный доступ.
    pub fn get<T: 'static>(&self, component: &DataComponentType) -> Option<&T> {
        self.get_by_id(&component.id)
    }

    /// `get` по registry-ключу типа.
    pub fn get_by_id<T: 'static>(&self, component_id: &Identifier) -> Option<&T> {
        self.entries
            .iter()
            .find(|(id, _)| id == component_id)
            .and_then(|(_, v)| v.as_any().downcast_ref::<T>())
    }

    /// `has(component)`.
    pub fn has(&self, component: &DataComponentType) -> bool {
        self.entries.iter().any(|(id, _)| *id == component.id)
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `entrySet` — (type, value) пары.
    pub fn entries(&self) -> &[(Identifier, ComponentValue)] {
        &self.entries
    }

    /// `remove` по registry-ключу типа.
    pub fn remove_by_id(&mut self, component_id: &Identifier) {
        self.entries.retain(|(id, _)| id != component_id);
    }

    /// `forget(predicate)` — удалить типы по предикату на DataComponentType.
    pub fn forget(&mut self, predicate: impl Fn(&DataComponentType) -> bool) {
        self.entries.retain(|(id, _)| {
            data_component_type_by_identifier(id)
                .map(|t| !predicate(&t))
                .unwrap_or(true)
        });
    }

    /// Список установленных типов.
    pub fn set_types(&self) -> Vec<Identifier> {
        self.entries.iter().map(|(id, _)| id.clone()).collect()
    }

    /// Итерация по (Identifier типа, &value as &dyn Any).
    pub fn iter(&self) -> impl Iterator<Item = (&Identifier, &dyn Any)> {
        self.entries
            .iter()
            .map(|(id, v)| (id, v.as_any() as &dyn Any))
    }
}

impl PartialEq for DataComponentMap {
    fn eq(&self, other: &Self) -> bool {
        if self.entries.len() != other.entries.len() {
            return false;
        }
        self.entries.iter().all(|(id, v)| {
            other
                .entries
                .iter()
                .find(|(oid, _)| oid == id)
                .map(|(_, ov)| v.value_eq(ov.as_ref()))
                .unwrap_or(false)
        })
    }
}

impl std::fmt::Debug for DataComponentMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut d = f.debug_map();
        for (id, v) in &self.entries {
            d.entry(&id.to_string(), &v.value_debug());
        }
        d.finish()
    }
}
