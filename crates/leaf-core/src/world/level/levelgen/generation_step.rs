//! `net.minecraft.world.level.levelgen.GenerationStep` —
//! шаги генерации декораций.

use crate::util::string_representable::StringRepresentable;

/// `GenerationStep.Decoration`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GenerationStep {
    /// `RAW_GENERATION`.
    RawGeneration,
    /// `LAKES`.
    Lakes,
    /// `LOCAL_MODIFICATIONS`.
    LocalModifications,
    /// `UNDERGROUND_STRUCTURES`.
    UndergroundStructures,
    /// `SURFACE_STRUCTURES`.
    SurfaceStructures,
    /// `STRONGHOLDS`.
    Strongholds,
    /// `UNDERGROUND_ORES`.
    UndergroundOres,
    /// `UNDERGROUND_DECORATION`.
    UndergroundDecoration,
    /// `FLUID_SPRINGS`.
    FluidSprings,
    /// `VEGETAL_DECORATION`.
    VegetalDecoration,
    /// `TOP_LAYER_MODIFICATION`.
    TopLayerModification,
}

/// Внутренний enum Java `GenerationStep.Decoration` — алиас для
/// совместимости с именованием `Decoration`.
pub type Decoration = GenerationStep;

impl GenerationStep {
    /// Все значения в порядке объявления.
    pub const VALUES: [GenerationStep; 11] = [
        GenerationStep::RawGeneration,
        GenerationStep::Lakes,
        GenerationStep::LocalModifications,
        GenerationStep::UndergroundStructures,
        GenerationStep::SurfaceStructures,
        GenerationStep::Strongholds,
        GenerationStep::UndergroundOres,
        GenerationStep::UndergroundDecoration,
        GenerationStep::FluidSprings,
        GenerationStep::VegetalDecoration,
        GenerationStep::TopLayerModification,
    ];

    /// `ordinal()`.
    pub fn ordinal(&self) -> usize {
        *self as usize
    }
}

impl StringRepresentable for GenerationStep {
    fn get_serialized_name(&self) -> String {
        match self {
            GenerationStep::RawGeneration => "raw_generation".to_string(),
            GenerationStep::Lakes => "lakes".to_string(),
            GenerationStep::LocalModifications => "local_modifications".to_string(),
            GenerationStep::UndergroundStructures => "underground_structures".to_string(),
            GenerationStep::SurfaceStructures => "surface_structures".to_string(),
            GenerationStep::Strongholds => "strongholds".to_string(),
            GenerationStep::UndergroundOres => "underground_ores".to_string(),
            GenerationStep::UndergroundDecoration => "underground_decoration".to_string(),
            GenerationStep::FluidSprings => "fluid_springs".to_string(),
            GenerationStep::VegetalDecoration => "vegetal_decoration".to_string(),
            GenerationStep::TopLayerModification => "top_layer_modification".to_string(),
        }
    }
}
