//! Перенос `net.minecraft.util.valueproviders.WeightedListInt`.

use crate::random::RandomSource;
use crate::util::random::WeightedList;
use crate::util::valueproviders::int_provider::IntProvider;
use std::sync::Arc;

/// `WeightedListInt(distribution)` — взвешенный выбор IntProvider.
pub struct WeightedListInt {
    distribution: WeightedList<Arc<dyn IntProvider>>,
    min_value: i32,
    max_value: i32,
}

impl WeightedListInt {
    pub fn new(distribution: WeightedList<Arc<dyn IntProvider>>) -> Self {
        let mut min = i32::MAX;
        let mut max = i32::MIN;
        for weighted in distribution.unwrap() {
            min = min.min(weighted.value.min_value());
            max = max.max(weighted.value.max_value());
        }
        Self { distribution, min_value: min, max_value: max }
    }
}

impl IntProvider for WeightedListInt {
    fn sample(&self, random: &mut dyn RandomSource) -> i32 {
        self.distribution.get_random_or_throw(random).sample(random)
    }
    fn min_value(&self) -> i32 {
        self.min_value
    }
    fn max_value(&self) -> i32 {
        self.max_value
    }
    fn describe(&self) -> String {
        "weighted_list".to_string()
    }
}
