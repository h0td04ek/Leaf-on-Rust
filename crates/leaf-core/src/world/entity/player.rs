//! Базовое состояние игрока поверх LivingEntity.

use std::sync::Arc;

use crate::authlib::game_profile::GameProfile;
use crate::core::holder::Holder;
use crate::entity::player::abilities::Abilities;
use crate::network::chat::component::Component;
use crate::world::damagesource::combat_tracker::CombatTracker;
use crate::world::food::food_data::{FoodData, FoodTick};
use crate::world::food::FoodProperties;
use crate::world::level::game_type::{GameType, DEFAULT_MODE};
use crate::world::level::level_reader::LevelReader;
use crate::world::level::Level;

use super::entity_type::player_entity_type;
use super::LivingEntity;

pub struct Player {
    pub living: LivingEntity,
    profile: GameProfile,
    abilities: Abilities,
    pub food_data: FoodData,
}

impl Player {
    pub fn new(profile: GameProfile, level: Option<Arc<dyn LevelReader>>) -> Self {
        let mut living = LivingEntity::new(Holder::Direct(player_entity_type()), level);
        living.entity.uuid = profile.id;
        living.entity.name = Some(Component::literal(profile.name.clone()));
        living.entity.player_game_mode = Some(DEFAULT_MODE);
        living.combat_tracker = CombatTracker::new(living.entity.clone());

        Self {
            living,
            profile,
            abilities: Abilities::default(),
            food_data: FoodData::default(),
        }
    }

    pub fn profile(&self) -> &GameProfile {
        &self.profile
    }

    pub fn abilities(&self) -> &Abilities {
        &self.abilities
    }

    pub fn game_mode(&self) -> GameType {
        self.living.entity.player_game_mode.unwrap_or(DEFAULT_MODE)
    }

    pub fn set_game_mode(&mut self, mode: GameType) {
        mode.update_player_abilities(&mut self.abilities);
        self.living.entity.player_game_mode = Some(mode);
        self.living
            .entity
            .set_no_physics(mode == GameType::Spectator);
    }

    pub fn can_eat(&self, can_always_eat: bool) -> bool {
        self.abilities.invulnerable || can_always_eat || self.food_data.needs_food()
    }

    pub fn eat(&mut self, food: FoodProperties) -> bool {
        if !self.can_eat(food.can_always_eat) {
            return false;
        }
        self.food_data.eat(food);
        true
    }

    pub fn cause_food_exhaustion(&mut self, amount: f32) {
        let client_side = self
            .living
            .entity
            .level
            .as_ref()
            .is_some_and(|level| level.is_client_side());
        if !self.abilities.invulnerable && !client_side {
            self.food_data.add_exhaustion(amount);
        }
    }

    /// Применяет голод, естественную регенерацию и урон голодания на серверной стороне.
    pub fn tick_food(
        &mut self,
        level: &dyn Level,
        natural_health_regeneration: bool,
        regen_exhaustion: f32,
        starvation_damage: f32,
    ) {
        if level.is_client_side() || !self.living.is_alive() || self.living.entity.is_spectator() {
            return;
        }
        let health = self.living.get_health();
        let max_health = self.living.get_max_health();
        match self.food_data.tick(
            level.get_difficulty(),
            health,
            max_health,
            natural_health_regeneration,
            regen_exhaustion,
        ) {
            Some(FoodTick::Regenerate { health, exhaustion }) => {
                self.living.heal(health);
                self.cause_food_exhaustion(exhaustion);
            }
            Some(FoodTick::Starve) if !self.abilities.invulnerable => {
                self.living
                    .hurt_server(level.damage_sources().starve(), starvation_damage);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::uuid::Uuid;

    #[test]
    fn game_mode_updates_base_entity_and_abilities() {
        let profile_id = Uuid::random();
        let mut player = Player::new(GameProfile::new(profile_id, "Leaf"), None);
        assert!(player.living.entity.is_player());
        assert_eq!(player.living.entity.get_uuid(), profile_id);
        assert_eq!(player.living.get_health(), 20.0);
        assert!(!player.can_eat(false));

        player.set_game_mode(GameType::Spectator);
        assert!(player.living.entity.is_spectator());
        assert!(player.living.entity.is_no_physics());
        assert!(player.abilities().flying);
        assert!(!player.abilities().may_build);

        player.set_game_mode(GameType::Creative);
        assert!(player.living.entity.is_creative());
        assert!(player.abilities().flying);
        assert!(player.abilities().instabuild);
        player.cause_food_exhaustion(8.0);
        assert_eq!(player.food_data.exhaustion_level, 0.0);

        player.set_game_mode(GameType::Adventure);
        assert!(!player.living.entity.is_spectator());
        assert!(!player.living.entity.is_no_physics());
        assert!(!player.abilities().flying);
        player.cause_food_exhaustion(8.0);
        assert_eq!(player.food_data.exhaustion_level, 8.0);
    }
}
