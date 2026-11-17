//! Перенос `net.minecraft.world.level.block.entity.StructureBlockEntity.UpdateType`.

/// `StructureBlockEntity.UpdateType`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureBlockUpdateType {
    /// `UPDATE_DATA`.
    UPDATE_DATA,
    /// `SAVE_AREA`.
    SAVE_AREA,
    /// `LOAD_AREA`.
    LOAD_AREA,
    /// `SCAN_AREA`.
    SCAN_AREA,
}

impl StructureBlockUpdateType {
    /// `values()`.
    pub const VALUES: [StructureBlockUpdateType; 4] = [
        StructureBlockUpdateType::UPDATE_DATA,
        StructureBlockUpdateType::SAVE_AREA,
        StructureBlockUpdateType::LOAD_AREA,
        StructureBlockUpdateType::SCAN_AREA,
    ];

    /// `byOrdinal` — `readEnum`/`writeEnum` (VarInt ordinal).
    pub fn by_ordinal(ordinal: i32) -> StructureBlockUpdateType {
        Self::VALUES[ordinal.rem_euclid(4) as usize]
    }
}

/// Маркер `StructureBlockEntity`.
#[derive(Debug)]
pub struct StructureBlockEntity;
