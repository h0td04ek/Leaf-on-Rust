//! Перенос `net.minecraft.util.random`.
//!
//! DFU-кодеки (`Weighted.codec`, `WeightedList.codec`) и stream-кодеки
//! появятся вместе с портом codec-слоя; здесь — чистая логика выбора.

pub mod weighted;
pub mod weighted_list;
pub mod weighted_random;

pub use weighted::Weighted;
pub use weighted_list::{WeightedList, WeightedListBuilder};
pub use weighted_random::WeightedRandom;
