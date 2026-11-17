//! Перенос `net.minecraft.util.ArrayListDeque<T>`.
//! Кольцевой буфер, реализующий и List, и Deque (ListAndDeque).

use std::collections::VecDeque as StdVecDeque;

/// `ArrayListDeque<T>` — List+Deque на кольцевом массиве.
pub struct ArrayListDeque<T> {
    contents: Vec<Option<T>>,
    head: usize,
    size: usize,
}

impl<T: Clone> ArrayListDeque<T> {
    pub fn new() -> Self {
        Self::with_capacity(1)
    }

    pub fn with_capacity(size: usize) -> Self {
        Self { contents: vec![None; size.max(1)], head: 0, size: 0 }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    /// `capacity()` — для тестов.
    pub fn capacity(&self) -> usize {
        self.contents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn get_index(&self, index: usize) -> usize {
        (index + self.head) % self.contents.len()
    }

    fn verify_index_in_range(index: usize, size: usize) {
        assert!(index < size, "index {} out of bounds for size {}", index, size);
    }

    fn verify_index_in_range_self(&self, index: usize) {
        Self::verify_index_in_range(index, self.size);
    }

    /// `get(index)`.
    pub fn get(&self, index: usize) -> T {
        self.verify_index_in_range_self(index);
        self.contents[self.get_index(index)].clone().expect("hole in deque")
    }

    /// `set(index, value)` → старое значение.
    pub fn set(&mut self, index: usize, value: T) -> T {
        self.verify_index_in_range_self(index);
        let index1 = self.get_index(index);
        self.contents[index1].replace(value).expect("hole in deque")
    }

    /// `add(index, element)`.
    pub fn add(&mut self, index: usize, element: T) {
        Self::verify_index_in_range(index, self.size + 1);
        if self.size == self.contents.len() {
            self.grow();
        }
        let index1 = self.get_index(index);
        if index == self.size {
            self.contents[index1] = Some(element);
        } else if index == 0 {
            self.head = (self.head + self.contents.len() - 1) % self.contents.len();
            let idx = self.get_index(0);
            self.contents[idx] = Some(element);
        } else {
            for i in (index..self.size).rev() {
                let dst = self.get_index(i + 1);
                let src = self.get_index(i);
                let v = self.contents[src].take();
                self.contents[dst] = v;
            }
            self.contents[index1] = Some(element);
        }
        self.size += 1;
    }

    fn grow(&mut self) {
        let i = self.contents.len() + (self.contents.len() >> 1).max(1);
        let mut objects = vec![None; i];
        self.copy_count(&mut objects, self.size);
        self.head = 0;
        self.contents = objects;
    }

    fn copy_count(&self, output: &mut [Option<T>], count: usize) {
        for i in 0..count {
            output[i] = self.contents[self.get_index(i)].clone();
        }
    }

    /// `remove(index)` → удалённый элемент.
    pub fn remove(&mut self, index: usize) -> T {
        self.verify_index_in_range_self(index);
        let index1 = self.get_index(index);
        let inner = self.contents[index1].take().expect("hole in deque");
        if index == 0 {
            self.head = (self.head + 1) % self.contents.len();
        } else if index == self.size - 1 {
            // уже None
        } else {
            for i in index + 1..self.size {
                let dst = self.get_index(i - 1);
                let src = self.get_index(i);
                let src_val = self.contents[src].take();
                self.contents[dst] = src_val;
            }
            let last = self.get_index(self.size - 1);
            self.contents[last] = None;
        }
        self.size -= 1;
        inner
    }

    /// `removeIf(predicate)`.
    pub fn remove_if<F: FnMut(&T) -> bool>(&mut self, mut predicate: F) -> bool {
        let mut i = 0usize;
        for i1 in 0..self.size {
            let object = self.get(i1);
            if predicate(&object) {
                i += 1;
            } else if i != 0 {
                let dst = self.get_index(i1 - i);
                self.contents[dst] = Some(object);
                let src = self.get_index(i1);
                self.contents[src] = None;
            }
        }
        self.size -= i;
        i != 0
    }

    /// `replaceAll(operator)`.
    pub fn replace_all<F: FnMut(T) -> T>(&mut self, mut operator: F) {
        for i in 0..self.size {
            let index = self.get_index(i);
            let cur = self.contents[index].take().expect("hole");
            self.contents[index] = Some(operator(cur));
        }
    }

    /// `forEach(action)`.
    pub fn for_each<F: FnMut(&T)>(&self, mut action: F) {
        for i in 0..self.size {
            action(&self.get(i));
        }
    }

    // ---------------- Deque-операции ----------------

    /// `addFirst(element)`.
    pub fn add_first(&mut self, element: T) {
        self.add(0, element);
    }

    /// `addLast(element)`.
    pub fn add_last(&mut self, element: T) {
        self.add(self.size, element);
    }

    /// `offerFirst` — всегда true.
    pub fn offer_first(&mut self, element: T) -> bool {
        self.add_first(element);
        true
    }

    /// `offerLast`.
    pub fn offer_last(&mut self, element: T) -> bool {
        self.add_last(element);
        true
    }

    /// `removeFirst()` — NoSuchElementException → panic.
    pub fn remove_first(&mut self) -> T {
        assert!(self.size != 0, "NoSuchElementException");
        self.remove(0)
    }

    /// `removeLast()`.
    pub fn remove_last(&mut self) -> T {
        assert!(self.size != 0, "NoSuchElementException");
        self.remove(self.size - 1)
    }

    /// `pollFirst()` → Option.
    pub fn poll_first(&mut self) -> Option<T> {
        if self.size == 0 { None } else { Some(self.remove_first()) }
    }

    /// `pollLast()`.
    pub fn poll_last(&mut self) -> Option<T> {
        if self.size == 0 { None } else { Some(self.remove_last()) }
    }

    /// `getFirst()`.
    pub fn get_first(&self) -> T {
        assert!(self.size != 0, "NoSuchElementException");
        self.get(0)
    }

    /// `getLast()`.
    pub fn get_last(&self) -> T {
        assert!(self.size != 0, "NoSuchElementException");
        self.get(self.size - 1)
    }

    /// `peekFirst()`.
    pub fn peek_first(&self) -> Option<T> {
        if self.size == 0 { None } else { Some(self.get_first()) }
    }

    /// `peekLast()`.
    pub fn peek_last(&self) -> Option<T> {
        if self.size == 0 { None } else { Some(self.get_last()) }
    }

    /// `removeFirstOccurrence(element)` — по equals.
    pub fn remove_first_occurrence(&mut self, element: &T) -> bool
    where
        T: PartialEq,
    {
        for i in 0..self.size {
            if &self.get(i) == element {
                self.remove(i);
                return true;
            }
        }
        false
    }

    /// `removeLastOccurrence(element)`.
    pub fn remove_last_occurrence(&mut self, element: &T) -> bool
    where
        T: PartialEq,
    {
        for i in (0..self.size).rev() {
            if &self.get(i) == element {
                self.remove(i);
                return true;
            }
        }
        false
    }

    // ---------------- List-операции ----------------

    /// `indexOf(element)`.
    pub fn index_of(&self, element: &T) -> isize
    where
        T: PartialEq,
    {
        for i in 0..self.size {
            if &self.get(i) == element {
                return i as isize;
            }
        }
        -1
    }

    /// `lastIndexOf(element)`.
    pub fn last_index_of(&self, element: &T) -> isize
    where
        T: PartialEq,
    {
        for i in (0..self.size).rev() {
            if &self.get(i) == element {
                return i as isize;
            }
        }
        -1
    }

    /// `contains(element)`.
    pub fn contains(&self, element: &T) -> bool
    where
        T: PartialEq,
    {
        self.index_of(element) >= 0
    }

    /// `clear()`.
    pub fn clear(&mut self) {
        for slot in &mut self.contents {
            *slot = None;
        }
        self.head = 0;
        self.size = 0;
    }

    /// `iterator()` — прямой порядок.
    pub fn iter(&self) -> ArrayListDequeIter<'_, T> {
        ArrayListDequeIter { deque: self, index: 0, descending: false }
    }

    /// `descendingIterator()`.
    pub fn descending_iter(&self) -> ArrayListDequeIter<'_, T> {
        ArrayListDequeIter { deque: self, index: self.size as isize - 1, descending: true }
    }

    /// `reversed()` — живое представление (в Rust — заимствованный view).
    pub fn reversed(&self) -> ReversedView<'_, T> {
        ReversedView { source: self }
    }

    /// `subList(start, end)` — Java subList; в Rust возвращаем Vec (копия).
    pub fn sub_list(&self, start: usize, end: usize) -> Vec<T> {
        (start..end).map(|i| self.get(i)).collect()
    }
}

/// Итератор ArrayListDeque (прямой и обратный).
pub struct ArrayListDequeIter<'a, T> {
    deque: &'a ArrayListDeque<T>,
    index: isize,
    descending: bool,
}

impl<'a, T: Clone> Iterator for ArrayListDequeIter<'a, T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.descending {
            if self.index < 0 {
                return None;
            }
            let v = self.deque.get(self.index as usize);
            self.index -= 1;
            Some(v)
        } else {
            if self.index as usize >= self.deque.size {
                return None;
            }
            let v = self.deque.get(self.index as usize);
            self.index += 1;
            Some(v)
        }
    }
}

/// `ArrayListDeque.ReversedView` — обратное представление.
pub struct ReversedView<'a, T> {
    source: &'a ArrayListDeque<T>,
}

impl<'a, T: Clone> ReversedView<'a, T> {
    pub fn size(&self) -> usize {
        self.source.size()
    }
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }
    pub fn get_first(&self) -> T {
        self.source.get_last()
    }
    pub fn get_last(&self) -> T {
        self.source.get_first()
    }
    pub fn reversed(&self) -> &'a ArrayListDeque<T> {
        self.source
    }
    fn reverse_index(&self, index: isize) -> isize {
        if index == -1 { -1 } else { self.source.size() as isize - 1 - index }
    }
    pub fn get(&self, index: usize) -> T {
        self.source.get(self.reverse_index(index as isize) as usize)
    }
    pub fn iter(&self) -> ArrayListDequeIter<'a, T> {
        self.source.descending_iter()
    }
}

impl<T: Clone> Default for ArrayListDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Утилита-конвертер в Std VecDeque для интеропа.
impl<T: Clone> From<&ArrayListDeque<T>> for StdVecDeque<T> {
    fn from(d: &ArrayListDeque<T>) -> Self {
        (0..d.size()).map(|i| d.get(i)).collect()
    }
}
