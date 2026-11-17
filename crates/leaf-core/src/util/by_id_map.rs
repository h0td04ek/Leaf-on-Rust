//! Перенос `net.minecraft.util.ByIdMap`.

use crate::util::mth;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutOfBoundsStrategy {
    ZERO,
    WRAP,
    CLAMP,
}

/// `ByIdMap.sparse` — разреженная карта id → значение с fallback.
pub fn sparse<T: Clone>(values: &[(i32, T)], fallback: T) -> impl Fn(i32) -> T + '_ {
    let map: HashMap<i32, T> = values.iter().cloned().collect();
    move |key| map.get(&key).cloned().unwrap_or_else(|| fallback.clone())
}

/// `ByIdMap.continuous` — плотный массив по id с проверкой непрерывности.
pub fn continuous<T: Clone>(
    values: &[(i32, T)],
    strategy: OutOfBoundsStrategy,
) -> impl Fn(i32) -> T + '_ {
    // createSortedArray
    let n = values.len();
    assert!(n > 0, "Empty value list");
    let mut objects: Vec<Option<T>> = vec![None; n];
    for (key, value) in values {
        let i1 = *key;
        assert!((0..n as i32).contains(&i1), "Values are not continous, found index {}", i1);
        assert!(objects[i1 as usize].is_none(), "Duplicate entry on id {}", i1);
        objects[i1 as usize] = Some(value.clone());
    }
    for (i, slot) in objects.iter().enumerate() {
        assert!(slot.is_some(), "Missing value at index: {}", i);
    }
    let objects: Vec<T> = objects.into_iter().map(|o| o.unwrap()).collect();

    let zero = objects[0].clone();
    move |key: i32| -> T {
        let i = n as i32;
        match strategy {
            OutOfBoundsStrategy::ZERO => {
                if key >= 0 && key < i {
                    objects[key as usize].clone()
                } else {
                    zero.clone()
                }
            }
            OutOfBoundsStrategy::WRAP => objects[mth::positive_modulo(key, i) as usize].clone(),
            OutOfBoundsStrategy::CLAMP => objects[mth::clamp_i32(key, 0, i - 1) as usize].clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_fallback() {
        let f = sparse(&[(0, "a"), (2, "c")], "x");
        assert_eq!(f(0), "a");
        assert_eq!(f(1), "x");
        assert_eq!(f(5), "x");
    }

    #[test]
    fn continuous_wrap() {
        let f = continuous(&[(0, 10), (1, 20), (2, 30)], OutOfBoundsStrategy::WRAP);
        assert_eq!(f(0), 10);
        assert_eq!(f(3), 10);
        assert_eq!(f(-1), 30);
    }

    #[test]
    fn continuous_clamp() {
        let f = continuous(&[(0, 10), (1, 20)], OutOfBoundsStrategy::CLAMP);
        assert_eq!(f(-5), 10);
        assert_eq!(f(99), 20);
    }

    #[test]
    #[should_panic(expected = "not continous")]
    fn continuous_out_of_range_panics() {
        let _ = continuous(&[(0, 1), (2, 3)], OutOfBoundsStrategy::ZERO);
    }

    #[should_panic(expected = "Duplicate entry")]
    fn continuous_duplicate_panics() {
        let _ = continuous(&[(0, 1), (1, 2), (1, 3)], OutOfBoundsStrategy::ZERO);
    }
}
