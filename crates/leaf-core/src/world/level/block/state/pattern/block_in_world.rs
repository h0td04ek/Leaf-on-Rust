//! Перенос `net.minecraft.world.level.block.state.pattern.BlockInWorld`
//! — кэшированное состояние блока в мире для предикатов.

use crate::core::block_pos::BlockPos;
use crate::world::level::block::entity::block_entity::BlockEntityRef;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;
use std::sync::{Arc, Mutex};

/// `BlockInWorld` — ленивый доступ к `BlockState`/`BlockEntity`
/// с кэшированием (Java mutable class).
pub struct BlockInWorld {
    level: Arc<dyn LevelReader>,
    pos: BlockPos,
    load_chunks: bool,
    state: Mutex<Option<Option<BlockState>>>,
    entity: Mutex<Option<Option<BlockEntityRef>>>,
}

impl BlockInWorld {
    /// `new BlockInWorld(level, pos, loadChunks)`.
    pub fn new(level: Arc<dyn LevelReader>, pos: BlockPos, load_chunks: bool) -> Self {
        Self {
            level,
            pos: pos.immutable(),
            load_chunks,
            state: Mutex::new(None),
            entity: Mutex::new(None),
        }
    }

    /// `getState` — `None` если чанк недоступен и не грузится.
    pub fn get_state(&self) -> Option<BlockState> {
        let mut cache = self.state.lock().unwrap();
        if cache.is_none() {
            let loaded = self.level.has_chunk_at(self.pos) || self.load_chunks;
            *cache = Some(if loaded { self.level.get_block_state(self.pos) } else { None });
        }
        cache.clone().flatten()
    }

    /// `getEntity` — block entity (кэшируется, включая None).
    pub fn get_entity(&self) -> Option<BlockEntityRef> {
        let mut cache = self.entity.lock().unwrap();
        if cache.is_none() {
            *cache = Some(self.level.get_block_entity(self.pos));
        }
        cache.clone().flatten()
    }

    /// `getLevel`.
    pub fn level(&self) -> &Arc<dyn LevelReader> {
        &self.level
    }

    /// `getPos`.
    pub fn pos(&self) -> BlockPos {
        self.pos
    }
}

/// `BlockInWorld.hasState(predicate)` — фабрика предиката.
pub fn has_state(
    pred: impl Fn(&BlockState) -> bool + 'static,
) -> impl Fn(Option<&BlockInWorld>) -> bool {
    move |b: Option<&BlockInWorld>| b.is_some_and(|b| b.get_state().is_some_and(|s| pred(&s)))
}
