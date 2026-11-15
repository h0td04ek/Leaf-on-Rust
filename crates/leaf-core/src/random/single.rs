//! `SingleThreadedRandomSource` — тот же 48-битный LCG, что и Legacy,
//! но без атомарности (для single-threaded контекстов).

use crate::random::bit::{BitRandomSource, Gaussian};
use crate::random::positional::PositionalRandomFactory;
use crate::random::legacy::LegacyPositionalRandomFactory;
use crate::random::RandomSource;

const MODULUS_MASK: i64 = 281474976710655;
const MULTIPLIER: i64 = 25214903917;
const INCREMENT: i64 = 11;

#[derive(Debug)]
pub struct SingleThreadedRandomSource {
    seed: i64,
    gaussian: Gaussian,
}

impl SingleThreadedRandomSource {
    pub fn new(seed: i64) -> Self {
        let mut r = Self { seed: 0, gaussian: Gaussian::default() };
        r.set_seed(seed);
        r
    }
}

impl BitRandomSource for SingleThreadedRandomSource {
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

impl RandomSource for SingleThreadedRandomSource {
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

/// `RandomSource.createNewThreadLocalInstance()` при выключенном FastRNG.
pub fn create_new_thread_local(seed: i64) -> Box<dyn RandomSource> {
    Box::new(SingleThreadedRandomSource::new(seed))
}
