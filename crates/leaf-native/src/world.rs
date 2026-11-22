use std::collections::HashSet;
use std::io;

use leaf_core::network::friendly_byte_buf::FriendlyByteBuf;
use leaf_core::network::protocol::game::{
    clientbound_chunk_batch_finished_packet::ClientboundChunkBatchFinishedPacket,
    clientbound_chunk_batch_start_packet::ClientboundChunkBatchStartPacket,
    clientbound_forget_level_chunk_packet::ClientboundForgetLevelChunkPacket,
    clientbound_level_chunk_packet_data::ClientboundLevelChunkPacketData,
    clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket,
    clientbound_light_update_packet_data::ClientboundLightUpdatePacketData,
    clientbound_set_chunk_cache_center_packet::ClientboundSetChunkCacheCenterPacket,
};
use leaf_core::world::level::chunk_pos::ChunkPos;

use crate::wire::{PacketIds, Wire};

const MIN_Y: i32 = -64;
const SECTIONS: usize = 24;
const AIR: i32 = 0;
const STONE: i32 = 1;
pub const SPAWN_Y: i32 = 16;
pub const RADIUS: i32 = 2;

pub struct FlatWorld(ClientboundLevelChunkWithLightPacket);

impl FlatWorld {
    pub fn new(biome: i32) -> Self {
        let mut sections = FriendlyByteBuf::new();
        let solid = ((SPAWN_Y - MIN_Y) / 16) as usize;
        for section in 0..SECTIONS {
            let stone = section < solid;
            sections.write_short(if stone { 4096 } else { 0 });
            // Single-value palettes in 1.21.11 have no storage longs.
            sections
                .write_byte(0)
                .write_var_int(if stone { STONE } else { AIR });
            sections.write_byte(0).write_var_int(biome);
        }
        let data = ClientboundLevelChunkPacketData::new(
            [(1, heightmap()), (4, heightmap())].into_iter().collect(),
            sections.copy_bytes(),
            Vec::new(),
        );

        let layers = SECTIONS + 2;
        let sky = solid + 1;
        let light = ClientboundLightUpdatePacketData::new(
            (0..layers).map(|index| index >= sky).collect(),
            Vec::new(),
            (0..layers).map(|index| index < sky).collect(),
            vec![true; layers],
            vec![vec![0xff; 2048]; layers - sky],
            Vec::new(),
        );
        Self(ClientboundLevelChunkWithLightPacket::new(0, 0, data, light))
    }

    pub fn update(
        &self,
        wire: &mut Wire,
        ids: &PacketIds,
        center: (i32, i32),
        loaded: &mut HashSet<(i32, i32)>,
    ) -> io::Result<()> {
        wire.send(
            ids,
            &ClientboundSetChunkCacheCenterPacket::new(center.0, center.1),
        )?;
        let mut wanted = Vec::new();
        for x in center.0 - RADIUS..=center.0 + RADIUS {
            for z in center.1 - RADIUS..=center.1 + RADIUS {
                wanted.push((x, z));
            }
        }
        // Send the destination chunk first so the client can complete loading.
        wanted.sort_by_key(|&(x, z)| (x - center.0).pow(2) + (z - center.1).pow(2));
        let target: HashSet<_> = wanted.iter().copied().collect();
        for &(x, z) in loaded.difference(&target) {
            wire.send(
                ids,
                &ClientboundForgetLevelChunkPacket::new(ChunkPos::new(x, z)),
            )?;
        }

        let fresh: Vec<_> = wanted
            .into_iter()
            .filter(|pos| !loaded.contains(pos))
            .collect();
        if !fresh.is_empty() {
            wire.send(ids, &ClientboundChunkBatchStartPacket::instance())?;
            for &(x, z) in &fresh {
                let packet = ClientboundLevelChunkWithLightPacket::new(
                    x,
                    z,
                    self.0.chunk_data.clone(),
                    self.0.light_data.clone(),
                );
                wire.send(ids, &packet)?;
            }
            wire.send(
                ids,
                &ClientboundChunkBatchFinishedPacket::new(fresh.len() as i32),
            )?;
        }
        *loaded = target;
        Ok(())
    }
}

fn heightmap() -> Vec<i64> {
    // The 384-block dimension uses nine bits per entry, seven entries per long.
    let mut data = vec![0; 256usize.div_ceil(7)];
    let height = i64::from(SPAWN_Y - MIN_Y);
    for column in 0..256 {
        data[column / 7] |= height << (9 * (column % 7));
    }
    data
}
