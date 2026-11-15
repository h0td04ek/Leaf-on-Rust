//! `PositionalRandomFactory` — RNG, выведенный из позиции/сида.

use crate::random::RandomSource;

pub trait PositionalRandomFactory {
    /// `at(x, y, z)` — RNG для позиции.
    fn at(&self, x: i32, y: i32, z: i32) -> Box<dyn RandomSource>;
    /// `fromHashOf(name)` — RNG из хэша имени.
    fn from_hash_of(&self, name: &str) -> Box<dyn RandomSource>;
    /// `fromSeed(seed)`.
    fn from_seed(&self, seed: i64) -> Box<dyn RandomSource>;
    /// `parityConfigString` — отладочное описание.
    fn parity_config_string(&self) -> String;
}
