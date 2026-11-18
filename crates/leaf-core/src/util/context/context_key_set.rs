//! Перенос `net.minecraft.util.context.ContextKeySet`.

use crate::resources::identifier::Identifier;
use crate::util::context::context_key::ContextKey;
use std::collections::HashSet;

/// `ContextKeySet` — множество допустимых ключей параметров
/// для конкретного вида контекста (loot param set).
#[derive(Clone, Debug, Default)]
pub struct ContextKeySet {
    required: HashSet<Identifier>,
    optional: HashSet<Identifier>,
}

impl ContextKeySet {
    /// `Builder`-эквивалент: конструирование набора.
    pub fn builder() -> ContextKeySetBuilder {
        ContextKeySetBuilder::default()
    }

    /// `allows(key)` — ключ допустим в наборе.
    pub fn allows(&self, key_name: &Identifier) -> bool {
        self.required.contains(key_name) || self.optional.contains(key_name)
    }

    /// `requiredKeys`/`optionalKeys` — имена ключей.
    pub fn required_keys(&self) -> &HashSet<Identifier> {
        &self.required
    }
    pub fn optional_keys(&self) -> &HashSet<Identifier> {
        &self.optional
    }
}

/// `ContextKeySet.Builder`.
#[derive(Default)]
pub struct ContextKeySetBuilder {
    required: Vec<Identifier>,
    optional: Vec<Identifier>,
}

impl ContextKeySetBuilder {
    /// `required(key)`.
    pub fn required<T>(&mut self, key: &ContextKey<T>) -> &mut Self {
        self.required.push(key.name().clone());
        self
    }
    /// `optional(key)`.
    pub fn optional<T>(&mut self, key: &ContextKey<T>) -> &mut Self {
        self.optional.push(key.name().clone());
        self
    }
    /// `build`.
    pub fn build(self) -> ContextKeySet {
        ContextKeySet {
            required: self.required.into_iter().collect(),
            optional: self.optional.into_iter().collect(),
        }
    }
}
