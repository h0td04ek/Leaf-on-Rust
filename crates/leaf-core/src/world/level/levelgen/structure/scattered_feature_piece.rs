//! Перенос `net.minecraft.world.level.levelgen.structure.ScatteredFeaturePiece`
//! — базовый класс рассеянных структурных кусков (width/height/
//! depth + heightPosition + updateAverageGroundHeight /
//! updateHeightPositionToLowestGroundHeight).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::structure_piece::{make_bounding_box, StructurePiece};

/// `ScatteredFeaturePiece` — общий предок кусков одиноких
/// структур (пирамида, маяк и т.п.). Хранит размеры и
/// `heightPosition` (-1 = не вычислена).
pub struct ScatteredFeaturePieceData {
    /// `width`.
    pub width: i32,
    /// `height`.
    pub height: i32,
    /// `depth`.
    pub depth: i32,
    /// `heightPosition` — -1 пока не вычислена.
    pub height_position: i32,
}

impl ScatteredFeaturePieceData {
    /// `new(x, y, z, width, height, depth, orientation)` —
    /// bounding box через `makeBoundingBox`.
    pub fn new(
        x: i32,
        y: i32,
        z: i32,
        width: i32,
        height: i32,
        depth: i32,
        orientation: Direction,
    ) -> (Self, BoundingBox) {
        (
            Self {
                width,
                height,
                depth,
                height_position: -1,
            },
            make_bounding_box(x, y, z, orientation, width, height, depth),
        )
    }

    /// `load(tag)` — чтение Width/Height/Depth/HPos.
    pub fn load(tag: &CompoundTag) -> Self {
        Self {
            width: tag.get_int("Width").unwrap_or(0),
            height: tag.get_int("Height").unwrap_or(0),
            depth: tag.get_int("Depth").unwrap_or(0),
            height_position: tag.get_int("HPos").unwrap_or(0),
        }
    }

    /// `addAdditionalSaveData(tag)`.
    pub fn save(&self, tag: &mut CompoundTag) {
        tag.put("Width", Tag::Int(self.width));
        tag.put("Height", Tag::Int(self.height));
        tag.put("Depth", Tag::Int(self.depth));
        tag.put("HPos", Tag::Int(self.height_position));
    }

    /// `updateAverageGroundHeight(level, bounds, height)` —
    /// средняя высота MOTION_BLOCKING_NO_LEAVES по боксу;
    /// сдвигает boundingBox по Y.
    pub fn update_average_ground_height(
        &mut self,
        level: &dyn LevelReader,
        bounds: &BoundingBox,
        height: i32,
        bb: &mut BoundingBox,
    ) -> bool {
        if self.height_position >= 0 {
            return true;
        }
        let mut sum = 0i64;
        let mut count = 0i64;
        for z in bb.min_z..=bb.max_z {
            for x in bb.min_x..=bb.max_x {
                let pos = BlockPos::new(x, 64, z);
                if bounds.is_inside_pos(pos) {
                    sum += level
                        .get_height(HeightmapType::MotionBlockingNoLeaves, x, z) as i64;
                    count += 1;
                }
            }
        }
        if count == 0 {
            return false;
        }
        self.height_position = (sum / count) as i32;
        bb.move_(0, self.height_position - bb.min_y + height, 0);
        true
    }

    /// `updateHeightPositionToLowestGroundHeight` — минимальная
    /// высота по боксу.
    pub fn update_height_position_to_lowest_ground_height(
        &mut self,
        level: &dyn LevelReader,
        height: i32,
        bb: &mut BoundingBox,
    ) -> bool {
        if self.height_position >= 0 {
            return true;
        }
        let mut min_y = level.min_build_height() + level.height();
        let mut found = false;
        for z in bb.min_z..=bb.max_z {
            for x in bb.min_x..=bb.max_x {
                let pos = BlockPos::new(x, 0, z);
                min_y = min_y.min(level.get_height(HeightmapType::MotionBlockingNoLeaves, pos.x, pos.z));
                found = true;
            }
        }
        if !found {
            return false;
        }
        self.height_position = min_y;
        bb.move_(0, self.height_position - bb.min_y + height, 0);
        true
    }
}
