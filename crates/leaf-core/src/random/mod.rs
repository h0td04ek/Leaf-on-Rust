//! `RandomSource` — трейт-объект вместо Java interface
//! (fork/forkPositional возвращают Box<dyn ...>).
//!
//! Реализации: `legacy::LegacyRandomSource`, `single::SingleThreadedRandomSource`,
//! `xoroshiro::XoroshiroRandomSource`. FasterRandomSource (Leaf opt-in,
//! JDK RandomGeneratorFactory) — TODO, по умолчанию FastRNG выключен.

pub use crate::random::positional::PositionalRandomFactory;

pub trait RandomSource: Send + Sync {
    fn set_seed(&mut self, seed: i64);

    /// `fork()` — в Java потребляет nextLong() исходника.
    fn fork(&mut self) -> Box<dyn RandomSource>;
    fn fork_positional(&mut self) -> Box<dyn PositionalRandomFactory>;

    fn next_int(&mut self) -> i32;
    fn next_int_bound(&mut self, bound: i32) -> i32;

    fn next_int_range(&mut self, origin: i32, bound: i32) -> i32 {
        assert!(origin < bound, "bound - origin is non positive");
        origin + self.next_int_bound(bound - origin)
    }

    fn next_long(&mut self) -> i64;
    fn next_boolean(&mut self) -> bool;
    fn next_float(&mut self) -> f32;
    fn next_double(&mut self) -> f64;
    fn next_gaussian(&mut self) -> f64;

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

pub mod bit;
pub mod legacy;
pub mod positional;
pub mod single;
pub mod support;
pub mod xoroshiro;

pub use legacy::{create, create_legacy, create_unique, java_hash_code, LegacyRandomSource};
pub use single::SingleThreadedRandomSource;
pub use support::{generate_unique_seed, Seed128bit};
pub use xoroshiro::{Xoroshiro128PlusPlus, XoroshiroRandomSource};

#[cfg(test)]
mod tests;
