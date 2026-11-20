//! `ca.spottedleaf.moonrise.common.util.WorldUtil` — утилиты границ мира
//! для Moonrise-патчей (Starlight и др.).

use crate::core::section_pos::SectionPos;
use crate::world::level::level_reader::LevelReader;

/// `WorldUtil` — min/max секции и блоков (все значения inclusive).
pub struct WorldUtil;

impl WorldUtil {
    /// `getMaxSection`.
    pub fn get_max_section(level: &dyn LevelReader) -> i32 {
        SectionPos::block_to_section_coord(level.min_build_height() + level.height() - 1)
    }

    /// `getMinSection`.
    pub fn get_min_section(level: &dyn LevelReader) -> i32 {
        SectionPos::block_to_section_coord(level.min_build_height())
    }

    /// `getMaxLightSection` = maxSection + 1.
    pub fn get_max_light_section(level: &dyn LevelReader) -> i32 {
        Self::get_max_section(level) + 1
    }

    /// `getMinLightSection` = minSection - 1.
    pub fn get_min_light_section(level: &dyn LevelReader) -> i32 {
        Self::get_min_section(level) - 1
    }

    /// `getTotalSections`.
    pub fn get_total_sections(level: &dyn LevelReader) -> i32 {
        Self::get_max_section(level) - Self::get_min_section(level) + 1
    }

    /// `getTotalLightSections`.
    pub fn get_total_light_sections(level: &dyn LevelReader) -> i32 {
        Self::get_max_light_section(level) - Self::get_min_light_section(level) + 1
    }

    /// `getMinBlockY` = minSection << 4.
    pub fn get_min_block_y(level: &dyn LevelReader) -> i32 {
        Self::get_min_section(level) << 4
    }

    /// `getMaxBlockY` = (maxSection << 4) | 15.
    pub fn get_max_block_y(level: &dyn LevelReader) -> i32 {
        (Self::get_max_section(level) << 4) | 15
    }
}
