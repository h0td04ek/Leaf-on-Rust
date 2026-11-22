use std::collections::{HashSet, VecDeque};
use std::io;
use std::time::{Duration, Instant};

use leaf_core::network::protocol::game::{
    clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket,
    clientbound_chunk_batch_start_packet::ClientboundChunkBatchStartPacket,
    clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket,
    clientbound_set_chunk_cache_center_packet::ClientboundSetChunkCacheCenterPacket,
    clientbound_set_chunk_cache_radius_packet::ClientboundSetChunkCacheRadiusPacket,
};
use leaf_core::world::level::chunk_pos::ChunkPos;

use crate::wire::{Cursor, PacketIds, Wire};
use crate::world::FlatWorld;

pub const MAX_VIEW_DISTANCE: i32 = 32;
pub const SIMULATION_DISTANCE: i32 = 2;

const TICK: Duration = Duration::from_millis(50);
const INITIAL_BATCH: usize = 9;
const MAX_BATCH: usize = 64;

type Chunk = (i32, i32);

pub struct ChunkStream {
    center: Chunk,
    radius: i32,
    offsets: Vec<Chunk>,
    loaded: HashSet<Chunk>,
    pending: VecDeque<Chunk>,
    batch_size: usize,
    batch_in_flight: bool,
    next_batch: Instant,
}

impl ChunkStream {
    pub fn start(center: Chunk, radius: i32, wire: &mut Wire, ids: &PacketIds) -> io::Result<Self> {
        let mut stream = Self {
            center,
            radius,
            offsets: offsets(radius),
            loaded: HashSet::new(),
            pending: VecDeque::new(),
            batch_size: INITIAL_BATCH,
            batch_in_flight: false,
            next_batch: Instant::now(),
        };
        wire.send(
            ids,
            &ClientboundSetChunkCacheCenterPacket::new(center.0, center.1),
        )?;
        stream.rebuild_queue(wire, ids)?;
        Ok(stream)
    }

    pub fn restart(&mut self, center: Chunk, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        self.center = center;
        self.loaded.clear();
        wire.send(
            ids,
            &ClientboundSetChunkCacheCenterPacket::new(center.0, center.1),
        )?;
        self.rebuild_queue(wire, ids)
    }

    pub fn loaded(&self) -> &HashSet<Chunk> {
        &self.loaded
    }

    pub fn move_to(&mut self, center: Chunk, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        if center == self.center {
            return Ok(());
        }
        self.center = center;
        wire.send(
            ids,
            &ClientboundSetChunkCacheCenterPacket::new(center.0, center.1),
        )?;
        self.rebuild_queue(wire, ids)
    }

    pub fn resize(&mut self, radius: i32, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        if radius == self.radius {
            return Ok(());
        }
        self.radius = radius;
        self.offsets = offsets(radius);
        wire.send(ids, &ClientboundSetChunkCacheRadiusPacket::new(radius))?;
        self.rebuild_queue(wire, ids)
    }

    fn rebuild_queue(&mut self, wire: &mut Wire, ids: &PacketIds) -> io::Result<()> {
        let (x, z) = self.center;
        let outside: Vec<_> = self
            .loaded
            .iter()
            .copied()
            .filter(|&(cx, cz)| !in_view(cx - x, cz - z, self.radius))
            .collect();
        for (cx, cz) in outside {
            wire.send(
                ids,
                &ClientboundForgetLevelChunkPacket::new(ChunkPos::new(cx, cz)),
            )?;
            self.loaded.remove(&(cx, cz));
        }
        self.pending.clear();
        self.pending.extend(
            self.offsets
                .iter()
                .map(|&(dx, dz)| (x + dx, z + dz))
                .filter(|pos| !self.loaded.contains(pos)),
        );
        Ok(())
    }

    pub fn acknowledge(&mut self, desired: f32) {
        if self.batch_in_flight {
            self.batch_size = if desired.is_nan() {
                1
            } else {
                desired.clamp(1.0, MAX_BATCH as f32) as usize
            };
            self.batch_in_flight = false;
        }
    }

    pub fn send_next(
        &mut self,
        world: &FlatWorld,
        wire: &mut Wire,
        ids: &PacketIds,
    ) -> io::Result<()> {
        if self.batch_in_flight || self.pending.is_empty() || Instant::now() < self.next_batch {
            return Ok(());
        }
        wire.send(ids, &ClientboundChunkBatchStartPacket::instance())?;
        let count = self.batch_size.min(self.pending.len());
        for _ in 0..count {
            let (x, z) = self.pending.pop_front().unwrap();
            world.send_chunk(x, z, wire, ids)?;
            self.loaded.insert((x, z));
        }
        wire.send(ids, &ClientboundChunkBatchFinishedPacket::new(count as i32))?;
        self.batch_in_flight = true;
        self.next_batch = Instant::now() + TICK;
        Ok(())
    }
}

pub fn client_view_distance(mut information: Cursor) -> io::Result<i32> {
    information.string(16)?; // Locale precedes the signed view-distance byte.
    let [requested] = information.fixed()?;
    Ok(i32::from(requested as i8).clamp(2, MAX_VIEW_DISTANCE))
}

fn in_view(dx: i32, dz: i32, radius: i32) -> bool {
    let (dx, dz) = (dx.abs(), dz.abs());
    dx <= radius && dz <= radius && (dx - 1).max(0).pow(2) + (dz - 1).max(0).pow(2) < radius.pow(2)
}

fn offsets(radius: i32) -> Vec<Chunk> {
    let mut result = Vec::new();
    for x in -radius..=radius {
        for z in -radius..=radius {
            if in_view(x, z, radius) {
                result.push((x, z));
            }
        }
    }
    result.sort_unstable_by_key(|&(x, z)| (x * x + z * z, x, z));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_nearby_chunks_first_and_reaches_the_32_chunk_border() {
        let chunks = offsets(MAX_VIEW_DISTANCE);
        assert!(chunks.len() > 3200);
        assert_eq!(chunks[0], (0, 0));
        assert!(chunks.windows(2).all(|pair| {
            let distance = |(x, z): Chunk| x * x + z * z;
            distance(pair[0]) <= distance(pair[1])
        }));
        for edge in [(-32, 0), (32, 0), (0, -32), (0, 32)] {
            assert!(chunks.contains(&edge));
        }
        assert!(!in_view(33, 0, MAX_VIEW_DISTANCE));
        assert!(!in_view(32, 32, MAX_VIEW_DISTANCE));
    }
}
