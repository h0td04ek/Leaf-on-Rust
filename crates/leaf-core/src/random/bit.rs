//! `BitRandomSource` — источники на битах (next(size)) с default-методами
//! `RandomSource` (nextInt/nextLong/nextFloat/nextDouble/nextBoolean/nextGaussian).

pub const FLOAT_MULTIPLIER: f32 = 5.9604645e-8;
// Java: `double DOUBLE_MULTIPLIER = 1.110223E-16F` — значение поля double,
// но инициализировано float-литералом → (double)(float)1.110223e-16 = 2^-53.
pub const DOUBLE_MULTIPLIER: f64 = 1.1102230246251565e-16;

/// Marsaglia–Polar gaussian state (`MarsagliaPolarGaussian` поля).
#[derive(Debug, Clone, Copy, Default)]
pub struct Gaussian {
    pub next_next_gaussian: f64,
    pub have_next_next_gaussian: bool,
}

impl Gaussian {
    pub fn reset(&mut self) {
        self.have_next_next_gaussian = false;
    }
}

/// `MarsagliaPolarGaussian.nextGaussian`.
/// d,d1 uniform(-1,1); d2=d²+d1² ∈ (0,1) → return d·s, spare d1·s.
pub fn next_gaussian(state: &mut Gaussian, mut next_double: impl FnMut() -> f64) -> f64 {
    if state.have_next_next_gaussian {
        state.have_next_next_gaussian = false;
        return state.next_next_gaussian;
    }
    loop {
        let d = 2.0 * next_double() - 1.0;
        let d1 = 2.0 * next_double() - 1.0;
        let d2 = d * d + d1 * d1;
        if d2 < 1.0 && d2 != 0.0 {
            let square_root = (-2.0 * d2.ln() / d2).sqrt();
            state.next_next_gaussian = d1 * square_root;
            state.have_next_next_gaussian = true;
            return d * square_root;
        }
    }
}

/// `BitRandomSource` — единственный абстрактный метод `next(size)`
/// плюс доступ к gaussian-состоянию.
pub trait BitRandomSource {
    fn next(&mut self, size: u32) -> i32;
    fn gaussian(&mut self) -> &mut Gaussian;

    fn next_gaussian(&mut self) -> f64 {
        // mem::take выносит состояние, чтобы замыкание могло брать &mut self
        let mut g = std::mem::take(self.gaussian());
        let result = next_gaussian(&mut g, || self.next_double());
        *self.gaussian() = g;
        result
    }

    fn next_int(&mut self) -> i32 {
        self.next(32)
    }

    /// `BitRandomSource.nextInt(bound)` — unbiased-версия (не Lemire).
    fn next_int_bound(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");
        if (bound & (bound - 1)) == 0 {
            return ((bound as i64 * self.next(31) as i64) >> 31) as i32;
        }
        loop {
            let i = self.next(31);
            let i1 = i % bound;
            if i - i1 + (bound - 1) >= 0 {
                return i1;
            }
        }
    }

    fn next_int_range(&mut self, origin: i32, bound: i32) -> i32 {
        assert!(origin < bound, "bound - origin is non positive");
        origin + self.next_int_bound(bound - origin)
    }

    fn next_long(&mut self) -> i64 {
        let i = self.next(32) as i64;
        let i1 = self.next(32) as i64;
        (i << 32).wrapping_add(i1)
    }

    fn next_boolean(&mut self) -> bool {
        self.next(1) != 0
    }

    fn next_float(&mut self) -> f32 {
        self.next(24) as f32 * FLOAT_MULTIPLIER
    }

    fn next_double(&mut self) -> f64 {
        let i = self.next(26) as i64;
        let i1 = self.next(27) as i64;
        let l = (i << 27).wrapping_add(i1);
        l as f64 * DOUBLE_MULTIPLIER
    }

    fn consume_count(&mut self, count: i32) {
        for _ in 0..count {
            self.next_int();
        }
    }

    fn triangle_f64(&mut self, center: f64, max_deviation: f64) -> f64 {
        center + max_deviation * (self.next_double() - self.next_double())
    }

    fn triangle_f32(&mut self, center: f32, max_deviation: f32) -> f32 {
        center + max_deviation * (self.next_float() - self.next_float())
    }
}
