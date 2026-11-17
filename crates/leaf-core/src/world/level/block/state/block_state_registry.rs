//! `Block.BLOCK_STATE_REGISTRY` — `IdMap<BlockState>` всех
//! зарегистрированных состояний блоков (порядок = порядок регистрации
//! при bootstrap'е `Blocks`).

use super::block_state::BlockState;
use super::super::super::chunk::palette::IdMap;
use crate::world::level::block::Block;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Глобальный реестр block states: `Vec<BlockState>` + обратный индекс.
struct BlockStateRegistry {
    states: Vec<BlockState>,
    index: HashMap<BlockState, usize>,
}

fn registry() -> &'static RwLock<BlockStateRegistry> {
    static REG: std::sync::OnceLock<RwLock<BlockStateRegistry>> = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        RwLock::new(BlockStateRegistry { states: Vec::new(), index: HashMap::new() })
    })
}

/// Регистрация всех состояний блока (вызывается при bootstrap'е блока).
pub fn register_block_states(block: &Arc<Block>) {
    let definition = match block.state_definition() {
        Some(d) => d.clone(),
        None => return,
    };
    let mut reg = registry().write().unwrap();
    for holder in definition.possible_states() {
        // Собираем BlockState из StateHolder
        let state = BlockState::new(block.clone(), holder.clone());
        if !reg.index.contains_key(&state) {
            let len = reg.states.len();
            reg.index.insert(state.clone(), len);
            reg.states.push(state);
        }
    }
}

/// Число зарегистрированных состояний.
pub fn block_state_count() -> usize {
    registry().read().unwrap().states.len()
}

/// BlockState по id.
pub fn block_state_by_id(id: i32) -> Option<BlockState> {
    registry().read().unwrap().states.get(id as usize).cloned()
}

/// IdMap для `PalettedContainer<BlockState>`.
pub struct BlockStateIdMap;

impl IdMap<BlockState> for BlockStateIdMap {
    fn by_id(&self, id: i32) -> Option<BlockState> {
        block_state_by_id(id)
    }
    fn get_id(&self, value: &BlockState) -> i32 {
        registry()
            .read()
            .unwrap()
            .index
            .get(value)
            .map(|&i| i as i32)
            .unwrap_or(-1)
    }
    fn size(&self) -> usize {
        block_state_count()
    }
}

/// Shared IdMap для контейнеров.
pub fn block_state_id_map() -> std::sync::Arc<dyn IdMap<BlockState>> {
    std::sync::Arc::new(BlockStateIdMap)
}
