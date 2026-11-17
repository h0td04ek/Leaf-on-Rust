//! Перенос `net.minecraft.tags` — ключи и резолверы тегов.
//!
//! Теги data-driven: `register_*_tag(location, members)` наполняется
//! при bootstrap'е реестров (из datapack/ванильных данных); до этого
//! членство в именованных тегHolderSet'ах — false.

pub mod block_tags;
pub mod tag_key;

pub use block_tags::BlockTags;
pub use tag_key::TagKey;

use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// Глобальный реестр блочных тегов: `location -> set<block id>`.
fn block_tag_registry() -> &'static Mutex<HashMap<Identifier, Vec<Identifier>>> {
    static REG: OnceLock<Mutex<HashMap<Identifier, Vec<Identifier>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Зарегистрировать блочный тег (аналог `TagLoader`-результата).
pub fn register_block_tag(location: Identifier, members: Vec<Identifier>) {
    block_tag_registry().lock().unwrap().insert(location, members);
}

/// `blockState.is(BlockTags.X)` для именованного тега —
/// проверка членства блока в теге по registry-локации.
pub fn is_block_in_named_tag(block_id: &Identifier, tag_location: &Identifier) -> bool {
    let reg = block_tag_registry().lock().unwrap();
    reg.get(tag_location)
        .map(|members| members.iter().any(|m| m == block_id))
        .unwrap_or(false)
}
