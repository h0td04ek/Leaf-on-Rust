//! Перенос `net.minecraft.world.flag.FeatureFlag`.

use crate::resources::identifier::Identifier;

/// `FeatureFlag` — флаг(feature) для контента.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FeatureFlag {
    /// `id` — `minecraft:<name>`.
    pub id: Identifier,
}

impl FeatureFlag {
    /// `new`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}
