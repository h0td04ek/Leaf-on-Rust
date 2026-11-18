//! Перенос `net.minecraft.world.level.pathfinder.PathfindingContext`.
//!
//! Java-версия держит `CollisionGetter`/`PathNavigationRegion` и
//! `Mob`; здесь — `&dyn LevelReader` + позиция моба + кэш
//! `PathType` (если уровень серверный — кэш живёт в Level,
//! здесь опциональный `&mut`-контекст не требуется: кэш
//! передаётся снаружи через `PathTypeCacheOwner`).

use crate::core::block_pos::BlockPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::path_type_cache::PathTypeCache;
use crate::world::level::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use std::sync::Arc;

/// `PathfindingContext` — контекст поиска пути.
pub struct PathfindingContext<'a> {
    /// `level` — `CollisionGetter` (здесь `LevelReader`).
    pub level: &'a dyn LevelReader,
    /// `cache` — `PathTypeCache` уровня (опционально).
    pub cache: Option<&'a PathTypeCache>,
    /// `mobPosition`.
    pub mob_position: BlockPos,
}

impl<'a> PathfindingContext<'a> {
    /// `new PathfindingContext(level, mob)`.
    ///
    /// `mob_position` — `mob.blockPosition()`, `cache` — кэш
    /// уровня если есть (в Java — только для `ServerLevel`).
    pub fn new(
        level: &'a dyn LevelReader,
        cache: Option<&'a PathTypeCache>,
        mob_position: BlockPos,
    ) -> Self {
        Self {
            level,
            cache,
            mob_position,
        }
    }

    /// `getPathTypeFromState(x, y, z)`.
    pub fn get_path_type_from_state(&mut self, x: i32, y: i32, z: i32) -> PathType {
        let pos = BlockPos::new(x, y, z);
        match self.cache {
            Some(cache) => cache.get_or_compute(self.level, pos),
            None => WalkNodeEvaluator::get_path_type_from_state(self.level, &pos),
        }
    }

    /// `getBlockState`.
    pub fn get_block_state(&self, pos: BlockPos) -> Option<BlockState> {
        self.level.get_block_state(pos)
    }

    /// `level()`.
    pub fn level(&self) -> &dyn LevelReader {
        self.level
    }

    /// `mobPosition()`.
    pub fn mob_position(&self) -> BlockPos {
        self.mob_position
    }
}
