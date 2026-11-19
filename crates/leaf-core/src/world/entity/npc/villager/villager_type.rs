//! Перенос `net.minecraft.world.entity.npc.villager.VillagerType` —
//! тип жителя по биому.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::core::holder::Holder;
use crate::core::registry::registries;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;

macro_rules! villager_type_key {
    ($name:ident, $str:literal) => {
        pub fn $name() -> ResourceKey<VillagerType> {
            ResourceKey::create_with_registry_name(
                registries::villager_type(),
                Identifier::minecraft($str),
            )
        }
    };
}

villager_type_key!(desert, "desert");
villager_type_key!(jungle, "jungle");
villager_type_key!(plains, "plains");
villager_type_key!(savanna, "savanna");
villager_type_key!(snow, "snow");
villager_type_key!(swamp, "swamp");
villager_type_key!(taiga, "taiga");

/// `VillagerType` — пустая запись (biome-маппинг отдельно).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VillagerType {
    pub id: Identifier,
}

impl VillagerType {
    /// `new`-эквивалент (в Java — record с biomePredicate).
    pub fn new(id: Identifier) -> Self {
        Self { id }
    }
}

/// `BY_BIOME` — biome key → villager type key.
pub fn by_biome(biome: &Identifier) -> ResourceKey<VillagerType> {
    static MAP: OnceLock<HashMap<&'static str, ResourceKey<VillagerType>>> = OnceLock::new();
    let map = MAP.get_or_init(|| {
        let mut m = HashMap::new();
        for b in [
            "badlands", "desert", "eroded_badlands", "wooded_badlands",
        ] {
            m.insert(b, desert());
        }
        for b in ["bamboo_jungle", "jungle", "sparse_jungle"] {
            m.insert(b, jungle());
        }
        for b in ["savanna_plateau", "savanna", "windswept_savanna"] {
            m.insert(b, savanna());
        }
        for b in [
            "deep_frozen_ocean", "frozen_ocean", "frozen_river", "ice_spikes",
            "snowy_beach", "snowy_taiga", "snowy_plains", "grove",
            "snowy_slopes", "frozen_peaks", "jagged_peaks",
        ] {
            m.insert(b, snow());
        }
        for b in ["swamp", "mangrove_swamp"] {
            m.insert(b, swamp());
        }
        for b in [
            "old_growth_spruce_taiga", "old_growth_pine_taiga",
            "windswept_gravelly_hills", "windswept_hills", "taiga",
            "windswept_forest",
        ] {
            m.insert(b, taiga());
        }
        m
    });
    map.get(biome.get_path()).cloned().unwrap_or_else(plains)
    // biome.get_path() → get_path()
}

/// Все ключи.
pub fn all_keys() -> Vec<ResourceKey<VillagerType>> {
    vec![desert(), jungle(), plains(), savanna(), snow(), swamp(), taiga()]
}

/// `get(key)` — Holder::Key-совместимое значение.
pub fn get(key: &ResourceKey<VillagerType>) -> Option<VillagerType> {
    let name = key.identifier.get_path();
    Some(VillagerType::new(key.identifier.clone()))
        .filter(|_| matches!(name, "desert" | "jungle" | "plains" | "savanna" | "snow" | "swamp" | "taiga"))
}

/// Holder этого типа.
pub fn holder(key: ResourceKey<VillagerType>) -> Holder<VillagerType> {
    Holder::Key(key)
}
