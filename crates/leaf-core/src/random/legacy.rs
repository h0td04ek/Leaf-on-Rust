//! `LegacyRandomSource` — java.util.Random-совместимый 48-битный LCG
//! (константы java.util.Random: mul=25214903917, add=11, mask=2^48-1).

use crate::random::bit::{BitRandomSource, Gaussian};
use crate::random::positional::PositionalRandomFactory;
use crate::random::support::generate_unique_seed;
use crate::random::RandomSource;
use crate::util::mth;

const MODULUS_MASK: i64 = 281474976710655; // (1<<48)-1
const MULTIPLIER: i64 = 25214903917;
const INCREMENT: i64 = 11;

#[derive(Debug)]
pub struct LegacyRandomSource {
    seed: i64,
    gaussian: Gaussian,
}

impl LegacyRandomSource {
    pub fn new(seed: i64) -> Self {
        let mut r = Self { seed: 0, gaussian: Gaussian::default() };
        r.set_seed(seed);
        r
    }

    pub fn seed(&self) -> i64 {
        self.seed
    }
}

impl BitRandomSource for LegacyRandomSource {
    fn next(&mut self, size: u32) -> i32 {
        self.seed = self
            .seed
            .wrapping_mul(MULTIPLIER)
            .wrapping_add(INCREMENT)
            & MODULUS_MASK;
        (self.seed >> (48 - size)) as i32
    }

    fn gaussian(&mut self) -> &mut Gaussian {
        &mut self.gaussian
    }
}

impl RandomSource for LegacyRandomSource {
    fn set_seed(&mut self, seed: i64) {
        self.seed = (seed ^ MULTIPLIER) & MODULUS_MASK;
        self.gaussian.reset();
    }

    fn fork(&mut self) -> Box<dyn RandomSource> {
        Box::new(Self::new(BitRandomSource::next_long(self)))
    }

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        Box::new(LegacyPositionalRandomFactory {
            seed: BitRandomSource::next_long(self),
        })
    }

    fn next_int(&mut self) -> i32 {
        BitRandomSource::next_int(self)
    }
    fn next_int_bound(&mut self, bound: i32) -> i32 {
        BitRandomSource::next_int_bound(self, bound)
    }
    fn next_int_range(&mut self, o: i32, b: i32) -> i32 {
        BitRandomSource::next_int_range(self, o, b)
    }
    fn next_long(&mut self) -> i64 {
        BitRandomSource::next_long(self)
    }
    fn next_boolean(&mut self) -> bool {
        BitRandomSource::next_boolean(self)
    }
    fn next_float(&mut self) -> f32 {
        BitRandomSource::next_float(self)
    }
    fn next_double(&mut self) -> f64 {
        BitRandomSource::next_double(self)
    }
    fn next_gaussian(&mut self) -> f64 {
        BitRandomSource::next_gaussian(self)
    }
    fn consume_count(&mut self, count: i32) {
        BitRandomSource::consume_count(self, count)
    }
    fn triangle_f64(&mut self, c: f64, d: f64) -> f64 {
        BitRandomSource::triangle_f64(self, c, d)
    }
    fn triangle_f32(&mut self, c: f32, d: f32) -> f32 {
        BitRandomSource::triangle_f32(self, c, d)
    }
}

/// `LegacyPositionalRandomFactory`.
#[derive(Debug, Clone)]
pub struct LegacyPositionalRandomFactory {
    pub seed: i64,
}

impl PositionalRandomFactory for LegacyPositionalRandomFactory {
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(mth::get_seed(x, y, z) ^ self.seed))
    }

    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(java_hash_code(name) as i64 ^ self.seed))
    }

    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource> {
        Box::new(LegacyRandomSource::new(seed))
    }

    fn parity_config_string(&self) -> String {
        format!("LegacyPositionalRandomFactory{{{}}}", self.seed)
    }
}

/// Java `String.hashCode()` (UTF-16 units, h*31+c).
pub fn java_hash_code(s: &str) -> i32 {
    let mut h: i32 = 0;
    for c in s.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(c as i32);
    }
    h
}

/// `RandomSource.create(seed)` при выключенном FastRNG (дефолт Leaf).
pub fn create(seed: i64) -> Box<dyn RandomSource> {
    Box::new(LegacyRandomSource::new(seed))
}

/// `RandomSource.create()` — уникальный сид.
pub fn create_unique() -> Box<dyn RandomSource> {
    create(generate_unique_seed())
}

/// `RandomSource.createLegacy(seed)`.
pub fn create_legacy(seed: i64) -> Box<dyn RandomSource> {
    create(seed)
}
