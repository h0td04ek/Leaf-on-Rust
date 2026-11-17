//! Перенос `net.minecraft.util.random.Weighted<T>` (record).

use std::fmt;

/// `Weighted(value, weight)` — элемент с весом для взвешенного выбора.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Weighted<T> {
    pub value: T,
    pub weight: i32,
}

impl<T> Weighted<T> {
    /// `new Weighted(value, weight)` — вес < 0 → IllegalArgumentException.
    pub fn new(value: T, weight: i32) -> Self {
        assert!(weight >= 0, "Weight should be >= 0");
        Self { value, weight }
    }

    /// `value()`
    pub fn value(&self) -> &T {
        &self.value
    }

    /// `weight()`
    pub fn weight(&self) -> i32 {
        self.weight
    }

    /// `map(Function)`
    pub fn map<U>(&self, mapper: impl Fn(&T) -> U) -> Weighted<U> {
        Weighted::new(mapper(&self.value), self.weight)
    }
}

impl<T: fmt::Display> fmt::Display for Weighted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.weight, self.value)
    }
}
