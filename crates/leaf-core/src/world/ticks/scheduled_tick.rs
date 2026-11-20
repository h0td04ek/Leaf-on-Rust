//! `net.minecraft.world.ticks.ScheduledTick` и `SavedTick` —
//! запланированный и сохранённый тики, компараторы и ключи уникальности.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::world::level::chunk_pos::ChunkPos;

use super::tick_priority::TickPriority;

/// Идентичность типа тика (`Block`/`Fluid` в Java — ссылочное `==`).
/// В Rust тип хранится по значению; для registry-синглтонов
/// `tick_hash` — стабильный id типа.
pub trait TickType: Send + Sync + Clone + 'static {
    /// Стабильный хэш идентичности типа (аналог `type().hashCode()`+`==`).
    fn tick_hash(&self) -> u64;
    /// Сравнение идентичности типа (аналог `type() == type()`).
    fn tick_eq(&self, other: &Self) -> bool;
}

/// Ключ уникальности тика по позиции (аналог `ScheduledTick.UNIQUE_TICK_HASH` /
/// `SavedTick.UNIQUE_TICK_HASH` — `31 * pos.hashCode() + type().hashCode()`
/// и `type() == type() && pos.equals`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TickKey {
    /// `pos`.
    pub pos: BlockPos,
    /// `type().hashCode()`-эквивалент.
    pub type_hash: u64,
}

impl TickKey {
    /// Ключ для `ScheduledTick::probe`-эквивалента.
    pub fn of<T: TickType>(pos: BlockPos, type_: &T) -> Self {
        Self {
            pos,
            type_hash: type_.tick_hash(),
        }
    }
}

/// `ScheduledTick<T>` — тик, запланированный на конкретный game-tick.
#[derive(Clone, Debug)]
pub struct ScheduledTick<T: TickType> {
    /// `type`.
    pub type_: T,
    /// `pos` (immutable).
    pub pos: BlockPos,
    /// `triggerTick` — game-time тика срабатывания.
    pub trigger_tick: i64,
    /// `priority`.
    pub priority: TickPriority,
    /// `subTickOrder` — порядок внутри тика.
    pub sub_tick_order: i64,
}

impl<T: TickType> ScheduledTick<T> {
    /// `new(type, pos, triggerTick, priority, subTickOrder)`.
    pub fn new(
        type_: T,
        pos: BlockPos,
        trigger_tick: i64,
        priority: TickPriority,
        sub_tick_order: i64,
    ) -> Self {
        Self {
            type_,
            pos,
            trigger_tick,
            priority,
            sub_tick_order,
        }
    }

    /// `new(type, pos, triggerTick, subTickOrder)` — NORMAL приоритет.
    pub fn with_default_priority(
        type_: T,
        pos: BlockPos,
        trigger_tick: i64,
        sub_tick_order: i64,
    ) -> Self {
        Self::new(type_, pos, trigger_tick, TickPriority::Normal, sub_tick_order)
    }

    /// `probe(type, pos)` — тик с triggerTick=0 для проверки наличия.
    pub fn probe(type_: T, pos: BlockPos) -> Self {
        Self::new(type_, pos, 0, TickPriority::Normal, 0)
    }

    /// `equals` — record-equality: все поля + идентичность типа.
    pub fn tick_eq_full(&self, other: &Self) -> bool {
        self.type_.tick_eq(&other.type_)
            && self.pos == other.pos
            && self.trigger_tick == other.trigger_tick
            && self.priority == other.priority
            && self.sub_tick_order == other.sub_tick_order
    }

    /// `toSavedTick(gameTime)`.
    pub fn to_saved_tick(&self, game_time: i64) -> SavedTick<T>
    where
        T: Clone,
    {
        SavedTick::new(
            self.type_.clone(),
            self.pos,
            (self.trigger_tick - game_time) as i32,
            self.priority,
        )
    }

    /// Ключ уникальности (`UNIQUE_TICK_HASH`).
    pub fn key(&self) -> TickKey {
        TickKey::of(self.pos, &self.type_)
    }
}

impl<T: TickType> PartialEq for ScheduledTick<T> {
    fn eq(&self, other: &Self) -> bool {
        self.tick_eq_full(other)
    }
}
impl<T: TickType> Eq for ScheduledTick<T> {}

/// `DRAIN_ORDER` — triggerTick, затем priority, затем subTickOrder.
pub fn drain_order_cmp<T: TickType>(a: &ScheduledTick<T>, b: &ScheduledTick<T>) -> std::cmp::Ordering {
    a.trigger_tick
        .cmp(&b.trigger_tick)
        .then(a.priority.cmp(&b.priority))
        .then(a.sub_tick_order.cmp(&b.sub_tick_order))
}

/// `INTRA_TICK_DRAIN_ORDER` — priority, затем subTickOrder.
pub fn intra_tick_drain_order_cmp<T: TickType>(
    a: &ScheduledTick<T>,
    b: &ScheduledTick<T>,
) -> std::cmp::Ordering {
    a.priority
        .cmp(&b.priority)
        .then(a.sub_tick_order.cmp(&b.sub_tick_order))
}

/// `SavedTick<T>` — сериализуемая форма тика (delay относительно gameTime).
#[derive(Clone, Debug)]
pub struct SavedTick<T: TickType> {
    /// `type`.
    pub type_: T,
    /// `pos`.
    pub pos: BlockPos,
    /// `delay` — тиков до срабатывания от текущего gameTime.
    pub delay: i32,
    /// `priority`.
    pub priority: TickPriority,
}

impl<T: TickType> SavedTick<T> {
    /// `new(type, pos, delay, priority)`.
    pub fn new(type_: T, pos: BlockPos, delay: i32, priority: TickPriority) -> Self {
        Self {
            type_,
            pos,
            delay,
            priority,
        }
    }

    /// `probe(type, pos)`.
    pub fn probe(type_: T, pos: BlockPos) -> Self {
        Self::new(type_, pos, 0, TickPriority::Normal)
    }

    /// `unpack(gameTime, subTickOrder)` → `ScheduledTick`.
    pub fn unpack(&self, game_time: i64, sub_tick_order: i64) -> ScheduledTick<T>
    where
        T: Clone,
    {
        ScheduledTick::new(
            self.type_.clone(),
            self.pos,
            game_time + self.delay as i64,
            self.priority,
            sub_tick_order,
        )
    }

    /// Ключ уникальности (`SavedTick.UNIQUE_TICK_HASH`).
    pub fn key(&self) -> TickKey {
        TickKey::of(self.pos, &self.type_)
    }

    /// `filterTickListForChunk(tickList, chunkPos)`.
    pub fn filter_tick_list_for_chunk(
        tick_list: &[SavedTick<T>],
        chunk_pos: ChunkPos,
    ) -> Vec<SavedTick<T>>
    where
        T: Clone,
    {
        let packed = chunk_pos.as_long();
        tick_list
            .iter()
            .filter(|t| {
                ChunkPos::as_long_static(t.pos.get_x() >> 4, t.pos.get_z() >> 4) == packed
            })
            .cloned()
            .collect()
    }

    /// `codec(typeCodec)` — NBT: `{i: type, x/y/z: pos, t: delay, p: priority}`
    /// (pos — MapCodec, поля x/y/z на верхнем уровне compound'а).
    /// Тип кодируется через [`TickNbt`].
    pub fn write_nbt(&self, tag: &mut CompoundTag)
    where
        T: TickNbt,
    {
        tag.put("i", self.type_.write_nbt());
        tag.put_int("x", self.pos.get_x());
        tag.put_int("y", self.pos.get_y());
        tag.put_int("z", self.pos.get_z());
        tag.put_int("t", self.delay);
        tag.put_int("p", self.priority.write_int());
    }

    /// Чтение из NBT.
    pub fn read_nbt(tag: &CompoundTag) -> Option<Self>
    where
        T: TickNbt,
    {
        let type_ = T::read_nbt(tag.get("i")?)?;
        let pos = BlockPos::new(
            tag.get_int("x")?,
            tag.get_int("y")?,
            tag.get_int("z")?,
        );
        let delay = tag.get_int("t")?;
        let priority = tag
            .get_int("p")
            .map(TickPriority::read_int)
            .unwrap_or(TickPriority::Normal);
        Some(Self::new(type_, pos, delay, priority))
    }
}

/// NBT-кодек типа тика для `SavedTick` (аналог `Codec<T> typeCodec`).
pub trait TickNbt: TickType {
    /// Записать тип в NBT tag.
    fn write_nbt(&self) -> Tag;
    /// Прочитать тип из NBT tag.
    fn read_nbt(tag: &Tag) -> Option<Self>
    where
        Self: Sized;
}
