//! `net.minecraft.world.level.LevelHeightAccessor` — доступ к высоте мира.

use crate::core::section_pos::SectionPos;

/// `LevelHeightAccessor`.
pub trait LevelHeightAccessor {
    /// `getMinY`.
    fn get_min_y(&self) -> i32;
    /// `getHeight`.
    fn get_height(&self) -> i32;
    /// `getMaxY` = minY + height - 1.
    fn get_max_y(&self) -> i32 {
        self.get_min_y() + self.get_height() - 1
    }
    /// `getSectionsCount`.
    fn get_sections_count(&self) -> i32 {
        self.get_max_section_y() - self.get_min_section_y() + 1
    }
    /// `getMinSectionY`.
    fn get_min_section_y(&self) -> i32 {
        SectionPos::block_to_section_coord(self.get_min_y())
    }
    /// `getMaxSectionY`.
    fn get_max_section_y(&self) -> i32 {
        SectionPos::block_to_section_coord(self.get_max_y())
    }
    /// `isInsideBuildHeight(y)`.
    fn is_inside_build_height(&self, y: i32) -> bool {
        y >= self.get_min_y() && y <= self.get_max_y()
    }
    /// `isOutsideBuildHeight(y)`.
    fn is_outside_build_height(&self, y: i32) -> bool {
        y < self.get_min_y() || y > self.get_max_y()
    }
    /// `getSectionIndex(sectionIndex)` — 0-based от minSection.
    fn get_section_index(&self, section_index: i32) -> i32 {
        section_index - self.get_min_section_y()
    }
    /// `getSectionIndexFromSectionY`.
    fn get_section_index_from_section_y(&self, section_y: i32) -> i32 {
        section_y + self.get_min_section_y()
    }
}

/// `LevelHeightAccessor.create(minBuildHeight, height)` — анонимная реализация.
pub struct SimpleLevelHeightAccessor {
    min_y: i32,
    height: i32,
}

impl SimpleLevelHeightAccessor {
    /// `create`.
    pub fn create(min_build_height: i32, height: i32) -> Self {
        Self { min_y: min_build_height, height }
    }
}

impl LevelHeightAccessor for SimpleLevelHeightAccessor {
    fn get_min_y(&self) -> i32 {
        self.min_y
    }
    fn get_height(&self) -> i32 {
        self.height
    }
}
