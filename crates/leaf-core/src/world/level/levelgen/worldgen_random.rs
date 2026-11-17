//! `net.minecraft.world.level.levelgen.WorldgenRandom` —
//! RNG-обёртка для worldgen с сид-методами декораций.

use crate::random::legacy::LegacyRandomSource;
use crate::random::{RandomSource, XoroshiroRandomSource};
use crate::random::positional::PositionalRandomFactory;
use std::sync::atomic::{AtomicI32, Ordering};

/// `WorldgenRandom extends LegacyRandomSource` — делегирует
/// `next(bits)` внутреннему RandomSource.
pub struct WorldgenRandom {
    /// Внутренний источник (в Java — поле `randomSource`).
    inner: Box<dyn RandomSource>,
    /// `count` — число вызовов `next`.
    count: AtomicI32,
}

impl WorldgenRandom {
    /// `new WorldgenRandom(randomSource)`.
    pub fn new(random_source: Box<dyn RandomSource>) -> Self {
        Self { inner: random_source, count: AtomicI32::new(0) }
    }

    /// `getCount`.
    pub fn get_count(&self) -> i32 {
        self.count.load(Ordering::Relaxed)
    }

    /// `setDecorationSeed`.
    pub fn set_decoration_seed(&mut self, level_seed: i64, min_chunk_block_x: i32, min_chunk_block_z: i32) -> i64 {
        self.set_seed(level_seed);
        let l = self.next_long() | 1;
        let l1 = self.next_long() | 1;
        let l2 = (min_chunk_block_x as i64).wrapping_mul(l)
            .wrapping_add((min_chunk_block_z as i64).wrapping_mul(l1))
            ^ level_seed;
        self.set_seed(l2);
        l2
    }

    /// `setFeatureSeed`.
    pub fn set_feature_seed(&mut self, decoration_seed: i64, index: i32, decoration_step: i32) {
        let l = decoration_seed.wrapping_add(index as i64).wrapping_add(10000i64 * decoration_step as i64);
        self.set_seed(l);
    }

    /// `setLargeFeatureSeed`.
    pub fn set_large_feature_seed(&mut self, base_seed: i64, chunk_x: i32, chunk_z: i32) {
        self.set_seed(base_seed);
        let random_long = self.next_long();
        let random_long1 = self.next_long();
        let l = (chunk_x as i64).wrapping_mul(random_long)
            ^ (chunk_z as i64).wrapping_mul(random_long1)
            ^ base_seed;
        self.set_seed(l);
    }

    /// `setLargeFeatureWithSalt`.
    pub fn set_large_feature_with_salt(&mut self, level_seed: i64, region_x: i32, region_z: i32, salt: i32) {
        let l = (region_x as i64).wrapping_mul(341873128712)
            .wrapping_add((region_z as i64).wrapping_mul(132897987541))
            .wrapping_add(level_seed)
            .wrapping_add(salt as i64);
        self.set_seed(l);
    }

    /// `seedSlimeChunk`.
    pub fn seed_slime_chunk(chunk_x: i32, chunk_z: i32, level_seed: i64, salt: i64) -> Box<dyn RandomSource> {
        let seed = level_seed
            .wrapping_add((chunk_x as i64).wrapping_mul(chunk_x as i64).wrapping_mul(4987142))
            .wrapping_add((chunk_x as i64).wrapping_mul(5947611))
            .wrapping_add((chunk_z as i64).wrapping_mul(chunk_z as i64).wrapping_mul(4392871))
            .wrapping_add((chunk_z as i64).wrapping_mul(389711))
            ^ salt;
        Box::new(LegacyRandomSource::new(seed))
    }
}

/// `WorldgenRandom.Algorithm`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Algorithm {
    /// `LEGACY`.
    Legacy,
    /// `XOROSHIRO`.
    Xoroshiro,
}

impl Algorithm {
    /// `newInstance(seed)`.
    pub fn new_instance(&self, seed: i64) -> Box<dyn RandomSource> {
        match self {
            Algorithm::Legacy => Box::new(LegacyRandomSource::new(seed)),
            Algorithm::Xoroshiro => Box::new(XoroshiroRandomSource::new(seed)),
        }
    }
}

impl RandomSource for WorldgenRandom {
    fn set_seed(&mut self, seed: i64) {
        self.inner.set_seed(seed);
    }

    fn fork(&mut self) -> Box<dyn RandomSource> {
        self.inner.fork()
    }

    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory> {
        self.inner.fork_positional()
    }

    fn next_int(&mut self) -> i32 {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.next_int()
    }

    fn next_int_bound(&mut self, bound: i32) -> i32 {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.next_int_bound(bound)
    }

    fn next_long(&mut self) -> i64 {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.next_long()
    }

    fn next_boolean(&mut self) -> bool {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.next_boolean()
    }

    fn next_float(&mut self) -> f32 {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.next_float()
    }

    fn next_double(&mut self) -> f64 {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.next_double()
    }

    fn next_gaussian(&mut self) -> f64 {
        self.count.fetch_add(1, Ordering::Relaxed);
        self.inner.next_gaussian()
    }

}
