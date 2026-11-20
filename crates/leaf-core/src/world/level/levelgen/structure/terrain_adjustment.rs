//! Перенос `TerrainAdjustment`.

/// `TerrainAdjustment` — как структура подгоняет террейн при
/// пост-обработке.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainAdjustment {
    /// `NONE`.
    None,
    /// `BEARD_THIN` — тонкая «борода» под строением.
    BeardThin,
    /// `BEARD_BOX` — коробка под строением.
    BeardBox,
    /// `ENCAPSULATE` — инкапсуляция (бастион).
    Encapsulate,
    /// `ENCAPSULATE_SOLID` — инкапсуляция сплошным блоком.
    EncapsulateSolid,
}

impl TerrainAdjustment {
    /// `isSet`.
    pub fn is_set(&self) -> bool {
        *self != TerrainAdjustment::None
    }
}
