//! `DataFixer` — порт DFU-интерфейса `com.mojang.datafixers.DataFixer`
//! в виде трейта над NBT-тегами.
//!
//! Java-сигнатура `update(TypeReference, Dynamic<T>, int, int)` здесь
//! принимает [`DataFixTypes`](crate::util::datafix::DataFixTypes)
//! вместо `TypeReference` и `CompoundTag` вместо `Dynamic<NbtOps>`.

use crate::nbt::compound::CompoundTag;
use crate::util::datafix::data_fix_types::DataFixTypes;

/// Минимальный контракт data fixer'а. Реальная DFU-система будет
/// подключена позже; существующие потребители (saved data, legacy
/// structure fix) работают через этот трейт.
pub trait DataFixer: Send + Sync {
    /// `update(type, input, oldVersion, newVersion)` — приводит тег
    /// от `old_version` к `new_version` для данного типа данных.
    fn update(
        &self,
        data_fix_type: DataFixTypes,
        tag: CompoundTag,
        old_version: i32,
        new_version: i32,
    ) -> CompoundTag;
}
