//! Перенос `net.minecraft.world.level.NoiseColumn` — колонка
//! сгенерированных блоков (результат `ChunkGenerator.getBaseColumn`).

use crate::core::block_pos::BlockPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;

/// `NoiseColumn` — столбец block-state по Y (индекс 0 = minY).
pub struct NoiseColumn {
    /// `states` — block-state'ы снизу вверх.
    states: Vec<BlockState>,
    min_y: i32,
}

impl NoiseColumn {
    /// `new(minY, states)`.
    pub fn new(min_y: i32, states: Vec<BlockState>) -> Self {
        Self { states, min_y }
    }

    /// `getBlock(y)` — state на абсолютной высоте y
    /// (вне диапазона → air).
    pub fn get_block(&self, y: i32) -> BlockState {
        let idx = (y - self.min_y) as usize;
        self.states.get(idx).cloned().unwrap_or_else(BlockState::air)
    }

    /// `minY`.
    pub fn min_y(&self) -> i32 {
        self.min_y
    }

    /// `build_from_level(level, x, z)` — колонка из `get_block_state`
    /// (default-реализация `LevelReader::get_base_column`).
    /// Дженерик по `?Sized`, чтобы работал и для `&dyn LevelReader`,
    /// и для `Self` внутри default-методов трейта.
    pub fn build_from_level<L: LevelReader + ?Sized>(level: &L, x: i32, z: i32) -> Self {
        let min_y = level.min_build_height();
        let max_y = min_y + level.height();
        let mut states = Vec::with_capacity((max_y - min_y) as usize);
        for y in min_y..max_y {
            states.push(
                level
                    .get_block_state(BlockPos::new(x, y, z))
                    .unwrap_or_else(BlockState::air),
            );
        }
        Self { states, min_y }
    }
}
