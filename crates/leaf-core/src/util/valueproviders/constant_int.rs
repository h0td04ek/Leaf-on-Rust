//! Перенос `net.minecraft.util.valueproviders.ConstantInt`.

use crate::random::RandomSource;
use crate::util::valueproviders::int_provider::IntProvider;
use std::sync::Arc;

/// `ConstantInt(value)` — константный провайдер.
#[derive(Clone, Debug)]
pub struct ConstantInt {
    value: i32,
}

pub const ZERO: ConstantInt = ConstantInt { value: 0 };

impl ConstantInt {
    /// `of(value)` — 0 → shared ZERO.
    pub fn of(value: i32) -> Self {
        if value == 0 { ZERO } else { Self { value } }
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }
}

impl IntProvider for ConstantInt {
    fn sample(&self, _random: &mut dyn RandomSource) -> i32 {
        self.value
    }
    fn min_value(&self) -> i32 {
        self.value
    }
    fn max_value(&self) -> i32 {
        self.value
    }
    fn describe(&self) -> String {
        self.value.to_string()
    }
}

impl std::fmt::Display for ConstantInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

pub type ConstantIntPtr = Arc<dyn IntProvider>;
