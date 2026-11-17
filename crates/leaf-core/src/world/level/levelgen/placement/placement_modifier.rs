//! `net.minecraft.world.level.levelgen.placement.PlacementModifier` —
//! модификатор размещения фичи.

use crate::core::block_pos::BlockPos;

/// `PlacementContext` — контекст placement (шов под WorldGenLevel).
pub trait PlacementContext: Send {
    /// `level()` — уровень (шов).
    fn level(&self) -> &dyn PlacementLevelAccess;
    /// `chunkGenerator()` — генератор (шов).
    fn chunk_generator(&self) -> &dyn PlacementGeneratorAccess;
    /// `random()` — источник случайности.
    fn random(&self) -> &mut dyn crate::random::RandomSource;
    /// `origin()` — origin.
    fn origin(&self) -> BlockPos;
}

/// Доступ к уровню (шов).
pub trait PlacementLevelAccess: Send + Sync {}
/// Доступ к генератору (шов).
pub trait PlacementGeneratorAccess: Send + Sync {}

/// `PlacementModifier`.
pub trait PlacementModifier: Send + Sync {
    /// `getPlace(context, pos)` — количество попыток размещения.
    fn get_place(&self, context: &mut dyn PlacementContext, pos: &mut BlockPos) -> i32;
}
