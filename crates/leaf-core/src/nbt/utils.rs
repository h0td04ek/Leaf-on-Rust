//! Перенос `net.minecraft.nbt.NbtUtils` — подмножество: DataVersion-хелперы.
//!
//! Полный `NbtUtils` (block-state/позиционные кодеки, SNBT-escape и пр.)
//! портируется вместе с соответствующими системами.

use crate::core::shared_constants::CURRENT_DATA_VERSION;
use crate::nbt::compound::CompoundTag;

/// `NbtUtils.getDataVersion(tag)` — default -1.
pub fn get_data_version(tag: &CompoundTag) -> i32 {
    get_data_version_or(tag, -1)
}

/// `NbtUtils.getDataVersion(tag, defaultValue)`.
pub fn get_data_version_or(tag: &CompoundTag, default_value: i32) -> i32 {
    tag.get_int_or("DataVersion", default_value)
}

/// `NbtUtils.addDataVersion(tag, dataVersion)` — возвращает тот же тег.
pub fn add_data_version(tag: &mut CompoundTag, data_version: i32) {
    tag.put_int("DataVersion", data_version);
}

/// `NbtUtils.addCurrentDataVersion(tag)`.
pub fn add_current_data_version(tag: &mut CompoundTag) {
    add_data_version(tag, CURRENT_DATA_VERSION);
}
