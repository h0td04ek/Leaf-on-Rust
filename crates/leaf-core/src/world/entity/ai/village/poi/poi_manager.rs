//! Перенос `net.minecraft.world.entity.ai.village.poi.PoiManager` —
//! менеджер POI: реестр записей + запросы (find/getInRange/
//! take/release/exists/getType/getRandom).
//!
//! Java-версия — `SectionStorage<PoiSection>` поверх chunk IO.
//! Здесь — trait `PoiManager` (object-safe) + in-memory
//! реализация `InMemoryPoiManager`, которую `ServerLevel` сможет
//! обернуть поверх persistent-хранилища позже. `LevelReader`
//! получает доступ через `poi_manager()`.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::random::RandomSource;
use crate::util::random::weighted_random::WeightedRandom;
use crate::world::entity::ai::village::poi::poi_record::{holder_eq, PoiRecord, resolve_poi_type};

pub use crate::world::entity::ai::village::poi::poi_record::holder_is;
use crate::world::entity::ai::village::poi::poi_type::PoiType;

/// `PoiManager.Occupancy`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occupancy {
    /// `HAS_SPACE` — есть свободные билеты.
    HasSpace,
    /// `IS_OCCUPIED` — занято (freeTickets != max).
    IsOccupied,
    /// `ANY`.
    Any,
}

impl Occupancy {
    fn test(self, record: &PoiRecord) -> bool {
        match self {
            Occupancy::HasSpace => record.has_space(),
            Occupancy::IsOccupied => record.is_occupied(),
            Occupancy::Any => true,
        }
    }
}

/// Тип предиката по типу POI — `Predicate<Holder<PoiType>>`.
pub type PoiTypePredicate<'a> = dyn Fn(&Holder<PoiType>) -> bool + 'a;

/// `PoiManager` — query API (object-safe).
/// `&self`-методы с `RefCell`-мутацией внутри реализаций,
/// т.к. Java-менеджер разделяется между сущностями.
pub trait PoiManager {
    /// `add(pos, type)` → Option<PoiRecord> (None если тип
    /// не добавлен — Java возвращает null при коллизии id? нет —
    /// всегда Some; оставляем Option для ABI-совместимости
    /// с `getOrCreate`-ошибками).
    fn add(&self, pos: BlockPos, poi_type: Holder<PoiType>) -> Option<PoiRecord>;

    /// `remove(pos)`.
    fn remove(&self, pos: BlockPos);

    /// `getCountInRange(typePredicate, pos, distance, status)`.
    fn get_count_in_range(
        &self,
        type_predicate: &PoiTypePredicate,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> i64 {
        self.get_in_range(type_predicate, pos, distance, status).len() as i64
    }

    /// `existsAtPosition(poiType key, pos)`.
    fn exists_at_position(&self, poi_type: &crate::resources::resource_key::ResourceKey<PoiType>, pos: BlockPos) -> bool {
        self.exists(pos, &|h| holder_is(h, poi_type))
    }

    /// `getInRange` — записи в кубе distance + евклидов фильтр.
    fn get_in_range(
        &self,
        type_predicate: &PoiTypePredicate,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Vec<PoiRecord>;

    /// `findAll` — позиции.
    fn find_all(
        &self,
        type_predicate: &PoiTypePredicate,
        pos_predicate: &dyn Fn(BlockPos) -> bool,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Vec<BlockPos> {
        self.get_in_range(type_predicate, pos, distance, status)
            .into_iter()
            .map(|r| r.get_pos())
            .filter(|p| pos_predicate(*p))
            .collect()
    }

    /// `findAllWithType` — (type, pos) пары.
    fn find_all_with_type(
        &self,
        type_predicate: &PoiTypePredicate,
        pos_predicate: &dyn Fn(BlockPos) -> bool,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Vec<(Holder<PoiType>, BlockPos)> {
        self.get_in_range(type_predicate, pos, distance, status)
            .into_iter()
            .filter(|r| pos_predicate(r.get_pos()))
            .map(|r| (r.get_poi_type().clone(), r.get_pos()))
            .collect()
    }

    /// `findAllClosestFirstWithType` — отсортировано по
    /// distSqr к pos.
    fn find_all_closest_first_with_type(
        &self,
        type_predicate: &PoiTypePredicate,
        pos_predicate: &dyn Fn(BlockPos) -> bool,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Vec<(Holder<PoiType>, BlockPos)> {
        let mut v = self.find_all_with_type(type_predicate, pos_predicate, pos, distance, status);
        v.sort_by(|a, b| {
            a.1.dist_sqr(pos)
                .partial_cmp(&b.1.dist_sqr(pos))
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        v
    }

    /// `find(typePredicate, posPredicate, pos, distance, status)`.
    fn find(
        &self,
        type_predicate: &PoiTypePredicate,
        pos_predicate: &dyn Fn(BlockPos) -> bool,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Option<BlockPos> {
        self.find_all(type_predicate, pos_predicate, pos, distance, status)
            .into_iter()
            .next()
    }

    /// `findClosest(typePredicate, pos, distance, status)`.
    fn find_closest(
        &self,
        type_predicate: &PoiTypePredicate,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Option<BlockPos> {
        self.get_in_range(type_predicate, pos, distance, status)
            .iter()
            .map(|r| r.get_pos())
            .min_by(|a, b| {
                a.dist_sqr(pos)
                    .partial_cmp(&b.dist_sqr(pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// `findClosest(typePredicate, posPredicate, pos, distance, status)`.
    fn find_closest_with_pos_pred(
        &self,
        type_predicate: &PoiTypePredicate,
        pos_predicate: &dyn Fn(BlockPos) -> bool,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Option<BlockPos> {
        self.get_in_range(type_predicate, pos, distance, status)
            .iter()
            .map(|r| r.get_pos())
            .filter(|p| pos_predicate(*p))
            .min_by(|a, b| {
                a.dist_sqr(pos)
                    .partial_cmp(&b.dist_sqr(pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    /// `findClosestWithType`.
    fn find_closest_with_type(
        &self,
        type_predicate: &PoiTypePredicate,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Option<(Holder<PoiType>, BlockPos)> {
        self.get_in_range(type_predicate, pos, distance, status)
            .iter()
            .min_by(|a, b| {
                a.get_pos()
                    .dist_sqr(pos)
                    .partial_cmp(&b.get_pos().dist_sqr(pos))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|r| (r.get_poi_type().clone(), r.get_pos()))
    }

    /// `take(typePredicate, combinedTypePosPredicate, pos, distance)`
    /// — занять ближайший POI с местом.
    fn take(
        &self,
        type_predicate: &PoiTypePredicate,
        combined: &dyn Fn(&Holder<PoiType>, BlockPos) -> bool,
        pos: BlockPos,
        distance: i32,
    ) -> Option<BlockPos> {
        // getInRange HAS_SPACE — но acquireTicket внутри;
        // реализация в trait не может мутировать record —
        // поэтому take реализуется в конкретных менеджерах.
        let _ = (type_predicate, combined, pos, distance);
        None
    }

    /// `getRandom(typePredicate, posPredicate, status, pos,
    /// distance, random)`.
    fn get_random(
        &self,
        type_predicate: &PoiTypePredicate,
        pos_predicate: &dyn Fn(BlockPos) -> bool,
        status: Occupancy,
        pos: BlockPos,
        distance: i32,
        random: &mut dyn RandomSource,
    ) -> Option<BlockPos> {
        let records = self.get_in_range(type_predicate, pos, distance, status);
        // Util.toShuffledList + findFirst.
        let mut list = records;
        // Fisher-Yates.
        for i in (1..list.len()).rev() {
            let j = random.next_int_bound((i + 1) as i32) as usize;
            list.swap(i, j);
        }
        list.iter()
            .find(|r| pos_predicate(r.get_pos()))
            .map(|r| r.get_pos())
    }

    /// `release(pos)` — освободить билет. `false`/ошибка если
    /// POI нет (Java бросает IllegalStateException).
    fn release(&self, pos: BlockPos) -> bool;

    /// `exists(pos, typePredicate)`.
    fn exists(&self, pos: BlockPos, type_predicate: &PoiTypePredicate) -> bool;

    /// `getType(pos)` — тип POI в позиции.
    fn get_type(&self, pos: BlockPos) -> Option<Holder<PoiType>>;

    /// `ensureLoadedAndValid(level, pos, radius)` — подгрузка
    /// чанков вокруг (no-op в in-memory).
    fn ensure_loaded_and_valid(&self, _level: &dyn crate::world::level::level_reader::LevelReader, _pos: BlockPos, _radius: i32) {}

    /// `sectionsToVillage(sectionPos)` — BFS-уровень до
    /// ближайшего village center (default 7 = MAX_VILLAGE_DISTANCE).
    fn sections_to_village(&self, _section_pos: i64) -> i32 {
        7
    }
}

// ===================== InMemoryPoiManager =====================

/// `InMemoryPoiManager` — POI-хранилище в памяти
/// (HashMap<BlockPos, PoiRecord>), `setDirty` — no-op.
/// Используется до порта `PoiSection`/chunk-storage.
pub struct InMemoryPoiManager {
    records: RefCell<HashMap<BlockPos, PoiRecord>>,
    /// Счётчик dirty-событий для отладки/тестов.
    pub dirty_count: std::cell::Cell<i64>,
}

impl InMemoryPoiManager {
    /// `new()`.
    pub fn new() -> Self {
        Self {
            records: RefCell::new(HashMap::new()),
            dirty_count: std::cell::Cell::new(0),
        }
    }

    fn dirty(&self) {
        self.dirty_count.set(self.dirty_count.get() + 1);
    }
}

impl Default for InMemoryPoiManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PoiManager for InMemoryPoiManager {
    fn add(&self, pos: BlockPos, poi_type: Holder<PoiType>) -> Option<PoiRecord> {
        let pos = pos.immutable();
        let mut records = self.records.borrow_mut();
        if records.contains_key(&pos) {
            // Java: секция хранит одну запись на pos —
            // перезаписываем тип (setDirty).
            let rec = PoiRecord::new(pos, poi_type, Rc::new(|| {}));
            records.insert(pos, rec.clone());
            Some(rec)
        } else {
            let rec = PoiRecord::new(pos, poi_type, Rc::new(|| {}));
            records.insert(pos, rec.clone());
            Some(rec)
        }
    }

    fn remove(&self, pos: BlockPos) {
        self.records.borrow_mut().remove(&pos);
    }

    fn get_in_range(
        &self,
        type_predicate: &PoiTypePredicate,
        pos: BlockPos,
        distance: i32,
        status: Occupancy,
    ) -> Vec<PoiRecord> {
        let i = distance * distance;
        self.records
            .borrow()
            .values()
            .filter(|r| {
                type_predicate(r.get_poi_type())
                    && status.test(r)
                    && r.get_pos().dist_sqr(pos) <= i as f64
            })
            .cloned()
            .collect()
    }

    fn take(
        &self,
        type_predicate: &PoiTypePredicate,
        combined: &dyn Fn(&Holder<PoiType>, BlockPos) -> bool,
        pos: BlockPos,
        distance: i32,
    ) -> Option<BlockPos> {
        // getInRange HAS_SPACE, первый совпавший,
        // acquireTicket.
        let candidates: Vec<BlockPos> = self
            .get_in_range(type_predicate, pos, distance, Occupancy::HasSpace)
            .into_iter()
            .filter(|r| combined(r.get_poi_type(), r.get_pos()))
            .map(|r| r.get_pos())
            .collect();
        for p in candidates {
            let mut records = self.records.borrow_mut();
            if let Some(rec) = records.get_mut(&p) {
                if rec.acquire_ticket() {
                    return Some(p);
                }
            }
        }
        None
    }

    fn release(&self, pos: BlockPos) -> bool {
        let mut records = self.records.borrow_mut();
        match records.get_mut(&pos) {
            Some(rec) => rec.release_ticket(),
            // Java: Util.pauseInIde(IllegalStateException) —
            // паника в debug, false в release.
            None => {
                debug_assert!(false, "POI never registered at {:?}", pos);
                false
            }
        }
    }

    fn exists(&self, pos: BlockPos, type_predicate: &PoiTypePredicate) -> bool {
        self.records
            .borrow()
            .get(&pos)
            .map(|r| type_predicate(r.get_poi_type()))
            .unwrap_or(false)
    }

    fn get_type(&self, pos: BlockPos) -> Option<Holder<PoiType>> {
        self.records
            .borrow()
            .get(&pos)
            .map(|r| r.get_poi_type().clone())
    }
}

/// Утилита для предикатов `holder.is(ResourceKey)`.
pub fn is_key(key: &crate::resources::resource_key::ResourceKey<PoiType>) -> impl Fn(&Holder<PoiType>) -> bool + '_ {
    move |h| holder_is(h, key)
}

/// `holder.is(otherHolder)` по id.
pub fn is_holder(other: &Holder<PoiType>) -> impl Fn(&Holder<PoiType>) -> bool + '_ {
    move |h| holder_eq(h, other)
}

/// `PoiType.NONE` — предикат, всегда false (для API-совместимости).
pub fn none_predicate() -> impl Fn(&Holder<PoiType>) -> bool {
    |_| false
}

/// `PoiType.value().validRange()` по holder.
pub fn valid_range_of(holder: &Holder<PoiType>) -> i32 {
    resolve_poi_type(holder).map(|t| t.valid_range).unwrap_or(1)
}

/// `PoiType.value().maxTickets()` по holder.
pub fn max_tickets_of(holder: &Holder<PoiType>) -> i32 {
    resolve_poi_type(holder).map(|t| t.max_tickets).unwrap_or(0)
}

#[allow(dead_code)]
fn _weighted_random_marker(_: &mut dyn RandomSource, _: &[PoiRecord]) {
    // WeightedRandom импортирован для будущего использования
    // в distance-weighted выборках (getRandom с весами).
    let _ = WeightedRandom::get_total_weight(&[] as &[PoiRecord], |_: &PoiRecord| 0);
}
