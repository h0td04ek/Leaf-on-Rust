//! Перенос `net.minecraft.util.context.ContextMap` — type-erased
//! хранилище параметров контекста.

use crate::resources::identifier::Identifier;
use crate::util::context::context_key::ContextKey;
use std::any::Any;
use std::collections::HashMap;

/// `ContextMap` — карта `name -> Box<dyn Any>`.
#[derive(Default)]
pub struct ContextMap {
    values: HashMap<Identifier, Box<dyn Any>>,
}

impl ContextMap {
    /// `get(key)` — типизированное чтение.
    pub fn get<T: 'static>(&self, key: &ContextKey<T>) -> Option<&T> {
        self.values
            .get(key.name())
            .and_then(|v| v.downcast_ref::<T>())
    }

    /// `put(key, value)`.
    pub fn put<T: 'static>(&mut self, key: &ContextKey<T>, value: T) {
        self.values.insert(key.name().clone(), Box::new(value));
    }

    /// `has(key)`.
    pub fn has<T: 'static>(&self, key: &ContextKey<T>) -> bool {
        self.get(key).is_some()
    }

    /// `getOptional` по имени (без типа).
    pub fn contains_name(&self, name: &Identifier) -> bool {
        self.values.contains_key(name)
    }
}
