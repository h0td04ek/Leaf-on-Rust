//! Перенос `net.minecraft.world.level.lighting.ChunkSkyLightSources` —
//! высоты нижних источников небесного света по колонкам чанка (256 значений).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::section_pos::SectionPos;
use crate::util::bit_storage::SimpleBitStorage;
use crate::util::mth;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::level_height_accessor::LevelHeightAccessor;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::lighting::light_engine::get_occlusion_shape;

pub const NEGATIVE_INFINITY: i32 = i32::MIN;

const SIZE: usize = 16;

/// `ChunkSkyLightSources`.
pub struct ChunkSkyLightSources {
    min_y: i32,
    heightmap: SimpleBitStorage,
}

impl ChunkSkyLightSources {
    /// `new(level)`.
    pub fn new(level: &dyn LevelHeightAccessor) -> Self {
        let min_y = level.get_min_y() - 1;
        let max_y = level.get_max_y() + 1;
        let bits = mth::ceillog2(max_y - min_y + 1);
        Self {
            min_y,
            heightmap: SimpleBitStorage::new(bits, 256),
        }
    }

    /// `fillFrom(chunk)`.
    pub fn fill_from(&mut self, chunk: &ChunkAccess) {
        let highest_filled_section_index = chunk.get_highest_filled_section_index();
        if highest_filled_section_index == -1 {
            self.fill(self.min_y);
        } else {
            for x in 0..16 {
                for z in 0..16 {
                    let max = self
                        .find_lowest_source_y(chunk, highest_filled_section_index, x, z)
                        .max(self.min_y);
                    self.set(index(x, z), max);
                }
            }
        }
    }

    /// `findLowestSourceY(chunk, sectionIndex, x, z)`.
    fn find_lowest_source_y(
        &self,
        chunk: &ChunkAccess,
        section_index: i32,
        x: i32,
        z: i32,
    ) -> i32 {
        let block_pos_coord =
            SectionPos::section_to_block_coord(chunk.get_section_y_from_section_index(section_index) + 1);
        let mut pos = BlockPos::new(x, block_pos_coord, z);
        let mut pos_below = pos.relative(Direction::DOWN);
        let mut block_state = BlockState::air();

        for i in (0..=section_index).rev() {
            let section = chunk.get_section(i);
            match section {
                Some(section) if !section.has_only_air() => {
                    for section_y in (0..16).rev() {
                        let state1 = section.get_block_state(x, section_y, z);
                        if is_edge_occluded(&block_state, &state1) {
                            return pos.get_y();
                        }
                        block_state = state1;
                        pos = pos_below;
                        pos_below = pos_below.relative(Direction::DOWN);
                    }
                }
                _ => {
                    block_state = BlockState::air();
                    let section_y = chunk.get_section_y_from_section_index(i);
                    pos = BlockPos::new(x, SectionPos::section_to_block_coord(section_y), z);
                    pos_below = pos.relative(Direction::DOWN);
                }
            }
        }

        self.min_y
    }

    /// `update(level, x, y, z)` — обновляет колонку после изменения блока.
    pub fn update(&mut self, level: &dyn LevelReader, x: i32, y: i32, z: i32) -> bool {
        let i = y + 1;
        let i1 = index(x, z);
        let i2 = self.get(i1);
        if i < i2 {
            return false;
        }
        let pos = BlockPos::new(x, y + 1, z);
        let state = level.get_block_state(pos).unwrap_or_else(BlockState::air);
        let pos1 = BlockPos::new(x, y, z);
        let state1 = level.get_block_state(pos1).unwrap_or_else(BlockState::air);
        if self.update_edge(level, i1, i2, &pos, &state, &pos1, &state1) {
            return true;
        }
        let pos2 = BlockPos::new(x, y - 1, z);
        let state2 = level.get_block_state(pos2).unwrap_or_else(BlockState::air);
        self.update_edge(level, i1, i2, &pos1, &state1, &pos2, &state2)
    }

    /// `updateEdge(level, index, minY, pos1, state1, pos2, state2)`.
    fn update_edge(
        &mut self,
        level: &dyn LevelReader,
        index: usize,
        min_y_stored: i32,
        pos1: &BlockPos,
        state1: &BlockState,
        pos2: &BlockPos,
        state2: &BlockState,
    ) -> bool {
        let y = pos1.get_y();
        if is_edge_occluded(state1, state2) {
            if y > min_y_stored {
                self.set(index, y);
                return true;
            }
        } else if y == min_y_stored {
            let found = self.find_lowest_source_below(level, pos2, state2);
            self.set(index, found);
            return true;
        }
        false
    }

    /// `findLowestSourceBelow(level, pos, state)`.
    fn find_lowest_source_below(
        &self,
        level: &dyn LevelReader,
        pos: &BlockPos,
        state: &BlockState,
    ) -> i32 {
        let mut pos_cur = *pos;
        let mut pos_below = pos.relative(Direction::DOWN);
        let mut block_state = state.clone();

        while pos_below.get_y() >= self.min_y {
            let state1 = level
                .get_block_state(pos_below)
                .unwrap_or_else(BlockState::air);
            if is_edge_occluded(&block_state, &state1) {
                return pos_cur.get_y();
            }
            block_state = state1;
            pos_cur = pos_below;
            pos_below = pos_below.relative(Direction::DOWN);
        }

        self.min_y
    }

    /// `getLowestSourceY(x, z)`.
    pub fn get_lowest_source_y(&self, x: i32, z: i32) -> i32 {
        let i = self.get(index(x, z));
        self.extend_sources_below_world(i)
    }

    /// `getHighestLowestSourceY()`.
    pub fn get_highest_lowest_source_y(&self) -> i32 {
        let mut i = i32::MIN;
        for i1 in 0..self.heightmap.get_size() {
            let i2 = self.heightmap.get(i1);
            if i2 > i {
                i = i2;
            }
        }
        self.extend_sources_below_world(i + self.min_y)
    }

    /// `fill(value)`.
    fn fill(&mut self, value: i32) {
        let i = value - self.min_y;
        for i1 in 0..self.heightmap.get_size() {
            self.heightmap.set(i1, i);
        }
    }

    /// `set(index, value)`.
    fn set(&mut self, index: usize, value: i32) {
        self.heightmap.set(index, value - self.min_y);
    }

    /// `get(index)`.
    fn get(&self, index: usize) -> i32 {
        self.heightmap.get(index) + self.min_y
    }

    /// `extendSourcesBelowWorld(y)`.
    fn extend_sources_below_world(&self, y: i32) -> i32 {
        if y == self.min_y {
            NEGATIVE_INFINITY
        } else {
            y
        }
    }
}

/// `isEdgeOccluded(state1, state2)`.
fn is_edge_occluded(state1: &BlockState, state2: &BlockState) -> bool {
    if state2.get_light_block() != 0 {
        return true;
    }
    let occlusion_shape = get_occlusion_shape(state1, Direction::DOWN);
    let occlusion_shape1 = get_occlusion_shape(state2, Direction::UP);
    crate::world::phys::shapes::face_shape_occludes(&occlusion_shape, &occlusion_shape1)
}

fn index(x: i32, z: i32) -> usize {
    (x + z * SIZE as i32) as usize
}
