//! Перенос `net.minecraft.world.level.levelgen.structure.StructureFeatureIndexSavedData`.
//!
//! Хранит индексы чанков со structure-start'ами: `all` — все когда-либо
//! сгенерированные, `remaining` — ещё не обработанные chunk-лоадером
//! (`markChunkDone` → `removeIndex`).
//!
//! Java CODEC: `RecordCodecBuilder` с `LONG_SET` (`Codec.LONG_STREAM`)
//! для полей `All`/`Remaining`. В NBT long-stream сериализуется как
//! `LongArrayTag` (NbtOps); decode принимает также список long'ов.

use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::util::datafix::data_fix_types::DataFixTypes;
use crate::world::level::saveddata::saved_data::{SavedData, SavedDataBase};
use crate::world::level::saveddata::saved_data_type::{SavedDataCodec, SavedDataType};
use std::any::Any;
use std::collections::HashSet;

/// `StructureFeatureIndexSavedData`.
pub struct StructureFeatureIndexSavedData {
    base: SavedDataBase,
    all: HashSet<i64>,
    remaining: HashSet<i64>,
}

impl StructureFeatureIndexSavedData {
    /// Приватный Java-конструктор `(all, remaining)` → `new`.
    pub fn new() -> Self {
        Self {
            base: SavedDataBase::default(),
            all: HashSet::new(),
            remaining: HashSet::new(),
        }
    }

    /// `StructureFeatureIndexSavedData.type(id)`.
    pub fn saved_data_type(id: impl Into<String>) -> SavedDataType<Self> {
        SavedDataType::new(
            id,
            Self::new,
            SavedDataCodec::new(Self::save, Self::load),
            DataFixTypes::SavedDataStructureFeatureIndices,
        )
    }

    /// `addIndex` — добавить в оба множества + dirty.
    pub fn add_index(&mut self, index: i64) {
        self.all.insert(index);
        self.remaining.insert(index);
        self.base.set_dirty();
    }

    /// `hasStartIndex`.
    pub fn has_start_index(&self, index: i64) -> bool {
        self.all.contains(&index)
    }

    /// `hasUnhandledIndex`.
    pub fn has_unhandled_index(&self, index: i64) -> bool {
        self.remaining.contains(&index)
    }

    /// `removeIndex` — dirty только если элемент реально был.
    pub fn remove_index(&mut self, index: i64) {
        if self.remaining.remove(&index) {
            self.base.set_dirty();
        }
    }

    /// `getAll`.
    pub fn get_all(&self) -> &HashSet<i64> {
        &self.all
    }

    /// `CODEC` encode — `{All: long[], Remaining: long[]}`.
    fn save(&self) -> CompoundTag {
        let mut tag = CompoundTag::new();
        tag.put_long_array("All", &self.all.iter().copied().collect::<Vec<_>>());
        tag.put_long_array(
            "Remaining",
            &self.remaining.iter().copied().collect::<Vec<_>>(),
        );
        tag
    }

    /// `CODEC` decode — принимает LongArray или List<Long> для каждого поля.
    fn load(tag: &CompoundTag) -> Option<Self> {
        let all = read_long_set(tag, "All")?;
        let remaining = read_long_set(tag, "Remaining")?;
        Some(Self {
            base: SavedDataBase::default(),
            all,
            remaining,
        })
    }
}

impl Default for StructureFeatureIndexSavedData {
    fn default() -> Self {
        Self::new()
    }
}

impl SavedData for StructureFeatureIndexSavedData {
    fn is_dirty(&self) -> bool {
        self.base.is_dirty()
    }

    fn set_dirty_flag(&mut self, dirty: bool) {
        self.base.set_dirty_flag(dirty);
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// `LONG_SET` decode — LongArrayTag или ListTag of Long.
fn read_long_set(tag: &CompoundTag, key: &str) -> Option<HashSet<i64>> {
    let mut set = HashSet::new();
    match tag.get(key)? {
        Tag::LongArray(arr) => {
            set.extend(arr.iter().copied());
        }
        Tag::List(list) => {
            for item in list {
                if let Some(v) = item.as_long() {
                    set.insert(v);
                }
            }
        }
        _ => return None,
    }
    Some(set)
}
