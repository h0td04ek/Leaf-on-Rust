//! Перенос `net.minecraft.world.level.levelgen.structure.Structure` —
//! структура генерации (record-тип реестра).

use crate::resources::identifier::Identifier;
use std::sync::Arc;

/// `Structure` — запись реестра структур.
///
/// Полная логика (findGenerationPoint, settings, terrainAdaptation,
/// liquidSettings, spawnOverrides, biomeSource) портируется по мере
/// генератора; здесь — идентичность структуры.
#[derive(Clone, Debug)]
pub struct Structure {
    /// `id` — identifier структуры.
    pub id: Identifier,
}

impl Structure {
    /// `new(id)`.
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}

/// Реестр структур — id → Arc<Structure>.
pub fn structure_registry() -> &'static std::sync::RwLock<Vec<Arc<Structure>>> {
    static REG: std::sync::OnceLock<std::sync::RwLock<Vec<Arc<Structure>>>> =
        std::sync::OnceLock::new();
    REG.get_or_init(|| std::sync::RwLock::new(Vec::new()))
}
