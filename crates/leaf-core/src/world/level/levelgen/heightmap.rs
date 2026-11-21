//! `net.minecraft.world.level.levelgen.Heightmap` — типы высотных карт
//! и предикаты подсчёта блоков.

use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::BlockKind;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_height_accessor::LevelHeightAccessor;

/// `Heightmap.Types`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeightmapType {
    /// `WORLD_SURFACE_WG`.
    WorldSurfaceWg,
    /// `WORLD_SURFACE`.
    WorldSurface,
    /// `OCEAN_FLOOR_WG`.
    OceanFloorWg,
    /// `OCEAN_FLOOR`.
    OceanFloor,
    /// `MOTION_BLOCKING`.
    MotionBlocking,
    /// `MOTION_BLOCKING_NO_LEAVES`.
    MotionBlockingNoLeaves,
}

impl HeightmapType {
    /// `values()`.
    pub const VALUES: [HeightmapType; 6] = [
        HeightmapType::WorldSurfaceWg,
        HeightmapType::WorldSurface,
        HeightmapType::OceanFloorWg,
        HeightmapType::OceanFloor,
        HeightmapType::MotionBlocking,
        HeightmapType::MotionBlockingNoLeaves,
    ];

    /// `isOpaque()`.
    pub fn is_opaque(&self) -> bool {
        matches!(
            self,
            HeightmapType::WorldSurfaceWg | HeightmapType::OceanFloorWg
        )
    }

    /// `clientRootHeight(lower)` / `clientRootHeight()`.
    pub fn client_root_height(&self, lower: i32) -> i32 {
        match self {
            HeightmapType::WorldSurfaceWg | HeightmapType::WorldSurface => lower + 8,
            HeightmapType::OceanFloorWg
            | HeightmapType::OceanFloor
            | HeightmapType::MotionBlocking
            | HeightmapType::MotionBlockingNoLeaves => lower,
        }
    }

    /// `clientRootHeight()` — от minY.
    pub fn client_root_height_min(&self, min_y: i32) -> i32 {
        self.client_root_height(min_y)
    }

    /// `isBlockCounted(state)` — учитывается ли блок в карте.
    pub fn is_block_counted(&self, state: &BlockState) -> bool {
        match self {
            HeightmapType::WorldSurfaceWg | HeightmapType::WorldSurface => !state.is_air(),
            HeightmapType::OceanFloorWg | HeightmapType::OceanFloor => state.blocks_motion(),
            HeightmapType::MotionBlocking => {
                state.blocks_motion() || !state.get_fluid_state().is_empty()
            }
            HeightmapType::MotionBlockingNoLeaves => {
                (state.blocks_motion() || !state.get_fluid_state().is_empty())
                    && !matches!(state.block.kind, BlockKind::Leaves)
            }
        }
    }
}

/// `Heightmap` — одна высотная карта чанка (16×16 колонок).
pub struct Heightmap {
    /// `chunkPos`.
    pub chunk_pos: ChunkPos,
    /// `data` — packed longs (Y в 9 битах на колонку... в Java — int[]).
    /// Здесь храним i32-высоты для каждой из 256 колонок.
    data: [i32; 256],
    /// `height` accessor (minY/maxY).
    pub height_accessor_min_y: i32,
}

impl Heightmap {
    /// `new Heightmap(chunkPos, type, level, lower)`.
    pub fn new(
        chunk_pos: ChunkPos,
        ty: HeightmapType,
        level: &dyn LevelHeightAccessor,
        lower: i32,
    ) -> Self {
        let _ = ty;
        let min_y = lower;
        let mut data = [0i32; 256];
        data.fill(min_y);
        Self {
            chunk_pos,
            data,
            height_accessor_min_y: min_y,
        }
    }

    /// `getFirstAvailable(x, z)`.
    pub fn get_first_available(&self, x: i32, z: i32) -> i32 {
        self.data[(((x & 15) << 4) | (z & 15)) as usize]
    }

    /// `setHeight(x, z, y)`.
    pub fn set_height(&mut self, x: i32, z: i32, y: i32) {
        self.data[(((x & 15) << 4) | (z & 15)) as usize] = y;
    }

    /// Обновляет первую свободную высоту колонки после изменения блока.
    pub fn update(
        &mut self,
        x: i32,
        y: i32,
        z: i32,
        state: &BlockState,
        ty: HeightmapType,
        lower_state: impl Fn(i32) -> BlockState,
    ) {
        let first = self.get_first_available(x, z);
        if y < first - 1 {
            return;
        }
        if ty.is_block_counted(state) {
            if y >= first {
                self.set_height(x, z, y + 1);
            }
        } else if y == first - 1 {
            let min_y = self.height_accessor_min_y;
            let surface = (min_y..y)
                .rev()
                .find(|&below| ty.is_block_counted(&lower_state(below)));
            self.set_height(x, z, surface.map_or(min_y, |below| below + 1));
        }
    }
}
