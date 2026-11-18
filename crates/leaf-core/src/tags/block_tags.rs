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
    /// `BlockTags.FIRE` — `minecraft:fire`.
    pub fn fire() -> Identifier {
        Identifier::minecraft("fire")
    }
    /// `BlockTags.TRAPDOORS` — `minecraft:trapdoors`.
    pub fn trapdoors() -> Identifier {
        Identifier::minecraft("trapdoors")
    }
    /// `BlockTags.FENCES` — `minecraft:fences`.
    pub fn fences() -> Identifier {
        Identifier::minecraft("fences")
    }
    /// `BlockTags.WALLS` — `minecraft:walls`.
    pub fn walls() -> Identifier {
        Identifier::minecraft("walls")
    }
    /// `BlockTags.DOORS` — `minecraft:doors`.
    pub fn doors() -> Identifier {
        Identifier::minecraft("doors")
    }
    /// `BlockTags.BEDS` — `minecraft:beds`.
    /// `cauldrons` — все котлы (для PathNavigation.trimPath).
    pub fn cauldrons() -> Identifier {
        Identifier::minecraft("cauldrons")
    }

    pub fn beds() -> Identifier {
        Identifier::minecraft("beds")
    }
}
