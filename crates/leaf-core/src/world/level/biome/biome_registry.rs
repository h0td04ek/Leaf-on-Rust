//! `net.minecraft.world.level.biome.Biomes`/registry — реестр биомов для палитр чанков.
//!
//! Java: биомы регистрируются через `Registry<Biome>` (bootstrap/datapack).
//! Здесь — `OnceLock`-реестр `Holder<Biome>`, заполняемый при bootstrap;
//! `biome_holder_id_map` даёт `IdMap` для `PalettedContainer<Holder<Biome>>`.

use crate::core::holder::Holder;
use crate::world::level::biome::biome::Biome;
use crate::world::level::chunk::palette::{IdMap, SharedIdMap};
use std::sync::{Arc, OnceLock, RwLock};

static REGISTRY: OnceLock<RwLock<Vec<Holder<Biome>>>> = OnceLock::new();

fn registry() -> &'static RwLock<Vec<Holder<Biome>>> {
    REGISTRY.get_or_init(|| RwLock::new(Vec::new()))
}

/// `register` — регистрация биома, возвращает id (как в Java registry).
pub fn register(holder: Holder<Biome>) -> i32 {
    let mut reg = registry().write().unwrap();
    if let Some(pos) = reg.iter().position(|h| h == &holder) {
        return pos as i32;
    }
    reg.push(holder);
    (reg.len() - 1) as i32
}

/// `biomeCount`.
pub fn biome_count() -> usize {
    registry().read().unwrap().len()
}

/// `biomeHolderIdMap` — глобальная `IdMap` для палитр.
pub fn biome_holder_id_map() -> SharedIdMap<Holder<Biome>> {
    static MAP: OnceLock<SharedIdMap<Holder<Biome>>> = OnceLock::new();
    MAP.get_or_init(|| Arc::new(BiomeHolderIdMap)).clone()
}

struct BiomeHolderIdMap;

impl IdMap<Holder<Biome>> for BiomeHolderIdMap {
    fn by_id(&self, id: i32) -> Option<Holder<Biome>> {
        registry().read().unwrap().get(id as usize).cloned()
    }

    fn get_id(&self, value: &Holder<Biome>) -> i32 {
        match value {
            Holder::Reference(id) => *id,
            _ => registry()
                .read()
                .unwrap()
                .iter()
                .position(|h| h == value)
                .map(|i| i as i32)
                .unwrap_or(-1),
        }
    }

    fn size(&self) -> usize {
        biome_count()
    }
}
