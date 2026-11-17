//! Перенос `net.minecraft.util.DependencySorter<K, V>`.

use std::collections::{HashMap, HashSet};

/// `DependencySorter.Entry<K>` — запись с зависимостями.
pub trait Entry<K> {
    /// `visitRequiredDependencies` — жёсткие зависимости (без них нельзя).
    fn visit_required_dependencies(&self, visitor: &mut dyn FnMut(K));
    /// `visitOptionalDependencies` — мягкие зависимости (игнорируются при цикле).
    fn visit_optional_dependencies(&self, visitor: &mut dyn FnMut(K));
}

/// `DependencySorter<K, V>` — топологическая сортировка по зависимостям.
pub struct DependencySorter<K, V> {
    contents: HashMap<K, V>,
}

impl<K: Eq + std::hash::Hash + Clone, V: Entry<K>> DependencySorter<K, V> {
    pub fn new() -> Self {
        Self { contents: HashMap::new() }
    }

    /// `addEntry(key, value)`.
    pub fn add_entry(&mut self, key: K, value: V) -> &mut Self {
        self.contents.insert(key, value);
        self
    }

    /// `orderByDependencies(action)` — DFS-топосортировка.
    /// Обязательные зависимости обходятся первыми; опциональные —
    /// только если не образуют цикл.
    pub fn order_by_dependencies(&self, action: &mut dyn FnMut(K, &V)) {
        // dependencies: source -> set of targets (source зависит от target)
        let mut dependencies: HashMap<K, Vec<K>> = HashMap::new();
        for (key, entry) in &self.contents {
            entry.visit_required_dependencies(&mut |dep| {
                Self::add_dependency_if_not_cyclic(&mut dependencies, key.clone(), dep);
            });
        }
        // Второй проход: optional-зависимости (как в Java — отдельный цикл)
        for (key, entry) in &self.contents {
            entry.visit_optional_dependencies(&mut |dep| {
                Self::add_dependency_if_not_cyclic(&mut dependencies, key.clone(), dep);
            });
        }
        let mut visited: HashSet<K> = HashSet::new();
        let keys: Vec<K> = self.contents.keys().cloned().collect();
        for key in keys {
            self.visit_dependencies_and_element(&dependencies, &mut visited, &key, action);
        }
    }

    fn visit_dependencies_and_element(
        &self,
        dependencies: &HashMap<K, Vec<K>>,
        visited: &mut HashSet<K>,
        element: &K,
        action: &mut dyn FnMut(K, &V),
    ) {
        if visited.insert(element.clone()) {
            if let Some(deps) = dependencies.get(element) {
                for dep in deps {
                    self.visit_dependencies_and_element(dependencies, visited, dep, action);
                }
            }
            if let Some(entry) = self.contents.get(element) {
                action(element.clone(), entry);
            }
        }
    }

    /// `isCyclic(dependencies, source, target)` — приведёт ли dep к циклу.
    fn is_cyclic(dependencies: &HashMap<K, Vec<K>>, source: &K, target: &K) -> bool {
        if let Some(collection) = dependencies.get(target) {
            if collection.contains(source) {
                return true;
            }
            for next in collection {
                if Self::is_cyclic(dependencies, source, next) {
                    return true;
                }
            }
        }
        false
    }

    fn add_dependency_if_not_cyclic(dependencies: &mut HashMap<K, Vec<K>>, source: K, target: K) {
        if !Self::is_cyclic(dependencies, &source, &target) {
            dependencies.entry(source).or_default().push(target);
        }
    }
}

impl<K, V> Default for DependencySorter<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Entry<K>,
{
    fn default() -> Self {
        Self::new()
    }
}
