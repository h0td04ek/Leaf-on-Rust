//! Перенос `net.minecraft.util.datafix.DataFixTypes`.
//!
//! Отличия от Java:
//! - `TypeReference` заменён самим enum'ом (каждый вариант ↔ reference
//!   из `fixes.References`).
//! - `Dynamic<T>`/`DynamicOps` заменены на `CompoundTag` (NbtOps).
//! - `wrapCodec` не портирован — требует DFU `Codec`; вместо него
//!   [`wrap_encode`](Self::wrap_encode)/[`wrap_decode`](Self::wrap_decode)
//!   реализуют tag-уровень той же обёртки (поле `DataVersion`).
//! - `TYPES_FOR_LEVEL_LIST` — набор `TypeReference`; здесь — срез
//!   [`DataFixTypes`] (семантически то же множество).

use crate::core::shared_constants::CURRENT_DATA_VERSION;
use crate::nbt::compound::CompoundTag;
use crate::util::datafix::data_fixer::DataFixer;

/// `DataFixTypes` — типы данных, проходящие через data fixer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataFixTypes {
    Level,
    LevelSummary,
    Player,
    Chunk,
    Hotbar,
    Options,
    Structure,
    Stats,
    SavedDataCommandStorage,
    SavedDataForcedChunks,
    SavedDataMapData,
    SavedDataMapIndex,
    SavedDataRaids,
    SavedDataRandomSequences,
    SavedDataScoreboard,
    SavedDataStopwatches,
    SavedDataStructureFeatureIndices,
    SavedDataWorldBorder,
    Advancements,
    PoiChunk,
    WorldGenSettings,
    EntityChunk,
    DebugProfile,
}

/// `TYPES_FOR_LEVEL_LIST` — references, входящие в level-list.
pub const TYPES_FOR_LEVEL_LIST: &[DataFixTypes] = &[DataFixTypes::LevelSummary];

impl DataFixTypes {
    /// `currentVersion()` — `SharedConstants.getCurrentVersion().dataVersion()`.
    pub fn current_version() -> i32 {
        CURRENT_DATA_VERSION
    }

    /// `update(fixer, input, version, newVersion)` — tag-вариант.
    pub fn update(
        &self,
        fixer: &dyn DataFixer,
        tag: CompoundTag,
        version: i32,
        new_version: i32,
    ) -> CompoundTag {
        fixer.update(*self, tag, version, new_version)
    }

    /// `updateToCurrentVersion(fixer, input, version)`.
    pub fn update_to_current_version(
        &self,
        fixer: &dyn DataFixer,
        tag: CompoundTag,
        version: i32,
    ) -> CompoundTag {
        self.update(fixer, tag, version, Self::current_version())
    }

    /// `updateToCurrentVersion` для `CompoundTag` (Java-перегрузка).
    pub fn update_to_current_version_compound(
        &self,
        fixer: &dyn DataFixer,
        tag: &mut CompoundTag,
        version: i32,
    ) {
        let updated = self.update(fixer, std::mem::take(tag), version, Self::current_version());
        *tag = updated;
    }

    /// `wrapCodec(...).encode` — добавляет `DataVersion` к уже
    /// закодированному codec'ом тегу.
    pub fn wrap_encode(&self, tag: &mut CompoundTag) {
        tag.put_int("DataVersion", Self::current_version());
    }

    /// `wrapCodec(...).decode` — извлекает `DataVersion` (fallback =
    /// `data_version` текущего хранилища) и возвращает тег без него
    /// вместе с найденной версией. Дальнейший datafix — через
    /// [`update_to_current_version`](Self::update_to_current_version).
    pub fn wrap_decode(&self, tag: &CompoundTag, data_version: i32) -> (CompoundTag, i32) {
        let version = tag.get_int("DataVersion").unwrap_or(data_version);
        let mut cleaned = tag.clone();
        cleaned.remove("DataVersion");
        (cleaned, version)
    }
}
