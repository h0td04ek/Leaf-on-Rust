use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

use leaf_core::core::block_pos::BlockPos;
use leaf_core::random::RandomSource;
use leaf_core::world::phys::vec3::Vec3;

use crate::audience::Audience;
use crate::blocks::Blocks;
use crate::workers::Workers;

use super::{
    broadcast_blocks, fluid_faces, fluids, matching_half, set_state, state_at, BlockView,
    NativeWorld, AIR,
};

impl NativeWorld {
    pub fn blast(
        &self,
        center: Vec3,
        power: f32,
        random: &mut impl RandomSource,
        audience: &Audience,
        workers: &Workers,
    ) -> Vec<(BlockPos, i32)> {
        let mut edits = self.edits.lock().unwrap();
        let world = BlockView(Arc::clone(&edits.chunks), Arc::clone(&edits.terrain));
        let directions = rays();
        let energies = directions
            .iter()
            .map(|_| power * (0.7 + random.next_float() * 0.6))
            .collect::<Vec<_>>();
        // Keep the edit lock through the commit so concurrent block input cannot
        // change the terrain between the raycasts and the removal.
        let batches = workers.map_ranges(directions.len(), 256, move |range| {
            trace_rays(&world, center, &directions[range.clone()], &energies[range])
        });
        let mut affected = batches
            .into_iter()
            .flatten()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        affected.sort_unstable_by_key(|p| (p.x, p.y, p.z));
        let mut removed = Vec::with_capacity(affected.len());
        let mut updates = Vec::with_capacity(affected.len());
        for pos in affected {
            let state = state_at(&edits, pos);
            let remaining = fluids::after_break(state);
            removed.push((pos, state));
            set_state(&mut edits, pos, remaining);
            updates.push((pos, remaining));
        }
        broadcast_blocks(audience, updates);
        removed
    }
}

fn trace_rays(
    world: &BlockView,
    center: Vec3,
    directions: &[Vec3],
    energies: &[f32],
) -> HashSet<BlockPos> {
    let blocks = Blocks::vanilla();
    let mut affected = HashSet::new();
    for (step, &energy) in directions.iter().zip(energies) {
        let mut energy = energy;
        let mut position = center;
        while energy > 0.0 {
            let pos = BlockPos::containing_vec3(&position);
            if !NativeWorld::editable(pos) {
                break;
            }
            let state = world.block(pos);
            if state != AIR {
                let block = blocks.get(state);
                let resistance = if fluid_faces::waterlogged(state) {
                    block.blast_resistance.max(100.0)
                } else {
                    block.blast_resistance
                };
                energy -= (resistance + 0.3) * 0.3;
                if energy > 0.0 && block.hardness >= 0 && affected.insert(pos) {
                    if let Some((other, property, value)) = block.paired(pos, state) {
                        if matching_half(block, world.block(other), property, value) {
                            affected.insert(other);
                        }
                    }
                }
            }
            energy -= 0.225;
            position = position.add_vec(step);
        }
    }
    affected
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
