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
    /// `BlockTags.STAIRS` — все лестницы.
    pub fn stairs() -> Identifier {
        Identifier::minecraft("stairs")
    }
    /// `BlockTags.SLABS` — все плиты.
    pub fn slabs() -> Identifier {
        Identifier::minecraft("slabs")
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
    // stairs/slabs — все варианты лестниц и плит.
    let woods = [
        "oak", "spruce", "birch", "jungle", "acacia", "dark_oak", "mangrove",
        "cherry", "bamboo", "bamboo_mosaic", "crimson", "warped",
    ];
    let mut stairs: Vec<Id> = Vec::new();
    let mut slabs: Vec<Id> = Vec::new();
    for w in woods {
        stairs.push(Id::with_default_namespace(&format!("{w}_stairs")).unwrap());
        slabs.push(Id::with_default_namespace(&format!("{w}_slab")).unwrap());
    }
    for s in [
        "stone_stairs", "stone_slab", "cobblestone_stairs", "cobblestone_slab",
        "mossy_cobblestone_stairs", "mossy_cobblestone_slab", "stone_brick_stairs",
        "stone_brick_slab", "mossy_stone_brick_stairs", "mossy_stone_brick_slab",
        "brick_stairs", "brick_slab", "nether_brick_stairs", "nether_brick_slab",
        "sandstone_stairs", "sandstone_slab", "red_sandstone_stairs",
        "red_sandstone_slab", "purpur_stairs", "purpur_slab", "quartz_stairs",
        "quartz_slab", "smooth_quartz_stairs", "smooth_quartz_slab",
        "prismarine_stairs", "prismarine_slab", "dark_prismarine_stairs",
        "dark_prismarine_slab", "polished_granite_stairs", "polished_granite_slab",
        "polished_diorite_stairs", "polished_diorite_slab", "polished_andesite_stairs",
        "polished_andesite_slab", "polished_deepslate_stairs", "polished_deepslate_slab",
        "deepslate_tile_stairs", "deepslate_tile_slab", "cobbled_deepslate_stairs",
        "cobbled_deepslate_slab", "deepslate_brick_stairs", "deepslate_brick_slab",
        "mud_brick_stairs", "mud_brick_slab", "resin_brick_stairs", "resin_brick_slab",
        "tuff_stairs", "tuff_slab", "polished_tuff_stairs", "polished_tuff_slab",
        "calcite_stairs", "calcite_slab",
    ] {
        if s.ends_with("_stairs") { stairs.push(Id::with_default_namespace(s).unwrap()); }
        else if s.ends_with("_slab") { slabs.push(Id::with_default_namespace(s).unwrap()); }
        else { slabs.push(Id::with_default_namespace(s).unwrap()); }
    }
    crate::tags::register_block_tag(Id::with_default_namespace("stairs").unwrap(), stairs);
    crate::tags::register_block_tag(Id::with_default_namespace("slabs").unwrap(), slabs);
}

/// `bootstrap_once()` — идемпотентный bootstrap.
pub fn bootstrap_once() {
    use std::sync::Once;
    static ONCE: Once = Once::new();
    ONCE.call_once(bootstrap);
}
