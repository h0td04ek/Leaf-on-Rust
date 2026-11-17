//! Перенос `net.minecraft.SharedConstants` + `net.minecraft.WorldVersion`
//! (константы версии 1.21.11 / protocol 774).

use std::time::SystemTime;

/// `WorldVersion` — версия игры.
#[derive(Clone, Debug)]
pub struct WorldVersion {
    /// `id` — например `1.21.11`.
    pub id: String,
    /// `name` — отображаемое имя (`1.21.11` / snapshot name).
    pub name: String,
    /// `dataVersion` — `DataVersion` мира (SNBT_NAG_VERSION-серия).
    pub data_version: i32,
    /// `protocolVersion`.
    pub protocol_version: i32,
    /// `resourcePackVersion` (`PackFormat`).
    pub resource_pack_version: PackFormat,
    /// `datapackVersion` (`PackFormat`).
    pub datapack_version: PackFormat,
    /// `buildTime`.
    pub build_time: SystemTime,
    /// `stable`.
    pub stable: bool,
}

/// `PackFormat` — (major, minor) формат пакета.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackFormat {
    pub major: i32,
    pub minor: i32,
}

impl PackFormat {
    /// `new PackFormat(major, minor)`.
    pub fn new(major: i32, minor: i32) -> Self {
        Self { major, minor }
    }
}

/// Текущая версия Leaf 1.21.11.
pub const CURRENT_VERSION_NAME: &str = "1.21.11";
pub const CURRENT_VERSION_ID: &str = "1.21.11";
pub const CURRENT_DATA_VERSION: i32 = 4671;
pub const CURRENT_PROTOCOL_VERSION: i32 = 774;
pub const RESOURCE_PACK_FORMAT_MAJOR: i32 = 75;
pub const RESOURCE_PACK_FORMAT_MINOR: i32 = 0;
pub const DATA_PACK_FORMAT_MAJOR: i32 = 94;
pub const DATA_PACK_FORMAT_MINOR: i32 = 1;

/// `SharedConstants.getCurrentVersion()` — в Java версия определяется
/// при старте (`setVersion`/`tryDetectVersion`); здесь — константа сборки.
pub fn current_version() -> WorldVersion {
    WorldVersion {
        id: CURRENT_VERSION_ID.to_string(),
        name: CURRENT_VERSION_NAME.to_string(),
        data_version: CURRENT_DATA_VERSION,
        protocol_version: CURRENT_PROTOCOL_VERSION,
        resource_pack_version: PackFormat::new(RESOURCE_PACK_FORMAT_MAJOR, RESOURCE_PACK_FORMAT_MINOR),
        datapack_version: PackFormat::new(DATA_PACK_FORMAT_MAJOR, DATA_PACK_FORMAT_MINOR),
        build_time: SystemTime::UNIX_EPOCH,
        stable: true,
    }
}

/// `SharedConstants.getProtocolVersion()`.
pub fn get_protocol_version() -> i32 {
    CURRENT_PROTOCOL_VERSION
}
