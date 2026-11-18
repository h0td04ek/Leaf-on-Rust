//! Перенос `net.minecraft.tags` — ключи и резолверы тегов.
//!
//! Теги data-driven: `register_*_tag(location, members)` наполняется
//! при bootstrap'е реестров (из datapack/ванильных данных); до этого
//! членство в именованных тегHolderSet'ах — false.

pub mod block_tags;
pub mod damage_type_tags;
pub mod entity_type_tags;
pub mod fluid_tags;
pub mod tag_key;

pub use block_tags::BlockTags;
pub use damage_type_tags as damage_type_tags_mod;
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

/// Глобальный реестр флюид-тегов: `location -> set<fluid id>`.
fn fluid_tag_registry() -> &'static Mutex<HashMap<Identifier, Vec<Identifier>>> {
    static REG: OnceLock<Mutex<HashMap<Identifier, Vec<Identifier>>>> = OnceLock::new();
    REG.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Зарегистрировать флюид-тег.
pub fn register_fluid_tag(location: Identifier, members: Vec<Identifier>) {
    fluid_tag_registry().lock().unwrap().insert(location, members);
}

/// `fluidState.is(FluidTags.X)` для именованного тега.
pub fn is_fluid_in_named_tag(fluid_id: &Identifier, tag_location: &Identifier) -> bool {
    let reg = fluid_tag_registry().lock().unwrap();
    reg.get(tag_location)
        .map(|members| members.iter().any(|m| m == fluid_id))
        .unwrap_or(false)
}
