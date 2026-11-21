//! Перенос `net.minecraft.world.food.FoodData`.

use crate::nbt::CompoundTag;
use crate::world::difficulty::Difficulty;

use super::FoodProperties;

#[derive(Clone, Debug)]
pub struct FoodData {
    pub food_level: i32,
    pub saturation_level: f32,
    pub exhaustion_level: f32,
    tick_timer: i32,
    pub saturated_regen_rate: i32,
    pub unsaturated_regen_rate: i32,
    pub starvation_rate: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FoodTick {
    Regenerate { health: f32, exhaustion: f32 },
    Starve,
}

impl Default for FoodData {
    fn default() -> Self {
        Self {
            food_level: 20,
            saturation_level: 5.0,
            exhaustion_level: 0.0,
            tick_timer: 0,
            saturated_regen_rate: 10,
            unsaturated_regen_rate: 80,
            starvation_rate: 80,
        }
    }
}

impl FoodData {
    pub fn eat(&mut self, food: FoodProperties) {
        self.add_food(food.nutrition, food.saturation);
    }

    pub fn eat_with_modifier(&mut self, nutrition: i32, saturation_modifier: f32) {
        self.add_food(nutrition, nutrition as f32 * saturation_modifier * 2.0);
    }

    fn add_food(&mut self, nutrition: i32, saturation: f32) {
        self.food_level = self.food_level.wrapping_add(nutrition).clamp(0, 20);
        self.saturation_level =
            (self.saturation_level + saturation).clamp(0.0, self.food_level as f32);
    }

    pub fn has_enough_food(&self) -> bool {
        self.food_level > 6
    }

    pub fn needs_food(&self) -> bool {
        self.food_level < 20
    }

    pub fn add_exhaustion(&mut self, amount: f32) {
        let exhaustion = self.exhaustion_level + amount;
        self.exhaustion_level = if exhaustion > 40.0 { 40.0 } else { exhaustion };
    }

    /// Обновляет голод и возвращает действие, которое игрок применит к здоровью.
    pub fn tick(
        &mut self,
        difficulty: Difficulty,
        health: f32,
        max_health: f32,
        natural_health_regeneration: bool,
        regen_exhaustion: f32,
    ) -> Option<FoodTick> {
        if self.exhaustion_level > 4.0 {
            self.exhaustion_level -= 4.0;
            if self.saturation_level > 0.0 {
                self.saturation_level = (self.saturation_level - 1.0).max(0.0);
            } else if difficulty != Difficulty::Peaceful {
                self.food_level = self.food_level.saturating_sub(1).max(0);
            }
        }

        let can_regenerate = natural_health_regeneration && health > 0.0 && health < max_health;
        let (rate, effect) =
            if can_regenerate && self.food_level >= 20 && self.saturation_level > 0.0 {
                let exhaustion = self.saturation_level.min(6.0);
                (
                    self.saturated_regen_rate,
                    Some(FoodTick::Regenerate {
                        health: exhaustion / 6.0,
                        exhaustion,
                    }),
                )
            } else if can_regenerate && self.food_level >= 18 {
                (
                    self.unsaturated_regen_rate,
                    Some(FoodTick::Regenerate {
                        health: 1.0,
                        exhaustion: regen_exhaustion,
                    }),
                )
            } else if self.food_level <= 0 {
                let can_starve = health > 10.0
                    || difficulty == Difficulty::Hard
                    || (health > 1.0 && difficulty == Difficulty::Normal);
                (self.starvation_rate, can_starve.then_some(FoodTick::Starve))
            } else {
                self.tick_timer = 0;
                return None;
            };

        self.tick_timer = self.tick_timer.wrapping_add(1);
        if self.tick_timer < rate {
            return None;
        }
        self.tick_timer = 0;
        effect
    }

    pub fn read_additional_save_data(&mut self, tag: &CompoundTag) {
        self.food_level = tag.get_int_or("foodLevel", 20);
        self.tick_timer = tag.get_int_or("foodTickTimer", 0);
        self.saturation_level = tag.get_float("foodSaturationLevel").unwrap_or(5.0);
        self.exhaustion_level = tag.get_float("foodExhaustionLevel").unwrap_or(0.0);
    }

    pub fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put_int("foodLevel", self.food_level);
        tag.put_int("foodTickTimer", self.tick_timer);
        tag.put_float("foodSaturationLevel", self.saturation_level);
        tag.put_float("foodExhaustionLevel", self.exhaustion_level);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saturating_regeneration_keeps_progress_after_save() {
        let mut food = FoodData::default();
        for _ in 0..9 {
            assert_eq!(food.tick(Difficulty::Normal, 18.0, 20.0, true, 6.0), None);
        }

        let mut tag = CompoundTag::new();
        food.add_additional_save_data(&mut tag);
        let mut restored = FoodData::default();
        restored.read_additional_save_data(&tag);
        assert_eq!(
            restored.tick(Difficulty::Normal, 18.0, 20.0, true, 6.0),
            Some(FoodTick::Regenerate {
                health: 5.0 / 6.0,
                exhaustion: 5.0
            })
        );
    }

    #[test]
    fn exhaustion_and_starvation_depend_on_difficulty() {
        let mut food = FoodData {
            food_level: 1,
            saturation_level: 0.0,
            exhaustion_level: 4.5,
            starvation_rate: 1,
            ..FoodData::default()
        };
        assert_eq!(food.tick(Difficulty::Peaceful, 10.0, 20.0, true, 6.0), None);
        assert_eq!(food.food_level, 1);
        assert_eq!(food.exhaustion_level, 0.5);

        food.add_exhaustion(4.0);
        assert_eq!(food.tick(Difficulty::Easy, 10.0, 20.0, true, 6.0), None);
        assert_eq!(food.food_level, 0);
        assert_eq!(food.tick(Difficulty::Normal, 1.0, 20.0, true, 6.0), None);
        assert_eq!(
            food.tick(Difficulty::Hard, 1.0, 20.0, true, 6.0),
            Some(FoodTick::Starve)
        );
    }
}
