//! Перенос `net.minecraft.util.context.ContextKey`.

use crate::resources::identifier::Identifier;
use std::marker::PhantomData;

/// `ContextKey<T>` — типизированный ключ параметра контекста.
/// Значение хранится в `ContextMap` как `Box<dyn Any>`.
#[derive(Debug)]
pub struct ContextKey<T> {
    name: Identifier,
    marker: PhantomData<fn() -> T>,
}

impl<T> ContextKey<T> {
    /// `new ContextKey(name)` / `ContextKey.vanilla(name)`.
    pub fn new(name: Identifier) -> Self {
        Self { name, marker: PhantomData }
    }

    /// `ContextKey.vanilla(name)` — `minecraft:` namespace.
    pub fn vanilla(name: &str) -> Self {
        Self::new(Identifier::with_default_namespace(name).expect("context key id"))
    }

    /// `name()`.
    pub fn name(&self) -> &Identifier {
        &self.name
    }
}

impl<T> PartialEq for ContextKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
impl<T> Eq for ContextKey<T> {}
impl<T> std::hash::Hash for ContextKey<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
