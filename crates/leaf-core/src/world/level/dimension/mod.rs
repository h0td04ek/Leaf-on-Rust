//! Перенос `net.minecraft.world.level.dimension` — измерения.

pub mod dimension_type;
pub mod level_stem;

pub use dimension_type::{CardinalLightType, DimensionType, MonsterSettings, Skybox};
pub use level_stem::{level_to_level_stem, LevelStem, END, LEVEL_STEM_REGISTRY, NETHER, OVERWORLD};
