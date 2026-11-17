//! Перенос `net.minecraft.world.flag.FeatureFlagSet` —
//! множество включённых feature-флагов (bitmask над universe).

use crate::resources::identifier::Identifier;
use crate::world::flag::feature_flag::FeatureFlag;
use std::collections::BTreeSet;

/// `FeatureFlagSet` — набор `FeatureFlag`.
/// Java-реализация — bitmask; здесь — упорядоченный набор id
/// (семантика `isEnabled`/`subset` сохранена).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FeatureFlagSet {
    /// Множество id флагов.
    pub flags: BTreeSet<Identifier>,
}

impl FeatureFlagSet {
    /// `FeatureFlagSet.of()` — пустое.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `VANILLA_SET` — все ванильные флаги.
    pub fn all_vanilla() -> Self {
        Self {
            flags: crate::world::flag::feature_flags::vanilla_flags()
                .into_iter()
                .map(|f| f.id)
                .collect(),
        }
    }

    /// `subset(flags)` — множество, содержащее только перечисленные.
    pub fn subset(flags: &[FeatureFlag]) -> Self {
        Self { flags: flags.iter().map(|f| f.id.clone()).collect() }
    }

    /// `isEnabled(flag)`.
    pub fn is_enabled(&self, flag: &FeatureFlag) -> bool {
        self.flags.contains(&flag.id)
    }

    /// `contains(flagId)`.
    pub fn contains(&self, id: &Identifier) -> bool {
        self.flags.contains(id)
    }
}
