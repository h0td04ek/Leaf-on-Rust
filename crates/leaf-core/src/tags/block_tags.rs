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
    /// `BlockTags.EDIBLE_FOR_SHEEP` — растения, которые ест овца.
    pub fn edible_for_sheep() -> Identifier {
        Identifier::minecraft("edible_for_sheep")
    }
}

/// `bootstrap()` — ванильное содержимое используемых блочных
/// тегов (плоские списки id блоков).
pub fn bootstrap() {
    use crate::resources::identifier::Identifier as Id;
    let ids = |names: &[&str]| {
        names.iter().map(|n| Id::with_default_namespace(n).unwrap()).collect::<Vec<_>>()
    };
    let colors = [
        "white", "orange", "magenta", "light_blue", "yellow", "lime",
        "pink", "gray", "light_gray", "cyan", "purple", "blue",
        "brown", "green", "red", "black",
    ];
    let beds: Vec<Id> = colors.iter().map(|c| Id::with_default_namespace(&format!("{c}_bed")).unwrap()).collect();
    crate::tags::register_block_tag(Id::with_default_namespace("beds").unwrap(), beds);
    crate::tags::register_block_tag(
        Id::with_default_namespace("edible_for_sheep").unwrap(),
        ids(&["short_grass", "fern", "large_fern", "dead_bush", "seagrass", "tall_seagrass"]),
    );
}

/// `bootstrap_once()` — идемпотентный bootstrap.
pub fn bootstrap_once() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(bootstrap);
}
