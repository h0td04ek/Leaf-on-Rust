//! `ca.spottedleaf.moonrise.patches.starlight.light.BlockStarLightEngine` —
//! block-light движок StarLight. Полный порт Java-класса.

use std::collections::HashSet;

use crate::core::block_pos::BlockPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::world::phys::shapes::voxel_shape::VoxelShape;

use super::star_light_engine::{
    AxisDirection, StarLightEngine, FLAG_HAS_SIDED_TRANSPARENT_BLOCKS,
    LEVEL_SHIFT, DIR_SHIFT,
};

/// `initNibble(chunkX, chunkY, chunkZ, extrude, initRemovedNibbles)`.
pub(crate) fn init_nibble(
    engine: &mut StarLightEngine,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    _extrude: bool,
    init_removed_nibbles: bool,
) {
    if chunk_y < engine.min_light_section
        || chunk_y > engine.max_light_section
        || engine.get_chunk_in_cache(chunk_x, chunk_z).is_none()
    {
        return;
    }

    let nibble = engine.get_nibble_from_cache(chunk_x, chunk_y, chunk_z);
    match nibble {
        None => {
            if !init_removed_nibbles {
                panic!("initNibble: nibble missing");
            }
            engine.set_nibble_in_cache(
                chunk_x,
                chunk_y,
                chunk_z,
                Some(std::sync::Arc::new(std::sync::Mutex::new(
                    super::swmr_nibble_array::SWMRNibbleArray::new(),
                ))),
            );
        }
        Some(n) => {
            n.lock().unwrap().set_non_null();
        }
    }
}

/// `setNibbleNull(chunkX, chunkY, chunkZ)` — block-light "прячет" ниббл
/// (`setHidden`), а не удаляет: деинициализация block light обычно
/// вызвана удалением блока, что может уменьшать свет, и decrease-
/// распространению нужны данные.
pub(crate) fn set_nibble_null(engine: &mut StarLightEngine, chunk_x: i32, chunk_y: i32, chunk_z: i32) {
    if let Some(nibble) = engine.get_nibble_from_cache(chunk_x, chunk_y, chunk_z) {
        nibble.lock().unwrap().set_hidden();
    }
}

/// `checkBlock(lightAccess, worldX, worldY, worldZ)` — block-версия.
pub(crate) fn check_block(
    engine: &mut StarLightEngine,
    _light_access: &dyn LightChunkGetter,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) {
    // blocks can change opacity
    // blocks can change emitted light
    // blocks can change direction of propagation

    let emitted_mask = engine.emitted_light_mask;

    let current_level = engine.get_light_level(world_x, world_y, world_z);
    let block_state = engine.get_block_state(world_x, world_y, world_z);
    let emitted_level =
        (block_state.get_light_emission() as i32 & emitted_mask) as u8;

    engine.set_light_level(world_x, world_y, world_z, emitted_level);
    // this accounts for change in emitted light that would cause an increase
    if emitted_level != 0 {
        engine.append_to_increase_queue(
            engine.encode_position(world_x, world_y, world_z)
                | (((emitted_level as i64) & 0xF) << LEVEL_SHIFT)
                | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << DIR_SHIFT)
                | if block_state.starlight_is_conditionally_full_opaque() {
                    FLAG_HAS_SIDED_TRANSPARENT_BLOCKS
                } else {
                    0
                },
        );
    }
    // this also accounts for a change in emitted light that would cause a decrease
    // this also accounts for the change of direction of propagation (i.e old block was
    // full transparent, new block is full opaque or vice versa)
    // as it checks all neighbours (even if current level is 0)
    engine.append_to_decrease_queue(
        engine.encode_position(world_x, world_y, world_z)
            | (((current_level as i64) & 0xF) << LEVEL_SHIFT)
            | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << DIR_SHIFT),
        // always keep sided transparent false here, new block might be conditionally
        // transparent which would prevent us from decreasing sources in the directions
        // where the new block is opaque; if it turns out we were wrong to de-propagate
        // the source, the re-propagate logic WILL always catch that and fix it.
    );
}

/// `calculateLightValue` — block-версия.
pub(crate) fn calculate_light_value(
    engine: &StarLightEngine,
    _light_access: &dyn LightChunkGetter,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    expect: u8,
) -> u8 {
    let center_state = engine.get_block_state(world_x, world_y, world_z);
    let mut level =
        (center_state.get_light_emission() as i32 & engine.emitted_light_mask) as i32;

    if level >= (15 - 1) || level > expect as i32 {
        return level as u8;
    }

    let opacity = center_state.get_light_block().max(1);
    if opacity >= 15 {
        return level as u8;
    }
    let conditionally_opaque_state: Option<BlockState> =
        if center_state.starlight_is_conditionally_full_opaque() {
            Some(center_state.clone())
        } else {
            None
        };

    let _section_offset = engine.chunk_section_index_offset;
    for direction in AxisDirection::VALUES {
        let off_x = world_x + direction.x();
        let off_y = world_y + direction.y();
        let off_z = world_z + direction.z();

        let section_index =
            engine.chunk_section_index(off_x >> 4, off_y >> 4, off_z >> 4);

        let neighbour_level = engine.get_light_level_at(
            section_index,
            ((off_x & 15) | (((off_z & 15) << 4)) | (((off_y & 15) << 8))) as usize,
        ) as i32;

        if (neighbour_level - 1) <= level {
            // don't need to test transparency, we know it wont affect the result.
            continue;
        }

        let neighbour_state = engine.get_block_state(off_x, off_y, off_z);
        if neighbour_state.starlight_is_conditionally_full_opaque() {
            // here the block can be conditionally opaque (i.e light cannot propagate
            // from it), so we need to test that we don't read the blockstate because
            // most of the time this is false, so using the faster known transparency
            // lookup results in a net win
            let neighbour_face = neighbour_state
                .face_occlusion_shape_unleveled(direction.get_opposite().nms());
            let this_face = match &conditionally_opaque_state {
                Some(s) => s.face_occlusion_shape_unleveled(direction.nms()),
                None => VoxelShape::empty(),
            };
            if crate::world::phys::shapes::face_shape_occludes(&this_face, &neighbour_face) {
                // not allowed to propagate
                continue;
            }
        }

        // passed transparency,

        let calculated = neighbour_level - opacity;
        level = calculated.max(level);
        if level > expect as i32 {
            return level as u8;
        }
    }

    level as u8
}

/// `propagateBlockChanges` — block-версия.
pub(crate) fn propagate_block_changes(
    engine: &mut StarLightEngine,
    light_access: &dyn LightChunkGetter,
    _at_chunk: &ChunkAccess,
    positions: &HashSet<BlockPos>,
) {
    for pos in positions {
        engine.check_block(light_access, pos.get_x(), pos.get_y(), pos.get_z());
    }

    engine.perform_light_decrease(light_access);
}

/// `getSources(lightAccess, chunk)` — все блоки с ненулевым излучением.
fn get_sources(engine: &StarLightEngine, chunk: &ChunkAccess) -> Vec<BlockPos> {
    let mut sources = Vec::new();

    let off_x = chunk.get_pos().x << 4;
    let off_z = chunk.get_pos().z << 4;

    let min_section = engine.min_section;
    let max_section = engine.max_section;

    for section_y in min_section..=max_section {
        let section = match chunk
            .sections
            .get((section_y - min_section) as usize)
        {
            Some(s) => s,
            None => continue,
        };
        if section.has_only_air() {
            // no sources in empty sections
            continue;
        }
        if !section.maybe_has(&|s: &BlockState| s.get_light_emission() != 0) {
            // no light sources in palette (PlatformHooks.maybeHasLightEmission)
            continue;
        }
        let off_y = section_y << 4;

        for index in 0..(16 * 16 * 16) {
            let state = section.states.get_by_index(index);
            if state.get_light_emission() == 0 {
                continue;
            }
            // index = x | (z << 4) | (y << 8)
            sources.push(BlockPos::new(
                off_x | (index & 15) as i32,
                off_y | ((index >> 8) & 15) as i32,
                off_z | (((index >> 4) & 15) as i32),
            ));
        }
    }

    sources
}

/// `lightChunk(lightAccess, chunk, needsEdgeChecks)` — block-версия.
pub(crate) fn light_chunk(
    engine: &mut StarLightEngine,
    light_access: &dyn LightChunkGetter,
    chunk: &ChunkAccess,
    needs_edge_checks: bool,
) {
    // setup sources
    let emitted_mask = engine.emitted_light_mask;
    let positions = get_sources(engine, chunk);
    for pos in &positions {
        let block_state = engine.get_block_state(pos.get_x(), pos.get_y(), pos.get_z());
        let emitted_light =
            (block_state.get_light_emission() as i32 & emitted_mask) as u8;

        if emitted_light <= engine.get_light_level(pos.get_x(), pos.get_y(), pos.get_z()) {
            // some other source is brighter
            continue;
        }

        engine.append_to_increase_queue(
            engine.encode_position(pos.get_x(), pos.get_y(), pos.get_z())
                | (((emitted_light as i64) & 0xF) << LEVEL_SHIFT)
                | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << DIR_SHIFT)
                | if block_state.starlight_is_conditionally_full_opaque() {
                    FLAG_HAS_SIDED_TRANSPARENT_BLOCKS
                } else {
                    0
                },
        );

        // propagation wont set this for us
        engine.set_light_level(pos.get_x(), pos.get_y(), pos.get_z(), emitted_light);
    }

    if needs_edge_checks {
        // not required to propagate here, but this will reduce the hit of the edge checks
        engine.perform_light_increase(light_access);

        // verify neighbour edges
        engine.check_chunk_edges_range_base(
            light_access,
            chunk,
            engine.min_light_section,
            engine.max_light_section,
        );
    } else {
        engine.propagate_neighbour_levels(
            light_access,
            chunk,
            engine.min_light_section,
            engine.max_light_section,
        );

        engine.perform_light_increase(light_access);
    }
}
