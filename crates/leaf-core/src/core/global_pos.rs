//! Перенос `net.minecraft.core.GlobalPos`.

use crate::core::block_pos::BlockPos;
use crate::core::registry::Registry;
use crate::resources::ResourceKey;
use crate::world::level::LevelKey;

/// `GlobalPos(dimension, pos)` — позиция в конкретном измерении.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GlobalPos {
    pub dimension: ResourceKey<LevelKey>,
    pub pos: BlockPos,
}

impl GlobalPos {
    pub fn of(dimension: ResourceKey<LevelKey>, pos: BlockPos) -> Self {
        Self { dimension, pos }
    }

    pub fn is_close_enough(&self, dimension: &ResourceKey<LevelKey>, pos: &BlockPos, max_distance: i32) -> bool {
        self.dimension == *dimension && self.pos.to_vec3i().dist_chessboard(&pos.to_vec3i()) <= max_distance
    }
}

impl std::fmt::Display for GlobalPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.dimension, self.pos.to_vec3i())
    }
}

// Заглушка типа для компиляции ResourceKey<LevelKey>;
// полная реализация Level появится при порте world.level.
pub type LevelRegistry = Registry<LevelKey>;
