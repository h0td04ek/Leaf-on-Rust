//! Перенос `net.minecraft.util.valueproviders.SampledFloat` реализаторов без
//! min/max (`MultipliedFloats`).

use crate::random::RandomSource;
use crate::util::valueproviders::float_provider::SampledFloat;
use std::sync::Arc;

/// `MultipliedFloats(values...)` — произведение семплов.
#[derive(Clone)]
pub struct MultipliedFloats {
    values: Vec<Arc<dyn SampledFloat>>,
}

impl MultipliedFloats {
    pub fn new(values: Vec<Arc<dyn SampledFloat>>) -> Self {
        Self { values }
    }
}

impl SampledFloat for MultipliedFloats {
    fn sample(&self, random: &mut dyn RandomSource) -> f32 {
        let mut f = 1.0;
        for sampled_float in &self.values {
            f *= sampled_float.sample(random);
        }
        f
    }
}

impl std::fmt::Debug for MultipliedFloats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MultipliedFloats(len={})", self.values.len())
    }
}
