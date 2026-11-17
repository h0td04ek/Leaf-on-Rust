//! Перенос `net.minecraft.world.flag.FeatureFlags` —
//! реестр ванильных feature-флагов.

use crate::resources::identifier::Identifier;
use crate::world::flag::feature_flag::FeatureFlag;

/// Зарегистрированные ванильные флаги (порядок vanilla).
pub const VANILLA_FLAGS: &[&str] = &[
    "vanilla",
    "trade_rebalance",
];

/// `FeatureFlags.REGISTRY`-подобный список.
pub fn vanilla_flags() -> Vec<FeatureFlag> {
    VANILLA_FLAGS
        .iter()
        .map(|name| FeatureFlag::new(Identifier::parse(&format!("minecraft:{name}")).expect("flag id")))
        .collect()
}
