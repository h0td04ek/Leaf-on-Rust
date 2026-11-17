//! Перенос `net.minecraft.tags.BlockTags` — константы тегов блоков.
//!
//! Членство резолвится через `tags::is_block_in_named_tag` (реестр
//! заполняется при bootstrap'е datapack/ванильных тегов).

use crate::resources::identifier::Identifier;

/// `BlockTags` — локации ванильных блочных тегов (подмножество,
/// расширяется по мере порта).
pub struct BlockTags;

impl BlockTags {
    /// `BlockTags.LEAVES` — `minecraft:leaves`.
    pub fn leaves() -> Identifier {
        Identifier::minecraft("leaves")
    }
    /// `BlockTags.LOGS` — `minecraft:logs`.
    pub fn logs() -> Identifier {
        Identifier::minecraft("logs")
    }
    /// `BlockTags.SNOW` — `minecraft:snow`.
    pub fn snow() -> Identifier {
        Identifier::minecraft("snow")
    }
}
