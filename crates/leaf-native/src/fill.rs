use std::collections::HashMap;

use leaf_core::core::{block_pos::BlockPos, section_pos::SectionPos};
use leaf_core::network::protocol::game::clientbound_section_blocks_update_packet::ClientboundSectionBlocksUpdatePacket;

use crate::audience::Audience;
use crate::blocks::{Block, Blocks};

use super::{set_state, state_at, FlatWorld, AIR};

const WORLD_BORDER: i32 = 30_000_000;

#[derive(Clone, Copy)]
pub enum FillMode {
    Replace(Option<i32>),
    Keep,
    Outline,
    Hollow,
    Destroy,
}

pub struct FillResult {
    pub changed: usize,
    pub destroyed: Vec<(BlockPos, i32)>,
}

impl FlatWorld {
    pub fn fill(
        &self,
        from: BlockPos,
        to: BlockPos,
        state: i32,
        mode: FillMode,
        limit: i32,
        audience: &Audience,
    ) -> Result<FillResult, String> {
        for pos in [from, to] {
            let horizontal = -WORLD_BORDER..WORLD_BORDER;
            if !Self::editable(pos) || !horizontal.contains(&pos.x) || !horizontal.contains(&pos.z)
            {
                return Err("The fill area is outside the world".into());
            }
        }

        let min = BlockPos::new(from.x.min(to.x), from.y.min(to.y), from.z.min(to.z));
        let max = BlockPos::new(from.x.max(to.x), from.y.max(to.y), from.z.max(to.z));
        let volume = i64::from(max.x - min.x + 1)
            * i64::from(max.y - min.y + 1)
            * i64::from(max.z - min.z + 1);
        if volume > i64::from(limit) {
            return Err(format!(
                "Too many blocks in the fill area ({volume}; maximum {limit})"
            ));
        }

        let blocks = Blocks::vanilla();
        let mut edits = self.edits.lock().unwrap();
        let mut updates = HashMap::<SectionPos, (Vec<i16>, Vec<i32>)>::new();
        let mut result = FillResult {
            changed: 0,
            destroyed: Vec::new(),
        };
        for x in min.x..=max.x {
            for y in min.y..=max.y {
                for z in min.z..=max.z {
                    let pos = BlockPos::new(x, y, z);
                    let on_surface = x == min.x
                        || x == max.x
                        || y == min.y
                        || y == max.y
                        || z == min.z
                        || z == max.z;
                    let previous = state_at(&edits, pos);
                    let block = blocks.get(previous);
                    let Some(next) = mode.state(state, block, on_surface) else {
                        continue;
                    };
                    let destroys_block = matches!(mode, FillMode::Destroy) && !is_air(block);
                    if previous == next && !destroys_block {
                        continue;
                    }
                    if destroys_block && block.hardness >= 0 {
                        if let Some(item) = block.item {
                            result.destroyed.push((pos, item));
                        }
                    }
                    result.changed += 1;
                    set_state(&mut edits, pos, next);
                    let (positions, states) = updates.entry(SectionPos::of(&pos)).or_default();
                    positions.push(SectionPos::section_relative_pos(&pos));
                    states.push(next);
                }
            }
        }
        if result.changed == 0 {
            return Err("No blocks were filled".into());
        }

        audience.broadcast_batch(updates.into_iter().map(|(section, (positions, states))| {
            let chunk = (section.x, section.z);
            let packet = ClientboundSectionBlocksUpdatePacket::new(section, positions, states);
            (chunk, packet)
        }));
        Ok(result)
    }
}

impl FillMode {
    fn state(self, state: i32, previous: &Block, on_surface: bool) -> Option<i32> {
        match self {
            Self::Replace(Some(filter)) if previous.default != filter => None,
            Self::Keep if !is_air(previous) => None,
            Self::Outline if !on_surface => None,
            Self::Hollow if !on_surface => Some(AIR),
            _ => Some(state),
        }
    }
}

fn is_air(block: &Block) -> bool {
    matches!(block.name, "air" | "cave_air" | "void_air")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_modes_preserve_interior_filter_and_drop_behavior() {
        let world = FlatWorld::new(0);
        let audience = Audience::default();
        let blocks = Blocks::vanilla();
        let tnt = blocks.named("tnt").unwrap();
        let stone = blocks.named("stone").unwrap().default;
        let from = BlockPos::new(0, 16, 0);
        let to = BlockPos::new(2, 18, 2);
        let middle = BlockPos::new(1, 17, 1);
        let fill = |from, to, state, mode| world.fill(from, to, state, mode, 32, &audience);
        assert_eq!(
            fill(from, to, tnt.default, FillMode::Outline)
                .unwrap()
                .changed,
            26
        );
        assert_eq!(world.block(middle), AIR);
        assert_eq!(fill(from, to, stone, FillMode::Keep).unwrap().changed, 1);
        assert_eq!(world.block(middle), stone);
        assert_eq!(
            fill(from, to, stone, FillMode::Replace(Some(tnt.default)))
                .unwrap()
                .changed,
            26
        );
        assert_eq!(fill(from, to, stone, FillMode::Hollow).unwrap().changed, 1);
        assert_eq!(world.block(middle), AIR);
        fill(middle, middle, tnt.default, FillMode::Keep).unwrap();
        let result = fill(middle, middle, AIR, FillMode::Destroy).unwrap();
        assert_eq!(result.changed, 1);
        assert_eq!(result.destroyed, [(middle, tnt.item.unwrap())]);
        assert_eq!(world.block(middle), AIR);
    }
}
