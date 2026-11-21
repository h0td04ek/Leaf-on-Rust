//! `net.minecraft.world.DifficultyInstance` — эффективная
//! сложность в позиции (время + inhabitedTime + фаза луны).

use crate::util::mth;
use crate::world::difficulty::Difficulty;

/// `DifficultyInstance`.
#[derive(Clone, Copy, Debug)]
pub struct DifficultyInstance {
    base: Difficulty,
    effective_difficulty: f32,
}

const DIFFICULTY_TIME_GLOBAL_OFFSET: f32 = -72000.0;
const MAX_DIFFICULTY_TIME_GLOBAL: f32 = 1440000.0;
const MAX_DIFFICULTY_TIME_LOCAL: f32 = 3600000.0;

impl DifficultyInstance {
    /// `new DifficultyInstance(base, levelTime, chunkInhabitedTime,
    /// moonPhaseFactor)`.
    pub fn new(
        base: Difficulty,
        level_time: i64,
        chunk_inhabited_time: i64,
        moon_phase_factor: f32,
    ) -> Self {
        let effective =
            Self::calculate_difficulty(base, level_time, chunk_inhabited_time, moon_phase_factor);
        Self { base, effective_difficulty: effective }
    }

    /// `getDifficulty()`.
    pub fn get_difficulty(&self) -> Difficulty {
        self.base
    }

    /// `getEffectiveDifficulty()`.
    pub fn get_effective_difficulty(&self) -> f32 {
        self.effective_difficulty
    }

    /// `isHard()`.
    pub fn is_hard(&self) -> bool {
        self.effective_difficulty >= Difficulty::Hard as i32 as f32
    }

    /// `isHarderThan(difficulty)`.
    pub fn is_harder_than(&self, difficulty: f32) -> bool {
        self.effective_difficulty > difficulty
    }

    /// `getSpecialMultiplier()`.
    pub fn get_special_multiplier(&self) -> f32 {
        if self.effective_difficulty < 2.0 {
            0.0
        } else if self.effective_difficulty > 4.0 {
            1.0
        } else {
            (self.effective_difficulty - 2.0) / 2.0
        }
    }

    /// `calculateDifficulty(...)`.
    fn calculate_difficulty(
        difficulty: Difficulty,
        level_time: i64,
        chunk_inhabited_time: i64,
        moon_phase_factor: f32,
    ) -> f32 {
        if difficulty == Difficulty::Peaceful {
            return 0.0;
        }
        let flag = difficulty == Difficulty::Hard;
        let mut f = 0.75f32;
        let f1 = mth::clamp_f32(
            (level_time as f32 + DIFFICULTY_TIME_GLOBAL_OFFSET) / MAX_DIFFICULTY_TIME_GLOBAL,
            0.0,
            1.0,
        ) * 0.25;
        f += f1;
        let mut f2 = 0.0f32;
        f2 += mth::clamp_f32(
            chunk_inhabited_time as f32 / MAX_DIFFICULTY_TIME_LOCAL,
            0.0,
            1.0,
        ) * if flag { 1.0 } else { 0.75 };
        f2 += mth::clamp_f32(moon_phase_factor * 0.25, 0.0, f1);
        if difficulty == Difficulty::Easy {
            f2 *= 0.5;
        }
        f += f2;
        difficulty.get_id() as f32 * f
    }
}
