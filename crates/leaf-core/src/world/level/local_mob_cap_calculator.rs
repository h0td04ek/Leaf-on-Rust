//! Перенос `net.minecraft.world.level.LocalMobCapCalculator` —
//! per-player mob cap подсчёт для NaturalSpawner.
//!
//! Адаптация: Java ключирует по `ServerPlayer`-объектам через
//! `ChunkMap.getPlayersCloseForSpawning`; у нас игроки ещё не
//! портированы — ключ `Uuid`, а провайдер "игроки возле чанка"
//! инжектируется замыканием (`players_near_chunk` — кэш как в Java).

use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use crate::util::uuid::Uuid;
use crate::world::entity::mob_category::MobCategory;

/// `LocalMobCapCalculator.MobCounts` — счётчики по категориям.
#[derive(Default)]
pub struct MobCounts {
    /// `counts` — массив по `MobCategory.ordinal()`.
    counts: Vec<i32>,
}

impl MobCounts {
    fn new() -> Self {
        Self { counts: vec![0; MobCategory::VALUES.len()] }
    }

    /// `add(category)`.
    pub fn add(&mut self, category: MobCategory) {
        self.counts[category as usize] += 1;
    }

    /// `canSpawn(category)` — count < maxInstancesPerChunk.
    pub fn can_spawn(&self, category: MobCategory) -> bool {
        self.counts[category as usize] < category.get_max_instances_per_chunk()
    }
}

/// `LocalMobCapCalculator`.
pub struct LocalMobCapCalculator {
    /// `playersNearChunk` — кэш chunk → игроки рядом.
    players_near_chunk: HashMap<i64, Vec<Uuid>>,
    /// `playerMobCounts` — uuid игрока → счётчики мобов.
    player_mob_counts: HashMap<Uuid, MobCounts>,
    /// Провайдер `ChunkMap.getPlayersCloseForSpawning(chunkPos)`.
    players_provider: Rc<dyn Fn(i64) -> Vec<Uuid>>,
}

impl LocalMobCapCalculator {
    /// `new(chunkMap)` — в Java брал ChunkMap; здесь — замыкание,
    /// возвращающее uuid игроков, влияющих на спавн в чанке.
    pub fn new(players_provider: Rc<dyn Fn(i64) -> Vec<Uuid>>) -> Self {
        Self {
            players_near_chunk: HashMap::new(),
            player_mob_counts: HashMap::new(),
            players_provider,
        }
    }

    /// `getPlayersNear(pos)` — кэшированные игроки возле чанка.
    fn get_players_near(&mut self, chunk_key: i64) -> &[Uuid] {
        self.players_near_chunk
            .entry(chunk_key)
            .or_insert_with(|| (self.players_provider)(chunk_key))
            .as_slice()
    }

    /// `addMob(pos, category)` — инкремент счётчика у всех
    /// игроков возле чанка.
    pub fn add_mob(&mut self, chunk_key: i64, category: MobCategory) {
        // Клонируем список игроков, чтобы не держать borrow кэша.
        let players = self.get_players_near(chunk_key).to_vec();
        for player in players {
            self.player_mob_counts
                .entry(player)
                .or_insert_with(MobCounts::new)
                .add(category);
        }
    }

    /// `canSpawn(category, pos)` — хотя бы у одного игрока возле
    /// чанка категория не переполнена (или игроков нет → true).
    pub fn can_spawn(&mut self, category: MobCategory, chunk_key: i64) -> bool {
        let players = self.get_players_near(chunk_key).to_vec();
        for player in players {
            match self.player_mob_counts.get(&player) {
                Some(counts) if !counts.can_spawn(category) => continue,
                _ => return true,
            }
        }
        false
    }
}

// Suppress unused-import warning для Arc (оставлен для
// совместимости с сигнатурами соседних модулей).
#[allow(unused_imports)]
use std::sync::Arc as _Arc;
