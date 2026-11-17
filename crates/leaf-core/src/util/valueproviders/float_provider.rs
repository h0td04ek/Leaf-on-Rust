//! Перенос `net.minecraft.util.valueproviders.FloatProvider` + `SampledFloat`.

use crate::random::RandomSource;

/// `SampledFloat` — семплер float из RandomSource.
pub trait SampledFloat {
    fn sample(&self, random: &mut dyn RandomSource) -> f32;
}

/// `FloatProvider` — провайдер случайных float.
pub trait FloatProvider: SampledFloat {
    fn min_value(&self) -> f32;
    fn max_value(&self) -> f32;
}
