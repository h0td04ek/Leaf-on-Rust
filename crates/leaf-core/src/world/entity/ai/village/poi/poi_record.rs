//! Перенос `net.minecraft.world.entity.ai.village.poi.PoiRecord` —
//! запись POI: позиция + тип (Holder) + свободные билеты.

use std::rc::Rc;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::world::entity::ai::village::poi::poi_type::PoiType;
use crate::world::entity::ai::village::poi::poi_types;

/// Разрешение `Holder<PoiType>` → `PoiType` (Direct → clone,
/// Key → таблица `poi_types`).
pub fn resolve_poi_type(holder: &Holder<PoiType>) -> Option<PoiType> {
    match holder {
        Holder::Direct(t) => Some(t.clone()),
        Holder::Key(k) => poi_types::get(k),
        Holder::Reference(_) => None,
    }
}

/// `Holder<PoiType>.is(ResourceKey)`-эквивалент.
pub fn holder_is(holder: &Holder<PoiType>, key: &crate::resources::resource_key::ResourceKey<PoiType>) -> bool {
    match holder {
        Holder::Direct(t) => t.id == key.identifier,
        Holder::Key(k) => k == key,
        Holder::Reference(_) => false,
    }
}

/// `Holder<PoiType>.is(Holder)`-эквивалент по id.
pub fn holder_eq(a: &Holder<PoiType>, b: &Holder<PoiType>) -> bool {
    match (a, b) {
        (Holder::Direct(x), Holder::Direct(y)) => x == y,
        (Holder::Key(x), Holder::Key(y)) => x == y,
        (Holder::Direct(x), Holder::Key(y)) | (Holder::Key(y), Holder::Direct(x)) => {
            x.id == y.identifier
        }
        _ => false,
    }
}

/// `PoiRecord(pos, poiType, freeTickets, setDirty)`.
#[derive(Clone)]
pub struct PoiRecord {
    pos: BlockPos,
    poi_type: Holder<PoiType>,
    free_tickets: i32,
    /// `setDirty` — пометка секции на сохранение.
    set_dirty: Rc<dyn Fn()>,
}

impl PoiRecord {
    /// `new(pos, poiType, setDirty)` — freeTickets = maxTickets.
    pub fn new(pos: BlockPos, poi_type: Holder<PoiType>, set_dirty: Rc<dyn Fn()>) -> Self {
        let max = resolve_poi_type(&poi_type).map(|t| t.max_tickets).unwrap_or(0);
        Self { pos: pos.immutable(), poi_type, free_tickets: max, set_dirty }
    }

    /// `new(pos, poiType, freeTickets, setDirty)`.
    pub fn with_tickets(
        pos: BlockPos,
        poi_type: Holder<PoiType>,
        free_tickets: i32,
        set_dirty: Rc<dyn Fn()>,
    ) -> Self {
        Self { pos: pos.immutable(), poi_type, free_tickets, set_dirty }
    }

    fn max_tickets(&self) -> i32 {
        resolve_poi_type(&self.poi_type).map(|t| t.max_tickets).unwrap_or(0)
    }

    /// `acquireTicket` — занять слот. `false` если мест нет.
    pub fn acquire_ticket(&mut self) -> bool {
        if self.free_tickets <= 0 {
            return false;
        }
        self.free_tickets -= 1;
        (self.set_dirty)();
        true
    }

    /// `releaseTicket` — освободить слот.
    pub fn release_ticket(&mut self) -> bool {
        if self.free_tickets >= self.max_tickets() {
            return false;
        }
        self.free_tickets += 1;
        (self.set_dirty)();
        true
    }

    /// `hasSpace`.
    pub fn has_space(&self) -> bool {
        self.free_tickets > 0
    }

    /// `isOccupied` — freeTickets != maxTickets.
    pub fn is_occupied(&self) -> bool {
        self.free_tickets != self.max_tickets()
    }

    /// `getPos`.
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    /// `getPoiType`.
    pub fn get_poi_type(&self) -> &Holder<PoiType> {
        &self.poi_type
    }

    /// `getFreeTickets` (debug).
    pub fn free_tickets(&self) -> i32 {
        self.free_tickets
    }
}

impl PartialEq for PoiRecord {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos
    }
}
impl Eq for PoiRecord {}
impl std::hash::Hash for PoiRecord {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.pos.hash(state);
    }
}
