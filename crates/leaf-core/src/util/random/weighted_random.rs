//! Перенос `net.minecraft.util.random.WeightedRandom`.

use crate::random::RandomSource;

/// `WeightedRandom` — утилиты для взвешенного случайного выбора.
pub struct WeightedRandom;

impl WeightedRandom {
    /// `getTotalWeight` — сумма весов; > i32::MAX → ошибка.
    pub fn get_total_weight<T>(elements: &[T], weight_getter: impl Fn(&T) -> i32) -> i32 {
        let mut l: i64 = 0;
        for object in elements {
            l += weight_getter(object) as i64;
        }
        if l > i32::MAX as i64 {
            panic!("Sum of weights must be <= {}", i32::MAX);
        }
        l as i32
    }

    /// `getRandomItem(random, elements, totalWeight, weightGetter)` → Option<&T>.
    pub fn get_random_item<'a, T>(
        random: &mut dyn RandomSource,
        elements: &'a [T],
        total_weight: i32,
        weight_getter: impl Fn(&T) -> i32,
    ) -> Option<&'a T> {
        if total_weight < 0 {
            panic!("Negative total weight in getRandomItem");
        }
        if total_weight == 0 {
            return None;
        }
        let random_int = random.next_int_bound(total_weight);
        Self::get_weighted_item(elements, random_int, weight_getter)
    }

    /// `getWeightedItem(elements, index, weightGetter)` → Option<&T>.
    pub fn get_weighted_item<'a, T>(
        elements: &'a [T],
        mut index: i32,
        weight_getter: impl Fn(&T) -> i32,
    ) -> Option<&'a T> {
        for object in elements {
            index -= weight_getter(object);
            if index < 0 {
                return Some(object);
            }
        }
        None
    }

    /// `getRandomItem(random, elements, weightGetter)`.
    pub fn get_random_item_simple<'a, T>(
        random: &mut dyn RandomSource,
        elements: &'a [T],
        weight_getter: impl Fn(&T) -> i32,
    ) -> Option<&'a T> {
        let total = Self::get_total_weight(elements, &weight_getter);
        Self::get_random_item(random, elements, total, weight_getter)
    }
}
