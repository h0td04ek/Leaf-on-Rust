//! Перенос `net.minecraft.util.ClassInstanceMultiMap`.
//! Java-версия лениво строит индексы по Class<?>; в Rust `find_typed::<S>()`
//! фильтрует all_instances через Any::downcast_ref — семантика та же,
//! кэш индексов не портирован (при необходимости добавить позже).

use std::any::Any;

/// Мультимапа экземпляров с поиском по типу (используется в entity lookups).
pub struct ClassInstanceMultiMap<T: 'static> {
    all_instances: Vec<Box<dyn Any>>,
    _phantom: std::marker::PhantomData<fn() -> T>,
}

impl<T: 'static> ClassInstanceMultiMap<T> {
    /// `new ClassInstanceMultiMap(Class<T> baseClass)`.
    pub fn new() -> Self {
        Self {
            all_instances: Vec::new(),
            _phantom: std::marker::PhantomData,
        }
    }

    /// `add` — всегда true при успешном добавлении.
    pub fn add(&mut self, value: T) -> bool {
        self.all_instances.push(Box::new(value));
        true
    }

    /// `remove(Object)` — без PartialEq у T удаление по значению невозможно;
    /// используйте `retain` с предикатом или `remove_at`.
    pub fn remove_at(&mut self, index: usize) -> Option<T> {
        self.all_instances.remove(index).downcast::<T>().ok().map(|b| *b)
    }

    /// `find(Class<S>)` — все экземпляры типа S.
    pub fn find_typed<S: 'static>(&self) -> Vec<&S> {
        self.all_instances
            .iter()
            .filter_map(|b| b.downcast_ref::<S>())
            .collect()
    }

    /// Все экземпляры базового типа T.
    pub fn iter_all(&self) -> impl Iterator<Item = &T> {
        self.all_instances.iter().filter_map(|b| b.downcast_ref::<T>())
    }

    /// `getAllInstances()`
    pub fn get_all_instances(&self) -> Vec<&T> {
        self.iter_all().collect()
    }

    /// `size()`
    pub fn size(&self) -> usize {
        self.all_instances.len()
    }

    /// `contains` по типу+значению (ptr-эквивалентность).
    pub fn contains_typed<S: 'static>(&self, value: &S) -> bool {
        self.find_typed::<S>().iter().any(|v| std::ptr::eq(*v, value))
    }

    /// Удалить все, для которых предикат false (замена remove по значению).
    pub fn retain(&mut self, f: impl Fn(&T) -> bool) {
        self.all_instances.retain(|b| {
            b.downcast_ref::<T>().map(|t| f(t)).unwrap_or(false)
        });
    }
}

impl<T: 'static> Default for ClassInstanceMultiMap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> IntoIterator for ClassInstanceMultiMap<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.all_instances
            .into_iter()
            .filter_map(|b| b.downcast::<T>().ok().map(|b| *b))
            .collect::<Vec<_>>()
            .into_iter()
    }
}
