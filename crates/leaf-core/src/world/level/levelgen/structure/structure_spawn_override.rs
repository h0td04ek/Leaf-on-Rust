//! Перенос `StructureSpawnOverride`.

use crate::resources::identifier::Identifier;
use std::sync::Arc;

/// `StructureSpawnOverride` — переопределение правил спавна
/// мобов внутри структуры (`BoundingBox` + EntityType).
#[derive(Clone, Debug)]
pub struct StructureSpawnOverride {
    /// `boundingBox`.
    pub bounding_box: super::bounding_box::BoundingBox,
    /// `entityType`.
    pub entity_type: Arc<crate::entity::entity_type::EntityType>,
}

impl StructureSpawnOverride {
    /// `new`.
    pub fn new(
        bounding_box: super::bounding_box::BoundingBox,
        entity_type: Arc<crate::entity::entity_type::EntityType>,
    ) -> Self {
        Self { bounding_box, entity_type }
    }
}
