use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use leaf_core::core::{block_pos::BlockPos, section_pos::SectionPos};
use leaf_core::network::protocol::game::clientbound_section_blocks_update_packet::ClientboundSectionBlocksUpdatePacket;
use leaf_core::random::RandomSource;
use leaf_core::world::phys::vec3::Vec3;

use crate::audience::Audience;
use crate::blocks::Blocks;

use super::{matching_half, set_state, state_at, FlatWorld, AIR};

impl FlatWorld {
    pub fn blast(
        &self,
        center: Vec3,
        power: f32,
        random: &mut impl RandomSource,
        audience: &Audience,
    ) -> Vec<(BlockPos, i32)> {
        let mut edits = self.edits.lock().unwrap();
        let blocks = Blocks::vanilla();
        let mut affected = HashSet::new();
        for step in rays() {
            let mut position = center;
            let mut energy = power * (0.7 + random.next_float() * 0.6);
            while energy > 0.0 {
                let pos = BlockPos::containing_vec3(&position);
                if !Self::editable(pos) {
                    break;
                }
                let state = state_at(&edits, pos);
                if state != AIR {
                    let block = blocks.get(state);
                    energy -= (block.blast_resistance + 0.3) * 0.3;
                    if energy > 0.0 && block.hardness >= 0 && affected.insert(pos) {
                        if let Some((other, property, value)) = block.paired(pos, state) {
                            if matching_half(block, state_at(&edits, other), property, value) {
                                affected.insert(other);
                            }
                        }
                    }
                }
                energy -= 0.225;
                position = position.add_vec(step);
            }
        }
        let mut removed = Vec::with_capacity(affected.len());
        let mut sections: HashMap<SectionPos, Vec<i16>> = HashMap::new();
        for pos in affected {
            removed.push((pos, state_at(&edits, pos)));
            set_state(&mut edits, pos, AIR);
            sections
                .entry(SectionPos::of(&pos))
                .or_default()
                .push(SectionPos::section_relative_pos(&pos));
        }
        for (section, positions) in sections {
            let chunk = (section.x, section.z);
            let states = vec![AIR; positions.len()];
            audience.broadcast(
                ClientboundSectionBlocksUpdatePacket::new(section, positions, states),
                Some(chunk),
            );
        }
        removed
    }
}

fn rays() -> &'static [Vec3] {
    static RAYS: OnceLock<Vec<Vec3>> = OnceLock::new();
    RAYS.get_or_init(|| {
        let mut rays = Vec::new();
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    if x == 0 || x == 15 || y == 0 || y == 15 || z == 0 || z == 15 {
                        let map = |coordinate| f64::from(coordinate) / 15.0 * 2.0 - 1.0;
                        rays.push(Vec3::new(map(x), map(y), map(z)).normalize().scale(0.3));
                    }
                }
            }
        }
        rays
    })
}
