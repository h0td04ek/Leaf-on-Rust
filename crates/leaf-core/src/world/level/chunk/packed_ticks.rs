//! Список тиков чанка с задержками относительно момента сохранения.

use crate::nbt::{CompoundTag, Tag};
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::scheduled_tick_access::{BlockTickType, FluidTickType};
use crate::world::ticks::scheduled_tick::TickNbt;
use crate::world::ticks::SavedTick;

#[derive(Default)]
pub struct PackedTicks {
    pub blocks: Vec<SavedTick<BlockTickType>>,
    pub fluids: Vec<SavedTick<FluidTickType>>,
}

impl PackedTicks {
    pub fn from_nbt(tag: &CompoundTag, pos: ChunkPos) -> Self {
        Self {
            blocks: read_ticks(tag.get_list_or_empty("block_ticks"), pos),
            fluids: read_ticks(tag.get_list_or_empty("fluid_ticks"), pos),
        }
    }

    pub fn write_nbt(&self, tag: &mut CompoundTag) {
        tag.put("block_ticks", write_ticks(&self.blocks));
        tag.put("fluid_ticks", write_ticks(&self.fluids));
    }
}

fn read_ticks<T: TickNbt>(list: &[Tag], pos: ChunkPos) -> Vec<SavedTick<T>> {
    list.iter()
        .filter_map(|entry| SavedTick::read_nbt(entry.as_compound()?))
        .filter(|tick| tick.pos.get_x() >> 4 == pos.x && tick.pos.get_z() >> 4 == pos.z)
        .collect()
}

fn write_ticks<T: TickNbt>(ticks: &[SavedTick<T>]) -> Tag {
    Tag::List(
        ticks
            .iter()
            .map(|tick| {
                let mut tag = CompoundTag::new();
                tick.write_nbt(&mut tag);
                Tag::Compound(tag)
            })
            .collect(),
    )
}
