//! `net.minecraft.world.level.storage.LevelData` и `WritableLevelData` —
//! интерфейсы сохранённых данных уровня (level.dat).

use crate::crash_report::{CrashReportCategorySink, DetailSupplier};
use crate::world::difficulty::Difficulty;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::RespawnData;

/// `LevelData` — доступ к метаданным мира.
pub trait LevelData {
    /// `getRespawnData()`.
    fn get_respawn_data(&self) -> RespawnData;

    /// `getGameTime()`.
    fn get_game_time(&self) -> i64;

    /// `getDayTime()`.
    fn get_day_time(&self) -> i64;

    /// `isThundering()`.
    fn is_thundering(&self) -> bool;

    /// `isRaining()`.
    fn is_raining(&self) -> bool;

    /// `setRaining(raining)`.
    fn set_raining(&mut self, raining: bool);

    /// `isHardcore()`.
    fn is_hardcore(&self) -> bool;

    /// `getDifficulty()`.
    fn get_difficulty(&self) -> Difficulty;

    /// `isDifficultyLocked()`.
    fn is_difficulty_locked(&self) -> bool;

    /// `fillCrashReportCategory(category, level)`.
    fn fill_crash_report_category(
        &self,
        category: &mut dyn CrashReportCategorySink,
        level: &dyn LevelReader,
    ) {
        let respawn = self.get_respawn_data();
        let respawn_pos = respawn.global_pos.pos;
        category.set_detail(
            "Level spawn location",
            Box::new(move || {
                // CrashReportCategory.formatLocation(level, pos)
                format!(
                    "{}, {}, {}",
                    respawn_pos.get_x(),
                    respawn_pos.get_y(),
                    respawn_pos.get_z()
                )
            }) as DetailSupplier,
        );
        let game_time = self.get_game_time();
        let day_time = self.get_day_time();
        category.set_detail(
            "Level time",
            Box::new(move || {
                format!("{game_time} game time, {day_time} day time")
            }),
        );
    }
}

/// `WritableLevelData` — записываемые данные уровня.
pub trait WritableLevelData: LevelData {
    /// `setSpawn(respawnData)`.
    fn set_spawn(&mut self, respawn_data: RespawnData);
}
