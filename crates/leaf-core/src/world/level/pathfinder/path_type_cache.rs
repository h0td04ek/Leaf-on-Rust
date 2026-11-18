//! Перенос `net.minecraft.world.level.pathfinder.PathTypeCache`.

use crate::core::block_pos::BlockPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use std::cell::RefCell;
use std::collections::HashMap;

/// `PathTypeCache` — кэш типов пути (vanilla — 4096 слотов
/// с миксом хэша; семантика LRU не критична — HashMap).
/// Внутренняя изменяемость — кэш разделяется через `&`.
#[derive(Default)]
pub struct PathTypeCache {
    cache: RefCell<HashMap<i64, PathType>>,
}

impl PathTypeCache {
    /// `new`.
    pub fn new() -> Self {
        Self::default()
    }

    /// `getOrCompute`.
    pub fn get_or_compute(&self, level: &dyn LevelReader, pos: BlockPos) -> PathType {
        let key = pos.as_long();
        if let Some(t) = self.cache.borrow().get(&key) {
            return *t;
        }
        let t = WalkNodeEvaluator::get_path_type_from_state(level, &pos);
        self.cache.borrow_mut().insert(key, t);
        t
    }

    /// `invalidate`.
    pub fn invalidate(&self, pos: BlockPos) {
        self.cache.borrow_mut().remove(&pos.as_long());
    }
}
