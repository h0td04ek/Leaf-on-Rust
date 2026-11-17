//! Перенос `net.minecraft.util.SmoothDouble`.

use crate::util::mth;

/// `SmoothDouble` — сглаживание целевого значения (используется в партиклах).
#[derive(Default)]
pub struct SmoothDouble {
    target_value: f64,
    remaining_value: f64,
    last_amount: f64,
}

impl SmoothDouble {
    pub fn new() -> Self {
        Self::default()
    }

    /// `getNewDeltaValue(input, multiplier)`.
    pub fn get_new_delta_value(&mut self, input: f64, multiplier: f64) -> f64 {
        self.target_value += input;
        let d = self.target_value - self.remaining_value;
        let d1 = mth::lerp_f64(0.5, self.last_amount, d);
        let d2 = d.signum();
        let d = if d2 * d > d2 * self.last_amount { d1 } else { d };
        self.last_amount = d1;
        self.remaining_value += d * multiplier;
        d * multiplier
    }

    pub fn reset(&mut self) {
        self.target_value = 0.0;
        self.remaining_value = 0.0;
        self.last_amount = 0.0;
    }
}
