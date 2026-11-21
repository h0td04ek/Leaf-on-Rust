//! Базовое состояние игрока поверх LivingEntity.

use std::sync::Arc;

use crate::authlib::game_profile::GameProfile;
use crate::core::holder::Holder;
use crate::entity::ai::attributes::attributes;
use crate::entity::player::abilities::{Abilities, Packed};
use crate::nbt::{CompoundTag, Tag};
use crate::network::chat::component::Component;
use crate::network::protocol::game::clientbound_set_experience_packet::ClientboundSetExperiencePacket;
use crate::resources::Identifier;
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
    pub experience: Experience,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Experience {
    pub progress: f32,
    pub level: i32,
    pub total: i32,
    pub score: i32,
    pub enchantment_seed: i32,
    last_level_up_tick: i32,
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
            experience: Experience::default(),
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

    pub fn xp_needed_for_next_level(&self) -> i32 {
        let level = self.experience.level.max(0);
        match level {
            0..15 => 7 + level * 2,
            15..30 => 37 + (level - 15) * 5,
            _ => 112i32.saturating_add((level - 30).saturating_mul(9)),
        }
    }

    pub fn give_experience_points(&mut self, points: i32) {
        self.experience.score = self.experience.score.wrapping_add(points);
        self.experience.progress += points as f32 / self.xp_needed_for_next_level() as f32;
        self.experience.total = self.experience.total.saturating_add(points).max(0);
        while self.experience.progress < 0.0 {
            let remainder = self.experience.progress * self.xp_needed_for_next_level() as f32;
            let can_lower = self.experience.level > 0;
            self.give_experience_levels(-1);
            if can_lower {
                self.experience.progress = 1.0 + remainder / self.xp_needed_for_next_level() as f32;
            }
        }
        while self.experience.progress >= 1.0 {
            if self.experience.level == i32::MAX {
                self.experience.progress = f32::from_bits(1.0f32.to_bits() - 1);
                break;
            }
            let remainder =
                (self.experience.progress - 1.0) * self.xp_needed_for_next_level() as f32;
            self.give_experience_levels(1);
            self.experience.progress = remainder / self.xp_needed_for_next_level() as f32;
        }
    }

    pub fn give_experience_levels(&mut self, levels: i32) {
        self.experience.level = self.experience.level.saturating_add(levels);
        self.reset_xp_if_below_zero();
        let tick = self.living.entity.tick_count();
        if levels > 0
            && self.experience.level % 5 == 0
            && self.experience.last_level_up_tick < tick.saturating_sub(100)
        {
            if let Some(level) = self.living.entity.level.as_ref() {
                let volume = (self.experience.level as f32 / 30.0).min(1.0) * 0.75;
                level.play_sound(
                    self.living.entity.position(),
                    &Identifier::minecraft("entity.player.levelup"),
                    volume,
                    1.0,
                );
            }
            self.experience.last_level_up_tick = tick;
        }
    }

    pub fn on_enchantment_performed(&mut self, level_cost: i32) {
        self.experience.level = self.experience.level.saturating_sub(level_cost);
        self.reset_xp_if_below_zero();
        self.experience.enchantment_seed = self.living.entity.random_mut().next_int();
    }

    fn reset_xp_if_below_zero(&mut self) {
        if self.experience.level < 0 {
            self.experience.level = 0;
            self.experience.progress = 0.0;
            self.experience.total = 0;
        }
    }

    pub fn experience_packet(&self) -> ClientboundSetExperiencePacket {
        ClientboundSetExperiencePacket::new(
            self.experience.progress,
            self.experience.total,
            self.experience.level,
        )
    }

    pub fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        self.living.add_additional_save_data(tag);
        crate::nbt::utils::add_current_data_version(tag);
        tag.put_float("XpP", self.experience.progress);
        tag.put_int("XpLevel", self.experience.level);
        tag.put_int("XpTotal", self.experience.total);
        tag.put_int("XpSeed", self.experience.enchantment_seed);
        tag.put_int("Score", self.experience.score);
        self.food_data.add_additional_save_data(tag);
        tag.put("abilities", Tag::Compound(self.abilities.pack().to_nbt()));
    }

    pub fn read_additional_save_data(&mut self, tag: &CompoundTag) {
        self.living.read_additional_save_data(tag);
        self.living.entity.uuid = self.profile.id;
        self.experience.progress = tag
            .get_float("XpP")
            .filter(|p| p.is_finite())
            .unwrap_or_default();
        self.experience.level = tag.get_int_or("XpLevel", 0).max(0);
        self.experience.total = tag.get_int_or("XpTotal", 0).max(0);
        self.experience.enchantment_seed = tag.get_int_or("XpSeed", 0);
        self.experience.score = tag.get_int_or("Score", 0);
        if self.experience.enchantment_seed == 0 {
            self.experience.enchantment_seed = self.living.entity.random_mut().next_int();
        }
        self.food_data.read_additional_save_data(tag);
        if let Some(abilities) = tag.get_compound("abilities") {
            self.abilities.apply(&Packed::from_nbt(abilities));
        }
        if let Some(speed) = self
            .living
            .attributes
            .get_instance(&attributes::movement_speed())
        {
            speed.set_base_value(f64::from(self.abilities.walking_speed));
        }
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
    fn experience_crosses_level_boundaries_and_loads_vanilla_fields() {
        let profile = GameProfile::new(Uuid::new(1, 2), "Leaf");
        let mut player = Player::new(profile.clone(), None);
        player.give_experience_points(7);
        assert_eq!(
            player.experience_packet(),
            ClientboundSetExperiencePacket::new(0.0, 7, 1)
        );
        player.give_experience_points(-1);
        assert_eq!(player.experience.level, 0);
        assert!((player.experience.progress - 6.0 / 7.0).abs() < 1.0e-6);
        player.give_experience_points(-20);
        assert_eq!(player.experience.progress, 0.0);
        assert_eq!(player.experience.total, 0);

        player.give_experience_levels(30);
        assert_eq!(player.xp_needed_for_next_level(), 112);
        player.on_enchantment_performed(1);
        assert_eq!(player.experience.level, 29);
        player.experience.enchantment_seed = 41;
        player.abilities.set_walking_speed(0.24);
        player.food_data.food_level = 15;

        let mut tag = CompoundTag::new();
        player.add_additional_save_data(&mut tag);
        let mut restored = Player::new(profile, None);
        restored.read_additional_save_data(&tag);
        assert_eq!(restored.experience, player.experience);
        assert_eq!(restored.abilities.walking_speed, 0.24);
        assert_eq!(restored.food_data.food_level, 15);
    }

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
