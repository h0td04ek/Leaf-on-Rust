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
    /// `BlockTags.WOODEN_DOORS` — `minecraft:wooden_doors`.
    pub fn wooden_doors() -> Identifier {
        Identifier::minecraft("wooden_doors")
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

    /// `BlockTags.PIGLIN_REPELLENTS` — soul_* блоки.
    pub fn piglin_repellents() -> Identifier {
        Identifier::minecraft("piglin_repellents")
    }
    /// `BlockTags.HOGLIN_REPELLENTS` — warped/portal/anchor.
    pub fn hoglin_repellents() -> Identifier {
        Identifier::minecraft("hoglin_repellents")
    }
}

/// `bootstrap()` — ванильное содержимое используемых блочных
/// тегов (плоские списки id блоков).
pub fn bootstrap() {
    use crate::resources::identifier::Identifier as Id;
    let ids = |names: &[&str]| names.iter().map(|n| Id::minecraft(n)).collect::<Vec<_>>();
    // wooden_doors = все деревянные двери (не iron/copper).
    crate::tags::register_block_tag(
        Id::minecraft("wooden_doors"),
        ids(&[
            "oak_door", "spruce_door", "birch_door", "jungle_door",
            "acacia_door", "dark_oak_door", "mangrove_door", "cherry_door",
            "bamboo_door", "bamboo_mosaic_door", "crimson_door", "warped_door",
        ]),
    );
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
    // piglin_repellents (data/minecraft/tags/block/piglin_repellents.json)
    crate::tags::register_block_tag(
        Id::with_default_namespace("piglin_repellents").unwrap(),
        ids(&["soul_fire", "soul_torch", "soul_lantern", "soul_wall_torch", "soul_campfire"]),
    );
    // snaps_goat_horn (RamTarget — RamTarget.hasRammedHornBreakingBlock)
    crate::tags::register_block_tag(
        Id::with_default_namespace("snaps_goat_horn").unwrap(),
        ids(&[
            "stone", "deepslate", "tuff", "calcite", "dripstone_block",
            "granite", "diorite", "andesite", "cobbled_deepslate",
        ]),
    );
    // hoglin_repellents
    crate::tags::register_block_tag(
        Id::with_default_namespace("hoglin_repellents").unwrap(),
        ids(&["warped_fungus", "potted_warped_fungus", "nether_portal", "respawn_anchor"]),
    );
}

/// `bootstrap_once()` — идемпотентный bootstrap.
pub fn bootstrap_once() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(bootstrap);
}
