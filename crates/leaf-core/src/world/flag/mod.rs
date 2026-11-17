//! Перенос `net.minecraft.world.flag`.

pub mod feature_flag;
pub mod feature_flag_set;
pub mod feature_flags;

pub use feature_flag::FeatureFlag;
pub use feature_flag_set::FeatureFlagSet;
pub use feature_flags::{vanilla_flags, VANILLA_FLAGS};
