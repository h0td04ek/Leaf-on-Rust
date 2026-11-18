//! Перенос `net.minecraft.world.level.PathNavigationRegion` —
//! ограниченный регион уровня для pathfinding (вне региона — air).

use crate::core::block_pos::BlockPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::level_reader::LevelReader;
use std::sync::Arc;

/// `PathNavigationRegion` — `CollisionGetter`-view уровня,
/// ограниченный прямоугольником чанков (снаружи — `EmptyLevelChunk` = air).
///
/// В Java регион строится по секциям между `centerPos` и
/// `offsetPos`; здесь храним границы в координатах секций
/// (`x >> 4`).
pub struct PathNavigationRegion<'a> {
    /// `level`.
    pub level: &'a dyn LevelReader,
    /// `centerX`/`centerZ` — центральная секция.
    pub center_x: i32,
    pub center_z: i32,
    /// Границы по секциям [min,max] включительно.
    pub min_section_x: i32,
    pub max_section_x: i32,
    pub min_section_z: i32,
    pub max_section_z: i32,
}

impl<'a> PathNavigationRegion<'a> {
    /// `new PathNavigationRegion(level, centerPos, offsetPos)`.
    pub fn new(level: &'a dyn LevelReader, center_pos: BlockPos, offset_pos: BlockPos) -> Self {
        let center_x = center_pos.x >> 4;
        let center_z = center_pos.z >> 4;
        let section_pos_x = offset_pos.x >> 4;
        let section_pos_z = offset_pos.z >> 4;
        Self {
            level,
            center_x,
            center_z,
            min_section_x: center_x.min(section_pos_x),
            max_section_x: center_x.max(section_pos_x),
            min_section_z: center_z.min(section_pos_z),
            max_section_z: center_z.max(section_pos_z),
        }
    }

    /// `getChunk(x, z)`-проверка: секция внутри региона.
    fn in_region(&self, pos: BlockPos) -> bool {
        let sx = pos.x >> 4;
        let sz = pos.z >> 4;
        sx >= self.min_section_x
            && sx <= self.max_section_x
            && sz >= self.min_section_z
            && sz <= self.max_section_z
    }
}

impl LevelReader for PathNavigationRegion<'_> {
    fn get_block_state(&self, pos: BlockPos) -> Option<BlockState> {
        if !self.level.is_inside_build_height(pos.y) || !self.in_region(pos) {
            return Some(BlockState::air());
        }
        self.level.get_block_state(pos)
    }

    fn get_fluid_state(&self, pos: BlockPos) -> Option<FluidState> {
        if !self.level.is_inside_build_height(pos.y) || !self.in_region(pos) {
            return Some(FluidState::empty());
        }
        self.level.get_fluid_state(pos)
    }

    fn get_block_entity(
        &self,
        pos: BlockPos,
    ) -> Option<Arc<dyn crate::world::level::block::entity::block_entity::BlockEntityAccess>> {
        if !self.in_region(pos) {
            return None;
        }
        self.level.get_block_entity(pos)
    }

    fn has_chunk_at(&self, pos: BlockPos) -> bool {
        self.in_region(pos) && self.level.has_chunk_at(pos)
    }

    fn min_build_height(&self) -> i32 {
        self.level.min_build_height()
    }
    fn height(&self) -> i32 {
        self.level.height()
    }
    fn sea_level(&self) -> i32 {
        self.level.sea_level()
    }
    fn is_inside_build_height(&self, y: i32) -> bool {
        self.level.is_inside_build_height(y)
    }
    fn game_time(&self) -> i64 {
        self.level.game_time()
    }
    fn path_type_cache(&self) -> Option<&crate::world::level::pathfinder::path_type_cache::PathTypeCache> {
        // PathNavigationRegion не имеет своего кэша — PathFinder
        // получает кэш уровня отдельно.
        None
    }
    fn no_collision(&self, aabb: &crate::world::phys::aabb::Aabb) -> bool {
        self.level.no_collision(aabb)
    }
    fn can_see_sky(&self, pos: BlockPos) -> bool {
        self.level.can_see_sky(pos)
    }
    fn scoreboard(&self) -> Option<&crate::world::scores::scoreboard::Scoreboard> {
        self.level.scoreboard()
    }
    fn is_raining(&self) -> bool {
        self.level.is_raining()
    }
    fn is_thundering(&self) -> bool {
        self.level.is_thundering()
    }
}
