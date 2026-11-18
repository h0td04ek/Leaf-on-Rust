//! Перенос `net.minecraft.world.level.storage.loot.IntRange`.

use crate::world::level::storage::loot::loot_context::LootContext;
use crate::world::level::storage::loot::providers::number::number_provider::NumberProvider;
use crate::world::level::storage::loot::providers::number::providers::ConstantValue;

/// `IntRange` — min/max `NumberProvider` диапазон.
/// `None` = отсутствующая граница.
pub struct IntRange {
    /// `min`.
    pub min: Option<Box<dyn NumberProvider>>,
    /// `max`.
    pub max: Option<Box<dyn NumberProvider>>,
}

impl IntRange {
    /// `IntRange.exact(value)`.
    pub fn exact(value: i32) -> Self {
        let c = || Box::new(ConstantValue::exactly(value as f32)) as Box<dyn NumberProvider>;
        Self { min: Some(c()), max: Some(c()) }
    }

    /// `IntRange.range(min, max)` — константные границы.
    pub fn range(min: i32, max: i32) -> Self {
        Self {
            min: Some(Box::new(ConstantValue::exactly(min as f32))),
            max: Some(Box::new(ConstantValue::exactly(max as f32))),
        }
    }

    /// `IntRange.lowerBound(min)`.
    pub fn lower_bound(min: i32) -> Self {
        Self {
            min: Some(Box::new(ConstantValue::exactly(min as f32))),
            max: None,
        }
    }

    /// `IntRange.upperBound(max)`.
    pub fn upper_bound(max: i32) -> Self {
        Self {
            min: None,
            max: Some(Box::new(ConstantValue::exactly(max as f32))),
        }
    }

    /// `IntRange.any()` — обе границы отсутствуют.
    pub fn any() -> Self {
        Self { min: None, max: None }
    }

    /// `clamp(lootContext, value)`.
    pub fn clamp(&self, loot_context: &mut LootContext, value: i32) -> i32 {
        match (&self.min, &self.max) {
            (None, None) => value,
            (Some(min), None) => value.max(min.get_int(loot_context)),
            (None, Some(max)) => value.min(max.get_int(loot_context)),
            (Some(min), Some(max)) => {
                value.clamp(min.get_int(loot_context), max.get_int(loot_context))
            }
        }
    }

    /// `test(lootContext, value)`.
    pub fn test(&self, loot_context: &mut LootContext, value: i32) -> bool {
        match (&self.min, &self.max) {
            (None, None) => true,
            (Some(min), None) => value >= min.get_int(loot_context),
            (None, Some(max)) => value <= max.get_int(loot_context),
            (Some(min), Some(max)) => {
                let min_v = min.get_int(loot_context);
                let max_v = max.get_int(loot_context);
                value >= min_v && value <= max_v
            }
        }
    }

    /// `unpackExact` — `Some` если точное значение.
    pub fn unpack_exact(&self) -> Option<i32> {
        match (&self.min, &self.max) {
            (Some(min), Some(max)) => {
                if let (Some(a), Some(b)) = (min.as_constant(), max.as_constant()) {
                    if (a - b).abs() < f32::EPSILON {
                        return Some(a.round() as i32);
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl Default for IntRange {
    fn default() -> Self {
        Self::any()
    }
}
