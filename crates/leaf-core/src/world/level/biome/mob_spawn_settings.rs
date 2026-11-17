//! `net.minecraft.world.level.biome.MobSpawnSettings` — настройки спавна мобов биома.

use crate::entity::entity_type::EntityType;
use crate::util::random::weighted_list::{WeightedList, WeightedListBuilder};
use crate::world::entity::mob_category::MobCategory;
use std::collections::HashMap;
use std::sync::Arc;

/// `MobSpawnSettings.EMPTY_MOB_LIST`.
pub fn empty_mob_list() -> WeightedList<SpawnerData> {
    WeightedList::empty()
}

/// `MobSpawnSettings.EMPTY`.
pub fn empty() -> MobSpawnSettings {
    MobSpawnSettingsBuilder::new().build()
}

/// `MobSpawnSettings`.
#[derive(Clone, Debug, Default)]
pub struct MobSpawnSettings {
    /// `creatureGenerationProbability`.
    pub creature_generation_probability: f32,
    /// `spawners` — категория → weighted list SpawnerData.
    pub spawners: HashMap<MobCategory, WeightedList<SpawnerData>>,
    /// `mobSpawnCosts`.
    pub mob_spawn_costs: HashMap<Arc<EntityType>, MobSpawnCost>,
}

impl MobSpawnSettings {
    /// `getMobs(category)`.
    pub fn get_mobs(&self, category: MobCategory) -> WeightedList<SpawnerData> {
        self.spawners.get(&category).cloned().unwrap_or_else(empty_mob_list)
    }

    /// `getMobSpawnCost(entityType)`.
    pub fn get_mob_spawn_cost(&self, entity_type: &EntityType) -> Option<&MobSpawnCost> {
        self.mob_spawn_costs.get(entity_type)
    }

    /// `getCreatureProbability`.
    pub fn get_creature_probability(&self) -> f32 {
        self.creature_generation_probability
    }
}

/// `MobSpawnSettings.MobSpawnCost(energyBudget, charge)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MobSpawnCost {
    /// `energyBudget`.
    pub energy_budget: f64,
    /// `charge`.
    pub charge: f64,
}

/// `MobSpawnSettings.SpawnerData(type, minCount, maxCount)`.
#[derive(Clone, Debug, PartialEq)]
pub struct SpawnerData {
    /// `type`.
    pub entity_type: Arc<EntityType>,
    /// `minCount`.
    pub min_count: i32,
    /// `maxCount`.
    pub max_count: i32,
}

impl SpawnerData {
    /// `new SpawnerData` — MISC-категория заменяется на свинью.
    pub fn new(entity_type: Arc<EntityType>, min_count: i32, max_count: i32) -> Self {
        let entity_type = if entity_type.get_category() == MobCategory::Misc {
            crate::entity::entity_type::pig().unwrap_or(entity_type)
        } else {
            entity_type
        };
        Self { entity_type, min_count, max_count }
    }
}

/// `MobSpawnSettings.Builder`.
#[derive(Default)]
pub struct MobSpawnSettingsBuilder {
    spawners: HashMap<MobCategory, WeightedListBuilder<SpawnerData>>,
    mob_spawn_costs: HashMap<Arc<EntityType>, MobSpawnCost>,
    creature_generation_probability: f32,
}

impl MobSpawnSettingsBuilder {
    /// `new Builder()`.
    pub fn new() -> Self {
        let mut b = Self::default();
        b.creature_generation_probability = 0.1;
        b
    }

    /// `addSpawn(category, weight, spawnerData)`.
    pub fn add_spawn(mut self, category: MobCategory, weight: i32, spawner_data: SpawnerData) -> Self {
        self.spawners.entry(category).or_default().add_with_weight(spawner_data, weight);
        self
    }

    /// `addMobCharge(entityType, charge, energyBudget)`.
    pub fn add_mob_charge(mut self, entity_type: Arc<EntityType>, charge: f64, energy_budget: f64) -> Self {
        self.mob_spawn_costs.insert(entity_type, MobSpawnCost { energy_budget, charge });
        self
    }

    /// `creatureGenerationProbability`.
    pub fn creature_generation_probability(mut self, probability: f32) -> Self {
        self.creature_generation_probability = probability;
        self
    }

    /// `build`.
    pub fn build(self) -> MobSpawnSettings {
        MobSpawnSettings {
            creature_generation_probability: self.creature_generation_probability,
            spawners: self.spawners.into_iter().map(|(k, v)| (k, v.build())).collect(),
            mob_spawn_costs: self.mob_spawn_costs,
        }
    }
}


