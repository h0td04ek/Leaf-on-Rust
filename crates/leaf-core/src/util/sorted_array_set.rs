//! Перенос `net.minecraft.util.SortedArraySet<T>`.
//! Отсортированное множество на массиве с бинарным поиском.
//! Comparator передаётся как `Arc<dyn Fn(&T,&T)->Ordering>`; для
//! natural order — `T: Ord` через `create_natural`.

use std::cmp::Ordering;
use std::sync::Arc;

/// `SortedArraySet<T>`.
pub struct SortedArraySet<T> {
    comparator: Arc<dyn Fn(&T, &T) -> Ordering + Send + Sync>,
    contents: Vec<T>,
    size: usize,
    /// Leaf-оптимизация: natural order → прямое сравнение без компаратора.
    is_natural_order: bool,
}

const DEFAULT_INITIAL_CAPACITY: usize = 10;

impl<T: Ord + Clone> SortedArraySet<T> {
    /// `create()` — natural ordering, ёмкость 10.
    pub fn create() -> Self {
        Self::create_natural(DEFAULT_INITIAL_CAPACITY)
    }

    /// `create(initialCapacity)`.
    pub fn create_natural(initial_capacity: usize) -> Self {
        Self {
            comparator: Arc::new(|a: &T, b: &T| a.cmp(b)),
            contents: Vec::with_capacity(initial_capacity),
            size: 0,
            is_natural_order: true,
        }
    }
}

impl<T: Clone> SortedArraySet<T> {
    /// `create(comparator)` / `create(comparator, initialCapacity)`.
    pub fn with_comparator(comparator: impl Fn(&T, &T) -> Ordering + Send + Sync + 'static, initial_capacity: usize) -> Self {
        Self {
            comparator: Arc::new(comparator),
            contents: Vec::with_capacity(initial_capacity),
            size: 0,
            is_natural_order: false,
        }
    }

    /// `findIndex` — бинарный поиск; >=0 — найден, <0 — -(insertion+1).
    /// Natural-ветка использует тот же comparator (для natural order он
    /// построен как a.cmp(b) в create_natural) — семантика идентична.
    fn find_index(&self, object: &T) -> isize {
        let mut low = 0isize;
        let mut high = self.size as isize - 1;
        while low <= high {
            let mid = ((low + high) >> 1) as usize;
            let mid_val = &self.contents[mid];
            let cmp = (self.comparator)(mid_val, object);
            match cmp {
                Ordering::Less => low = mid as isize + 1,
                Ordering::Greater => high = mid as isize - 1,
                Ordering::Equal => return mid as isize,
            }
        }
        -(low + 1)
    }

    /// `add(element)` — false если уже есть.
    pub fn add(&mut self, element: T) -> bool
    where
        T: PartialEq,
    {
        let i = self.find_index(&element);
        if i >= 0 {
            return false;
        }
        let insertion = (-i - 1) as usize;
        self.add_internal(element, insertion);
        true
    }

    fn add_internal(&mut self, element: T, index: usize) {
        if self.contents.len() < self.size + 1 {
            let new_cap = if self.size + 1 < 10 { 10 } else { self.size + 1 + (self.size + 1) / 2 };
            self.contents.reserve(new_cap - self.contents.len());
        }
        self.contents.insert(index, element);
        self.size += 1;
    }

    /// `removeInternal(index)`.
    fn remove_internal(&mut self, index: usize) {
        self.contents.remove(index);
        self.size -= 1;
    }

    /// `addOrGet(element)` — возвращает существующий или вставляет.
    pub fn add_or_get(&mut self, element: T) -> &T
    where
        T: PartialEq,
    {
        let i = self.find_index(&element);
        if i >= 0 {
            &self.contents[i as usize]
        } else {
            let insertion = (-i - 1) as usize;
            self.add_internal(element, insertion);
            &self.contents[insertion]
        }
    }

    /// `remove(element)`.
    pub fn remove(&mut self, element: &T) -> bool {
        let i = self.find_index(element);
        if i >= 0 {
            self.remove_internal(i as usize);
            return true;
        }
        false
    }

    /// `get(element)` → Option<&T>.
    pub fn get(&self, element: &T) -> Option<&T> {
        let i = self.find_index(element);
        if i >= 0 { Some(&self.contents[i as usize]) } else { None }
    }

    pub fn first(&self) -> &T {
        &self.contents[0]
    }

    pub fn last(&self) -> &T {
        &self.contents[self.size - 1]
    }

    /// `contains(element)`.
    pub fn contains(&self, element: &T) -> bool {
        self.find_index(element) >= 0
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// `removeIf(predicate)` — сдвиг in-place (Leaf-оптимизация).
    pub fn remove_if<F: FnMut(&T) -> bool>(&mut self, mut filter: F) -> bool {
        let mut i = 0;
        let len = self.size;
        while i < len && !filter(&self.contents[i]) {
            i += 1;
        }
        if i == len {
            return false;
        }
        let mut last_index = i;
        i += 1;
        while i < len {
            if !filter(&self.contents[i]) {
                let v = self.contents[i].clone();
                self.contents[last_index] = v;
                last_index += 1;
            }
            i += 1;
        }
        self.contents.truncate(last_index);
        self.size = last_index;
        true
    }

    /// `moonrise$replace` — заменить или вставить, вернуть старый/новый.
    pub fn replace(&mut self, object: T) -> T
    where
        T: PartialEq,
    {
        let index = self.find_index(&object);
        if index >= 0 {
            let old = std::mem::replace(&mut self.contents[index as usize], object);
            old
        } else {
            let insertion = (-index - 1) as usize;
            self.add_internal(object, insertion);
            self.contents[insertion].clone()
        }
    }

    /// `moonrise$removeAndGet`.
    pub fn remove_and_get(&mut self, object: &T) -> Option<T> {
        let i = self.find_index(object);
        if i >= 0 {
            let ret = self.contents.remove(i as usize);
            self.size -= 1;
            Some(ret)
        } else {
            None
        }
    }

    /// `moonrise$copy`.
    pub fn copy(&self) -> Self
    where
        T: Clone,
    {
        let mut ret = Self::with_comparator2(self.comparator.clone(), 0, self.is_natural_order);
        ret.contents = self.contents[..self.size].to_vec();
        ret.size = self.size;
        ret
    }

    fn with_comparator2(
        comparator: Arc<dyn Fn(&T, &T) -> Ordering + Send + Sync>,
        _cap: usize,
        natural: bool,
    ) -> Self {
        Self { comparator, contents: Vec::new(), size: 0, is_natural_order: natural }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.contents[..self.size].iter()
    }

    pub fn clear(&mut self) {
        self.contents.clear();
        self.size = 0;
    }
}

impl<T: Ord + Clone> Default for SortedArraySet<T> {
    fn default() -> Self {
        Self::create()
    }
}
