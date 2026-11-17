//! Перенос `net.minecraft.resources.ResourceKey<T>`.
//! Интернирование (weak-values map) — TODO: пока обычная структура.

use crate::core::registry::Registry;
use crate::resources::Identifier;
use std::marker::PhantomData;

/// `ResourceKey<T>` — ключ в реестре: (registryName, identifier).
/// Clone/PartialEq/Eq/Hash реализованы вручную — phantom не должен
/// требовать T: Clone/Eq/Hash.
#[derive(Debug)]
pub struct ResourceKey<T> {
    pub registry_name: Identifier,
    pub identifier: Identifier,
    pub _phantom: PhantomData<fn() -> T>,
}

impl<T> ResourceKey<T> {
    /// `ResourceKey.create(registryKey, location)`.
    pub fn create(registry_key: &ResourceKey<Registry<T>>, location: Identifier) -> Self {
        Self::create_with_registry_name(registry_key.identifier.clone(), location)
    }

    /// `createRegistryKey(location)` — ключ самого реестра.
    pub fn create_registry_key(location: Identifier) -> ResourceKey<Registry<T>> {
        ResourceKey {
            registry_name: crate::core::registry::registries::root_registry_name(),
            identifier: location,
            _phantom: PhantomData,
        }
    }

    /// Создание по имени реестра напрямую.
    pub fn create_with_registry_name(registry_name: Identifier, location: Identifier) -> Self {
        Self {
            registry_name,
            identifier: location,
            _phantom: PhantomData,
        }
    }

    pub fn is_for<U>(&self, registry_key: &ResourceKey<Registry<U>>) -> bool {
        self.registry_name == registry_key.identifier
    }

    /// `cast` — проверка реестра с сохранением типа.
    pub fn cast<U>(&self, registry_key: &ResourceKey<Registry<U>>) -> Option<ResourceKey<U>> {
        if self.registry_name == registry_key.identifier {
            Some(ResourceKey {
                registry_name: self.registry_name.clone(),
                identifier: self.identifier.clone(),
                _phantom: PhantomData,
            })
        } else {
            None
        }
    }

    /// `identifier()`
    pub fn id(&self) -> &Identifier {
        &self.identifier
    }

    /// `registry()` — имя реестра.
    pub fn registry(&self) -> &Identifier {
        &self.registry_name
    }

    /// `registryKey()` — ResourceKey<Registry<T>>.
    pub fn registry_key(&self) -> ResourceKey<Registry<T>> {
        ResourceKey::create_registry_key(self.registry_name.clone())
    }
}

impl<T> Clone for ResourceKey<T> {
    fn clone(&self) -> Self {
        Self {
            registry_name: self.registry_name.clone(),
            identifier: self.identifier.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for ResourceKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.registry_name == other.registry_name && self.identifier == other.identifier
    }
}
impl<T> Eq for ResourceKey<T> {}

impl<T> std::hash::Hash for ResourceKey<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.registry_name.hash(state);
        self.identifier.hash(state);
    }
}

impl<T> std::fmt::Display for ResourceKey<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ResourceKey[{} / {}]", self.registry_name, self.identifier)
    }
}
