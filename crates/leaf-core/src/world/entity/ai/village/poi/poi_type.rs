//! Перенос `net.minecraft.world.entity.ai.village.poi.PoiType` —
//! тип точки интереса: набор подходящих block states,
//! maxTickets, validRange.

use std::sync::Arc;

use crate::resources::identifier::Identifier;
use crate::world::level::block::state::block_state::BlockState;

/// `PoiType(matchingStates, maxTickets, validRange)` —
/// `matchingStates` в Java — `Set<BlockState>`; здесь —
/// предикат по `BlockState` (эквивалентно, т.к. состояния
/// в нашей модели не хэшируются).
#[derive(Clone)]
pub struct PoiType {
    /// registry-id (`minecraft:home` и т.д.).
    pub id: Identifier,
    /// `matchingStates.contains(state)` — предикат.
    pub matcher: Arc<dyn Fn(&BlockState) -> bool + Send + Sync>,
    /// `maxTickets` — максимум одновременно занимающих POI.
    pub max_tickets: i32,
    /// `validRange` — радиус поиска POI этого типа.
    pub valid_range: i32,
}

impl PoiType {
    /// `new(matchingStates, maxTickets, validRange)`.
    pub fn new(
        id: Identifier,
        matcher: Arc<dyn Fn(&BlockState) -> bool + Send + Sync>,
        max_tickets: i32,
        valid_range: i32,
    ) -> Self {
        Self { id, matcher, max_tickets, valid_range }
    }

    /// `is(state)` — подходит ли состояние под POI.
    pub fn is(&self, state: &BlockState) -> bool {
        (self.matcher)(state)
    }
}

impl std::fmt::Debug for PoiType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PoiType")
            .field("id", &self.id)
            .field("max_tickets", &self.max_tickets)
            .field("valid_range", &self.valid_range)
            .finish()
    }
}

impl PartialEq for PoiType {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for PoiType {}
impl std::hash::Hash for PoiType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
