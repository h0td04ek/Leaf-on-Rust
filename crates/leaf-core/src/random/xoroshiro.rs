//! `Xoroshiro128PlusPlus` + `XoroshiroRandomSource` — xoroshiro128++ RNG
//! (worldgen-генератор Minecraft) и `XoroshiroPositionalRandomFactory`.

use crate::random::bit::{next_gaussian, Gaussian, FLOAT_MULTIPLIER};
use crate::random::positional::PositionalRandomFactory;
use crate::random::support::{seed_from_hash_of, upgrade_seed_to_128bit, Seed128bit};
use crate::random::RandomSource;
use crate::util::mth;

/// `Xoroshiro128PlusPlus`.
#[derive(Debug, Clone, Copy)]
pub struct Xoroshiro128PlusPlus {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

impl Xoroshiro128PlusPlus {
    pub fn new(seed: Seed128bit) -> Self {
        Self::with_seeds(seed.seed_lo, seed.seed_hi)
    }

    pub fn with_seeds(seed_lo: i64, seed_hi: i64) -> Self {
        if (seed_lo | seed_hi) == 0 {
            Self {
                seed_lo: -7046029254386353131,
                seed_hi: 7640891576956012809,
            }
        } else {
            Self { seed_lo, seed_hi }
        }
    }

    #[inline]
    pub fn next_long(&mut self) -> i64 {
        let l = self.seed_lo;
        let mut l1 = self.seed_hi;
        let l2 = l
            .wrapping_add(l1)
            .rotate_left(17)
            .wrapping_add(l);
        l1 ^= l;
        self.seed_lo = l.rotate_left(49) ^ l1 ^ (l1 << 21);
        self.seed_hi = l1.rotate_left(28);
        l2
    }
}

/// `XoroshiroRandomSource`.
#[derive(Debug)]
pub struct XoroshiroRandomSource {
    rng: Xoroshiro128PlusPlus,
    gaussian: Gaussian,
}

impl XoroshiroRandomSource {
    pub fn new(seed: i64) -> Self {
        Self {
            rng: Xoroshiro128PlusPlus::new(upgrade_seed_to_128bit(seed)),
            gaussian: Gaussian::default(),
        }
    }

    pub fn with_128(seed: Seed128bit) -> Self {
        Self {
            rng: Xoroshiro128PlusPlus::new(seed),
            gaussian: Gaussian::default(),
        }
    }

    pub fn with_seeds(seed_lo: i64, seed_hi: i64) -> Self {
        Self {
            rng: Xoroshiro128PlusPlus::with_seeds(seed_lo, seed_hi),
            gaussian: Gaussian::default(),
        }
    }

    fn next_bits(&mut self, bits: u32) -> i64 {
        (self.rng.next_long() as u64 >> (64 - bits)) as i64
    }

    /// Lemire nextInt(bound) — `XoroshiroRandomSource.nextInt(int)`.
    fn next_int_l(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "Bound must be positive");
        let bound_l = bound as i64;
        let mut l = self.next_int() as u32 as i64; // toUnsignedLong
        let mut l1 = l.wrapping_mul(bound_l);
        let mut l2 = l1 & 0xFFFF_FFFF;
        if l2 < bound_l {
            // i = Integer.remainderUnsigned(~bound + 1, bound)
            let i = rem_unsigned_i32((!bound).wrapping_add(1), bound);
            while l2 < i as i64 {
                l = self.next_int() as u32 as i64;
                l1 = l.wrapping_mul(bound_l);
                l2 = l1 & 0xFFFF_FFFF;
            }
        }
        (l1 >> 32) as i32
    }
}

/// `Integer.remainderUnsigned(x, y)` для i32.
fn rem_unsigned_i32(x: i32, y: i32) -> i32 {
    let ux = x as u32;
    let uy = y as u32;
    (ux % uy) as i32
}

impl RandomSource for XoroshiroRandomSource {
    fn set_seed(&mut self, seed: i64) {
        self.rng = Xoroshiro128PlusPlus::new(upgrade_seed_to_128bit(seed));
        self.gaussian.reset();
    }

    fn fork(&mut self) -> Box<dyn RandomSource> {
        let lo = self.rng.next_long();
        let hi = self.rng.next_long();
        Box::new(Self::with_seeds(lo, hi))
    }

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        Box::new(XoroshiroPositionalRandomFactory {
            seed_lo: self.rng.next_long(),
            seed_hi: self.rng.next_long(),
        })
    }

    fn next_int(&mut self) -> i32 {
        self.rng.next_long() as i32
    }

    fn next_int_bound(&mut self, bound: i32) -> i32 {
        self.next_int_l(bound)
    }

    fn next_int_range(&mut self, origin: i32, bound: i32) -> i32 {
        // RandomSource default: nextInt(bound-origin)+origin — но у Xoroshiro
        // nextInt(bound) — Lemire; диапазон тоже через Lemire? В Java
        // nextInt(origin,bound) — default метод RandomSource →
        // nextInt(bound-origin)+origin → Lemire. ОК.
        assert!(origin < bound, "bound - origin is non positive");
        origin + self.next_int_l(bound - origin)
    }

    fn next_long(&mut self) -> i64 {
        self.rng.next_long()
    }

    fn next_boolean(&mut self) -> bool {
        self.rng.next_long() & 1 != 0
    }

    fn next_float(&mut self) -> f32 {
        self.next_bits(24) as f32 * FLOAT_MULTIPLIER
    }

    fn next_double(&mut self) -> f64 {
        // Java: `nextBits(53) * 1.110223E-16F` — long*float → float-умножение,
        // результат расширяется до double (реальная потеря точности в MC).
        (self.next_bits(53) as f32 * 1.1102230246251565e-16f32) as f64
    }

    fn next_gaussian(&mut self) -> f64 {
        let mut g = std::mem::take(&mut self.gaussian);
        let result = next_gaussian(&mut g, || self.next_double());
        self.gaussian = g;
        result
    }

    fn consume_count(&mut self, count: i32) {
        for _ in 0..count {
            self.rng.next_long();
        }
    }
}

/// `XoroshiroPositionalRandomFactory`.
#[derive(Debug, Clone)]
pub struct XoroshiroPositionalRandomFactory {
    pub seed_lo: i64,
    pub seed_hi: i64,
}

impl PositionalRandomFactory for XoroshiroPositionalRandomFactory {
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource> {
        Box::new(XoroshiroRandomSource::with_seeds(
            mth::get_seed(x, y, z) ^ self.seed_lo,
            self.seed_hi,
        ))
    }

    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource> {
        let seed = seed_from_hash_of(name).xor_lo_hi(self.seed_lo, self.seed_hi);
        Box::new(XoroshiroRandomSource::with_128(seed))
    }

    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource> {
        Box::new(XoroshiroRandomSource::with_seeds(
            seed ^ self.seed_lo,
            seed ^ self.seed_hi,
        ))
    }

    fn parity_config_string(&self) -> String {
        format!("seedLo: {}, seedHi: {}", self.seed_lo, self.seed_hi)
    }
}


