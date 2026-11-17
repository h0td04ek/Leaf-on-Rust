//! Перенос `net.minecraft.util.valueproviders.TrapezoidFloat`.

use crate::random::RandomSource;
use crate::util::valueproviders::float_provider::{FloatProvider, SampledFloat};

/// `TrapezoidFloat(min, max, plateau)` — трапециевидное распределение.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrapezoidFloat {
    min: f32,
    max: f32,
    plateau: f32,
}

impl TrapezoidFloat {
    pub fn of(min: f32, max: f32, plateau: f32) -> Self {
        Self { min, max, plateau }
    }
}

impl SampledFloat for TrapezoidFloat {
    fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        let f = self.max - self.min;
        let f1 = (f - self.plateau) / 2.0;
        let f2 = f - f1;
        self.min + random.next_float() * f2 + random.next_float() * f1
    }
}

impl FloatProvider for TrapezoidFloat {
    fn min_value(&self) -> f32 {
        self.min
    }
    fn max_value(&self) -> f32 {
        self.max
    }
}

impl std::fmt::Display for TrapezoidFloat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "trapezoid({}) in [{}-{}]", self.plateau, self.min, self.max)
    }
}
