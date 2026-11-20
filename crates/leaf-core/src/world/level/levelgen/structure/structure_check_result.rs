//! Перенос `net.minecraft.world.level.levelgen.structure.StructureCheckResult`.

/// `StructureCheckResult` — результат проверки структуры в чанке.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StructureCheckResult {
    /// `START_PRESENT` — структура уже сгенерирована в чанке.
    StartPresent,
    /// `START_NOT_PRESENT` — структуры нет.
    StartNotPresent,
    /// `CHUNK_LOAD_NEEDED` — нужно загрузить чанк для проверки.
    ChunkLoadNeeded,
}
