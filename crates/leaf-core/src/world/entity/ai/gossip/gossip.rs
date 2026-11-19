//! Перенос `net.minecraft.world.entity.ai.gossip` —
//! `GossipType`, `GossipContainer` (репутация жителя о других).

use std::collections::{HashMap, HashSet};

use crate::random::RandomSource;
use crate::util::uuid::Uuid;

/// `GossipType` — тип сплетни с весом и decay-параметрами.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GossipType {
    /// `MAJOR_NEGATIVE`.
    MajorNegative,
    /// `MINOR_NEGATIVE`.
    MinorNegative,
    /// `MINOR_POSITIVE`.
    MinorPositive,
    /// `MAJOR_POSITIVE`.
    MajorPositive,
    /// `TRADING`.
    Trading,
}

/// Java-константы `GossipType`.
pub mod gossip_consts {
    use super::GossipType;

    /// `REPUTATION_CHANGE_PER_EVENT`.
    pub const REPUTATION_CHANGE_PER_EVENT: i32 = 25;
    /// `REPUTATION_CHANGE_PER_EVERLASTING_MEMORY`.
    pub const REPUTATION_CHANGE_PER_EVERLASTING_MEMORY: i32 = 20;
    /// `REPUTATION_CHANGE_PER_TRADE`.
    pub const REPUTATION_CHANGE_PER_TRADE: i32 = 2;

    impl GossipType {
        /// `id` — сериализованное имя.
        pub fn id(&self) -> &'static str {
            match self {
                Self::MajorNegative => "major_negative",
                Self::MinorNegative => "minor_negative",
                Self::MinorPositive => "minor_positive",
                Self::MajorPositive => "major_positive",
                Self::Trading => "trading",
            }
        }

        /// `weight`.
        pub fn weight(&self) -> i32 {
            match self {
                Self::MajorNegative => -5,
                Self::MinorNegative => -1,
                Self::MinorPositive => 1,
                Self::MajorPositive => 5,
                Self::Trading => 1,
            }
        }

        /// `max`.
        pub fn max_reputation(&self) -> i32 {
            match self {
                Self::MajorNegative => 100,
                Self::MinorNegative => 200,
                Self::MinorPositive => 25,
                Self::MajorPositive => 20,
                Self::Trading => 25,
            }
        }

        /// `decayPerDay`.
        pub fn decay_per_day(&self) -> i32 {
            match self {
                Self::MajorNegative => 10,
                Self::MinorNegative => 20,
                Self::MinorPositive => 1,
                Self::MajorPositive => 0,
                Self::Trading => 2,
            }
        }

        /// `decayPerTransfer`.
        pub fn decay_per_transfer(&self) -> i32 {
            match self {
                Self::MajorNegative => 10,
                Self::MinorNegative => 20,
                Self::MinorPositive => 5,
                Self::MajorPositive => 20,
                Self::Trading => 20,
            }
        }

        /// Все значения (`GossipType.values()`).
        pub fn values() -> [GossipType; 5] {
            [
                Self::MajorNegative,
                Self::MinorNegative,
                Self::MinorPositive,
                Self::MajorPositive,
                Self::Trading,
            ]
        }

        /// `StringRepresentable.fromEnum` — по имени.
        pub fn by_name(name: &str) -> Option<GossipType> {
            Self::values().into_iter().find(|t| t.id() == name)
        }
    }
}

/// `GossipContainer.GossipEntry` — (target, type, value).
#[derive(Clone, Copy, Debug)]
pub struct GossipEntry {
    /// `target` — UUID цели сплетни.
    pub target: Uuid,
    /// `type`.
    pub gossip_type: GossipType,
    /// `value`.
    pub value: i32,
}

impl GossipEntry {
    /// `new(target, type, value)`.
    pub fn new(target: Uuid, gossip_type: GossipType, value: i32) -> Self {
        Self {
            target,
            gossip_type,
            value,
        }
    }

    /// `weightedValue()` — `value * type.weight`.
    pub fn weighted_value(&self) -> i32 {
        self.value * self.gossip_type.weight()
    }
}

/// `GossipContainer.EntityGossips` — map<GossipType,int> одного
/// целевого UUID.
#[derive(Default)]
pub struct EntityGossips {
    /// `entries` — Object2IntMap<GossipType>.
    pub entries: HashMap<GossipType, i32>,
}

impl EntityGossips {
    /// `weightedValue(predicate)` — сумма value*weight по типам,
    /// прошедшим предикат.
    pub fn weighted_value(&self, gossip: &dyn Fn(GossipType) -> bool) -> i32 {
        self.entries
            .iter()
            .filter(|(t, _)| gossip(**t))
            .map(|(t, v)| v * t.weight())
            .sum()
    }

    /// `decompress(uuid)` — все записи как GossipEntry.
    pub fn decompress(&self, uuid: Uuid) -> Vec<GossipEntry> {
        self.entries
            .iter()
            .map(|(t, v)| GossipEntry::new(uuid, *t, *v))
            .collect()
    }

    /// `unpack(uuid)` — stream-вариант (здесь Vec).
    pub fn unpack(&self, uuid: Uuid) -> Vec<GossipEntry> {
        self.decompress(uuid)
    }

    /// `decay()` — уменьшение на decayPerDay; <2 → удаление.
    pub fn decay(&mut self) {
        self.entries.retain(|t, v| {
            let nv = *v - t.decay_per_day();
            if nv < 2 {
                false
            } else {
                *v = nv;
                true
            }
        });
    }

    /// `isEmpty()`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `makeSureValueIsntTooLowOrTooHigh(type)`.
    pub fn make_sure_value_isnt_too_low_or_too_high(&mut self, gossip_type: GossipType) {
        let v = self.entries.get(&gossip_type).copied().unwrap_or(0);
        if v > gossip_type.max_reputation() {
            self.entries.insert(gossip_type, gossip_type.max_reputation());
        }
        if v < 2 {
            self.entries.remove(&gossip_type);
        }
    }

    /// `remove(type)`.
    pub fn remove(&mut self, gossip_type: GossipType) {
        self.entries.remove(&gossip_type);
    }
}

/// `GossipContainer` — репутация жителя: Map<UUID, EntityGossips>.
#[derive(Default)]
pub struct GossipContainer {
    /// `gossips`.
    pub gossips: HashMap<Uuid, EntityGossips>,
}

impl GossipContainer {
    /// `new()`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `getOrCreate(uuid)` — приватный в Java; здесь pub(crate)-экв.
    fn get_or_create(&mut self, uuid: Uuid) -> &mut EntityGossips {
        self.gossips.entry(uuid).or_default()
    }

    /// `decay()` — decay всех EntityGossips + удаление пустых.
    pub fn decay(&mut self) {
        self.gossips.retain(|_, g| {
            g.decay();
            !g.is_empty()
        });
    }

    /// `unpack()` — все записи всех целей.
    pub fn unpack(&self) -> Vec<GossipEntry> {
        let mut out = Vec::new();
        for (uuid, g) in &self.gossips {
            out.extend(g.unpack(*uuid));
        }
        out
    }

    /// `decompress()` — Paper-вариант unpack с фильтром weighted!=0.
    pub fn decompress(&self) -> Vec<GossipEntry> {
        self.unpack()
            .into_iter()
            .filter(|e| e.weighted_value() != 0)
            .collect()
    }

    /// `selectGossipsForTransfer(random, amount)` — взвешенный
    /// выбор `amount` записей (с возвращением — Set по identity,
    /// здесь HashSet по значению записи).
    pub fn select_gossips_for_transfer(
        &self,
        random: &mut dyn RandomSource,
        amount: i32,
    ) -> Vec<GossipEntry> {
        let list = self.decompress();
        if list.is_empty() {
            return Vec::new();
        }
        // Взвешенные интервалы как в Java.
        let mut ints = Vec::with_capacity(list.len());
        let mut i = 0i32;
        for e in &list {
            i += e.weighted_value().abs();
            ints.push(i - 1);
        }
        let mut set: HashSet<GossipEntryKey> = HashSet::new();
        for _ in 0..amount {
            if i <= 0 {
                break;
            }
            let random_int = random.next_int_bound(i);
            // binarySearch-эквивалент: первый ints[idx] >= randomInt.
            let idx = ints.partition_point(|&v| v < random_int);
            let idx = if idx < ints.len() { idx } else { ints.len() - 1 };
            set.insert(GossipEntryKey::from(&list[idx]));
        }
        // Возвращаем записи в порядке оригинального списка
        // (Set в Java — identity; у нас дедуп по ключу).
        list.into_iter()
            .filter(|e| set.contains(&GossipEntryKey::from(e)))
            .collect()
    }

    /// `transferFrom(container, random, amount)` — перенос
    /// сплетен с decayPerTransfer.
    pub fn transfer_from(
        &mut self,
        container: &GossipContainer,
        random: &mut dyn RandomSource,
        amount: i32,
    ) {
        let selected = container.select_gossips_for_transfer(random, amount);
        for entry in selected {
            let i = entry.value - entry.gossip_type.decay_per_transfer();
            if i >= 2 {
                // mergeValuesForTransfer = max(existing, new).
                let g = self.get_or_create(entry.target);
                g.entries
                    .entry(entry.gossip_type)
                    .and_modify(|v| *v = (*v).max(i))
                    .or_insert(i);
            }
        }
    }

    /// `getReputation(uuid, predicate)`.
    pub fn get_reputation(&self, identifier: Uuid, gossip: &dyn Fn(GossipType) -> bool) -> i32 {
        self.gossips
            .get(&identifier)
            .map(|g| g.weighted_value(gossip))
            .unwrap_or(0)
    }

    /// `getCountForType(type, predicate)` — количество записей,
    /// чьё value*weight удовлетворяет предикату.
    pub fn get_count_for_type(
        &self,
        gossip_type: GossipType,
        gossip_predicate: &dyn Fn(f64) -> bool,
    ) -> usize {
        self.gossips
            .values()
            .filter(|g| {
                gossip_predicate(
                    g.entries.get(&gossip_type).copied().unwrap_or(0) as f64
                        * gossip_type.weight() as f64,
                )
            })
            .count()
    }

    /// `add(uuid, type, value)` — merge с clamp по max.
    pub fn add(&mut self, identifier: Uuid, gossip_type: GossipType, gossip_value: i32) {
        {
            let g = self.get_or_create(identifier);
            let existing = g.entries.get(&gossip_type).copied().unwrap_or(0);
            let sum = existing + gossip_value;
            let merged = if sum > gossip_type.max_reputation() {
                existing.max(gossip_type.max_reputation())
            } else {
                sum
            };
            g.entries.insert(gossip_type, merged);
            g.make_sure_value_isnt_too_low_or_too_high(gossip_type);
        }
        if self
            .gossips
            .get(&identifier)
            .map(|g| g.is_empty())
            .unwrap_or(false)
        {
            self.gossips.remove(&identifier);
        }
    }

    /// `remove(uuid, type, value)` — add с отрицательным value.
    pub fn remove(&mut self, identifier: Uuid, gossip_type: GossipType, gossip_value: i32) {
        self.add(identifier, gossip_type, -gossip_value);
    }

    /// `remove(uuid, type)`.
    pub fn remove_type_for(&mut self, identifier: Uuid, gossip_type: GossipType) {
        if let Some(g) = self.gossips.get_mut(&identifier) {
            g.remove(gossip_type);
            if g.is_empty() {
                self.gossips.remove(&identifier);
            }
        }
    }

    /// `remove(type)` — удалить тип у всех.
    pub fn remove_type(&mut self, gossip_type: GossipType) {
        self.gossips.retain(|_, g| {
            g.remove(gossip_type);
            !g.is_empty()
        });
    }

    /// `clear()`.
    pub fn clear(&mut self) {
        self.gossips.clear();
    }

    /// `putAll(other)` — слияние всех записей.
    pub fn put_all(&mut self, other: &GossipContainer) {
        for (uuid, g) in &other.gossips {
            let target = self.get_or_create(*uuid);
            for (t, v) in &g.entries {
                target.entries.insert(*t, *v);
            }
        }
    }

    /// `copy()`.
    pub fn copy(&self) -> GossipContainer {
        let mut c = GossipContainer::new();
        c.put_all(self);
        c
    }
}

/// Ключ дедупликации GossipEntry (target,type,value) для
/// identity-set семантики `selectGossipsForTransfer`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct GossipEntryKey {
    target: Uuid,
    gossip_type: GossipType,
    value: i32,
}

impl From<&GossipEntry> for GossipEntryKey {
    fn from(e: &GossipEntry) -> Self {
        Self {
            target: e.target,
            gossip_type: e.gossip_type,
            value: e.value,
        }
    }
}
