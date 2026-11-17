//! Перенос `net.minecraft.world.level.block.state.properties.StructureMode`.

/// `StructureMode` — 4 значения в порядке объявления.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StructureMode {
    /// `SAVE`.
    SAVE,
    /// `LOAD`.
    LOAD,
    /// `CORNER`.
    CORNER,
    /// `DATA`.
    DATA,
}

impl StructureMode {
    /// `values()`.
    pub const VALUES: [StructureMode; 4] = [
        StructureMode::SAVE,
        StructureMode::LOAD,
        StructureMode::CORNER,
        StructureMode::DATA,
    ];

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            StructureMode::SAVE => "save",
            StructureMode::LOAD => "load",
            StructureMode::CORNER => "corner",
            StructureMode::DATA => "data",
        }
    }

    /// `CODEC.byName`.
    pub fn by_name(name: &str) -> Option<StructureMode> {
        Self::VALUES.iter().copied().find(|m| m.get_serialized_name() == name)
    }
}
