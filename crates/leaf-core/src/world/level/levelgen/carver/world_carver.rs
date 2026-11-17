//! `net.minecraft.world.level.levelgen.carver.WorldCarver` —
//! интерфейс карвера пещер.

/// `CarvingContext` — контекст карвинга (шов под ChunkGenerator).
pub trait CarvingContext: Send {}

/// `WorldCarver`.
pub trait WorldCarver: Send + Sync {
    /// `carve(context, chunk, ...)` — вырезать пещеру в чанке.
    fn carve(&self, context: &mut dyn CarvingContext);
}
