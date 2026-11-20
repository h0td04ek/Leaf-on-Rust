//! Перенос `net.minecraft.world.level.dimension` — измерения.

pub mod level_stem;

pub use level_stem::{level_to_level_stem, LevelStem, END, LEVEL_STEM_REGISTRY, NETHER, OVERWORLD};
