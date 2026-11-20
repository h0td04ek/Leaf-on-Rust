//! `ca.spottedleaf.moonrise.patches.starlight.light.SkyStarLightEngine` —
//! sky-light движок StarLight. Полный порт Java-класса: инициализация
//! нибблов через emptiness-map, null-секции, вертикальное распространение
//! 15-го уровня света и отложенные light-set'ы при изменении блоков.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use crate::core::block_pos::BlockPos;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::light_chunk_getter::LightChunkGetter;
use crate::world::phys::shapes::voxel_shape::VoxelShape;

use super::star_light_engine::{
    AxisDirection, StarLightEngine, FLAG_HAS_SIDED_TRANSPARENT_BLOCKS,
};
use super::swmr_nibble_array::{NibbleArrayRef, SWMRNibbleArray};

fn new_null_nibble() -> NibbleArrayRef {
    Arc::new(Mutex::new(SWMRNibbleArray::new_null()))
}

/// `initNibble(chunkX, chunkY, chunkZ, extrude, initRemovedNibbles)`.
pub(crate) fn init_nibble(
    engine: &mut StarLightEngine,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    extrude: bool,
    init_removed_nibbles: bool,
) {
    if chunk_y < engine.min_light_section
        || chunk_y > engine.max_light_section
        || engine.get_chunk_in_cache(chunk_x, chunk_z).is_none()
    {
        return;
    }
    let nibble = engine.get_nibble_from_cache(chunk_x, chunk_y, chunk_z);
    let nibble = match nibble {
        Some(n) => n,
        None => {
            if !init_removed_nibbles {
                panic!("initNibble: nibble missing");
            }
            let n = new_null_nibble();
            engine.set_nibble_in_cache(chunk_x, chunk_y, chunk_z, Some(n.clone()));
            n
        }
    };
    init_nibble_inner(engine, &nibble, chunk_x, chunk_y, chunk_z, extrude);
}

/// `initNibble(currNibble, chunkX, chunkY, chunkZ, extrude)`.
fn init_nibble_inner(
    engine: &mut StarLightEngine,
    curr_nibble: &NibbleArrayRef,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    extrude: bool,
) {
    if !curr_nibble.lock().unwrap().is_null_nibble_updating() {
        // already initialised
        return;
    }

    // NB: Java читает кэш-карту (`getEmptinessMap(chunkX, chunkZ)`),
    // а не поле чанка.
    let emptiness_map = engine.get_emptiness_map_cached(chunk_x, chunk_z);

    // are we above this chunk's lowest empty section?
    let mut lowest_y = engine.min_light_section - 1;
    let mut curr_y = engine.max_section;
    while curr_y >= engine.min_section {
        let skip = match &emptiness_map {
            None => {
                // cannot delay nibble init for lit chunks, as we need to init to propagate into them.
                match engine.get_chunk_section(chunk_x, curr_y, chunk_z) {
                    None => true,
                    Some(s) => s.has_only_air(),
                }
            }
            Some(map) => map.lock().unwrap()[(curr_y - engine.min_section) as usize],
        };
        if !skip {
            // should always be full lit here
            lowest_y = curr_y;
            break;
        }
        curr_y -= 1;
    }

    if chunk_y > lowest_y {
        // we need to set this one to full
        if let Some(nibble) = engine.get_nibble_from_cache(chunk_x, chunk_y, chunk_z) {
            let mut n = nibble.lock().unwrap();
            n.set_non_null();
            n.set_full();
        }
        return;
    }

    if extrude {
        // this nibble is going to depend solely on the skylight data above it
        // find first non-null data above (there does exist one, as we just found it above)
        let mut above_y = chunk_y + 1;
        while above_y <= engine.max_light_section {
            let nibble = engine.get_nibble_from_cache(chunk_x, above_y, chunk_z);
            match nibble {
                Some(n)
                    if !n.lock().unwrap().is_null_nibble_updating() =>
                {
                    let mut curr = curr_nibble.lock().unwrap();
                    let src = n.lock().unwrap();
                    curr.set_non_null();
                    curr.extrude_lower(&src);
                    break;
                }
                _ => {}
            }
            above_y += 1;
        }
    } else {
        curr_nibble.lock().unwrap().set_non_null();
    }
}

/// `setNibbleNull(chunkX, chunkY, chunkZ)`.
pub(crate) fn set_nibble_null(engine: &mut StarLightEngine, chunk_x: i32, chunk_y: i32, chunk_z: i32) {
    if let Some(nibble) = engine.get_nibble_from_cache(chunk_x, chunk_y, chunk_z) {
        nibble.lock().unwrap().set_null();
    }
}

/// `rewriteNibbleCacheForSkylight(chunk)` — убирает null-нibbles из кэша.
pub(crate) fn rewrite_nibble_cache_for_skylight(engine: &mut StarLightEngine) {
    for index in 0..engine.nibble_cache.len() {
        let is_null = match &engine.nibble_cache[index] {
            Some(n) => n.lock().unwrap().is_null_nibble_updating(),
            None => false,
        };
        if is_null {
            // stop propagation in these areas
            let nibble = engine.nibble_cache[index].take();
            if let Some(n) = nibble {
                n.lock().unwrap().update_visible();
            }
        }
    }
}

/// `checkNullSection(chunkX, chunkY, chunkZ, extrudeInitialised)` —
/// возвращает, были ли проинициализированы соседи.
pub(crate) fn check_null_section(
    engine: &mut StarLightEngine,
    chunk_x: i32,
    chunk_y: i32,
    chunk_z: i32,
    extrude_initialised: bool,
) -> bool {
    // null chunk sections may have nibble neighbours in the horizontal 1 radius
    // that are non-null. Propagation to these neighbours is necessary.
    if chunk_y < engine.min_light_section
        || chunk_y > engine.max_light_section
        || engine
            .null_propagation_check_cache
            .get((chunk_y - engine.min_light_section) as usize)
            .copied()
            .unwrap_or(false)
    {
        return false;
    }
    engine.null_propagation_check_cache
        [(chunk_y - engine.min_light_section) as usize] = true;

    // check horizontal neighbours
    let mut need_init_neighbours = false;
    'neighbour_search: for dz in -1..=1 {
        for dx in -1..=1 {
            let nibble = engine.get_nibble_from_cache(chunk_x + dx, chunk_y, chunk_z + dz);
            if let Some(n) = nibble {
                if !n.lock().unwrap().is_null_nibble_updating() {
                    need_init_neighbours = true;
                    break 'neighbour_search;
                }
            }
        }
    }

    if need_init_neighbours {
        for dz in -1..=1 {
            for dx in -1..=1 {
                let extrude = if (dx | dz) == 0 {
                    extrude_initialised
                } else {
                    true
                };
                init_nibble(engine, chunk_x + dx, chunk_y, chunk_z + dz, extrude, true);
            }
        }
    }

    need_init_neighbours
}

/// `getLightLevelExtruded(worldX, worldY, worldZ)` — читает уровень,
/// "вытягивая" из ближайшего ненулевого ниббла выше.
pub(crate) fn get_light_level_extruded(
    engine: &StarLightEngine,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) -> u8 {
    let chunk_x = world_x >> 4;
    let mut chunk_y = world_y >> 4;
    let chunk_z = world_z >> 4;

    if let Some(nibble) = engine.get_nibble_from_cache(chunk_x, chunk_y, chunk_z) {
        return nibble.lock().unwrap().get_updating_xyz(world_x, world_y, world_z);
    }

    loop {
        chunk_y += 1;
        if chunk_y > engine.max_light_section {
            return 15;
        }
        if let Some(nibble) = engine.get_nibble_from_cache(chunk_x, chunk_y, chunk_z) {
            return nibble.lock().unwrap().get_updating_xyz(world_x, 0, world_z);
        }
    }
}

/// `checkChunkEdges(lightAccess, chunk, fromSection, toSection)` — sky-версия.
pub(crate) fn check_chunk_edges_range(
    engine: &mut StarLightEngine,
    light_access: &dyn LightChunkGetter,
    chunk: &ChunkAccess,
    from_section: i32,
    to_section: i32,
) {
    for v in engine.null_propagation_check_cache.iter_mut() {
        *v = false;
    }
    rewrite_nibble_cache_for_skylight(engine);
    let chunk_pos = chunk.get_pos();
    let (chunk_x, chunk_z) = (chunk_pos.x, chunk_pos.z);
    let mut y = to_section;
    while y >= from_section {
        check_null_section(engine, chunk_x, y, chunk_z, true);
        y -= 1;
    }

    engine.check_chunk_edges_range_base(light_access, chunk, from_section, to_section);
}

/// `checkChunkEdges(lightAccess, chunk, sections)` — sky-версия.
pub(crate) fn check_chunk_edges_sections(
    engine: &mut StarLightEngine,
    light_access: &dyn LightChunkGetter,
    chunk: &ChunkAccess,
    sections: &[i16],
) {
    for v in engine.null_propagation_check_cache.iter_mut() {
        *v = false;
    }
    rewrite_nibble_cache_for_skylight(engine);
    let chunk_pos = chunk.get_pos();
    let (chunk_x, chunk_z) = (chunk_pos.x, chunk_pos.z);
    for &y in sections {
        check_null_section(engine, chunk_x, y as i32, chunk_z, true);
    }

    engine.check_chunk_edges_sections_base(light_access, chunk, sections);
}

/// `checkBlock(lightAccess, worldX, worldY, worldZ)` — sky-версия.
pub(crate) fn check_block(
    engine: &mut StarLightEngine,
    light_access: &dyn LightChunkGetter,
    world_x: i32,
    world_y: i32,
    world_z: i32,
) {
    // blocks can change opacity
    // blocks can change direction of propagation

    // same logic applies from BlockStarLightEngine#checkBlock
    let encode_offset = engine.coordinate_offset;

    let current_level = engine.get_light_level(world_x, world_y, world_z);

    if current_level == 15 {
        // must re-propagate clobbered source
        engine.append_to_increase_queue(
            engine.encode_position(world_x, world_y, world_z)
                | ((current_level as i64 & 0xF) << super::star_light_engine::LEVEL_SHIFT)
                | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << super::star_light_engine::DIR_SHIFT)
                | FLAG_HAS_SIDED_TRANSPARENT_BLOCKS, // don't know if the block is conditionally transparent
        );
    } else {
        engine.set_light_level(world_x, world_y, world_z, 0);
    }

    engine.append_to_decrease_queue(
        engine.encode_position(world_x, world_y, world_z)
            | ((current_level as i64 & 0xF) << super::star_light_engine::LEVEL_SHIFT)
            | ((AxisDirection::ALL_DIRECTIONS_BITSET as i64) << super::star_light_engine::DIR_SHIFT),
    );
    let _ = light_access;
    let _ = encode_offset;
}

/// `calculateLightValue` — sky-версия.
pub(crate) fn calculate_light_value(
    engine: &StarLightEngine,
    _light_access: &dyn LightChunkGetter,
    world_x: i32,
    world_y: i32,
    world_z: i32,
    expect: u8,
) -> u8 {
    if expect == 15 {
        return expect;
    }

    let _section_offset = engine.chunk_section_index_offset;
    let center_state = engine.get_block_state(world_x, world_y, world_z);

    let conditionally_opaque_state: Option<BlockState> =
        if center_state.starlight_is_conditionally_full_opaque() {
            Some(center_state.clone())
        } else {
            None
        };

    let opacity = center_state.get_light_block().max(1);

    let mut level: i32 = 0;

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

        let calculated = neighbour_level - opacity;
        level = calculated.max(level);
        if level > expect as i32 {
            return level as u8;
        }
    }

    level as u8
}

/// `propagateBlockChanges` — sky-версия.
pub(crate) fn propagate_block_changes(
    engine: &mut StarLightEngine,
    light_access: &dyn LightChunkGetter,
    at_chunk: &ChunkAccess,
    positions: &HashSet<BlockPos>,
) {
    rewrite_nibble_cache_for_skylight(engine);
    for v in engine.null_propagation_check_cache.iter_mut() {
        *v = false;
    }

    let chunk_pos = at_chunk.get_pos();
    let chunk_x = chunk_pos.x;
    let chunk_z = chunk_pos.z;
    let height_map_offset = chunk_x * -16 + (chunk_z * (-16 * 16));

    // setup heightmap for changes
    for pos in positions {
        let index = pos.get_x() + (pos.get_z() << 4) + height_map_offset;
        if index < 0 || index as usize >= engine.height_map_block_change.len() {
            continue;
        }
        let curr = engine.height_map_block_change[index as usize];
        if pos.get_y() > curr {
            engine.height_map_block_change[index as usize] = pos.get_y();
        }
    }

    // note: light sets are delayed while processing skylight source changes due to how
    // nibbles are initialised, as we want to avoid clobbering nibble values so what when
    // below nibbles are initialised they aren't reading from partially modified nibbles

    // now we can recalculate the sources for the changed columns
    for index in 0..(16 * 16) {
        let max_y = engine.height_map_block_change[index];
        if max_y == i32::MIN {
            // not changed
            continue;
        }
        engine.height_map_block_change[index] = i32::MIN; // restore default for next caller

        let column_x = ((index & 15) as i32) | (chunk_x << 4);
        let column_z = ((index >> 4) as i32) | (chunk_z << 4);

        // try and propagate from the above y
        // delay light set until after processing all sources to setup
        let max_propagation_y =
            try_propagate_skylight(engine, column_x, max_y, column_z, true, true);

        // maxPropagationY is now the highest block that could not be propagated to

        // remove all sources below that are 15
        let propagate_direction =
            AxisDirection::PositiveY.everything_but_this_direction();
        let encode_offset = engine.coordinate_offset;

        if get_light_level_extruded(engine, column_x, max_propagation_y, column_z) == 15 {
            // ensure section is checked
            check_null_section(
                engine,
                column_x >> 4,
                max_propagation_y >> 4,
                column_z >> 4,
                true,
            );

            let mut curr_y = max_propagation_y;
            let min_block_y = engine.min_light_section << 4;
            while curr_y >= min_block_y {
                if (curr_y & 15) == 15 {
                    // ensure section is checked
                    check_null_section(engine, column_x >> 4, curr_y >> 4, column_z >> 4, true);
                }

                // ensure section below is always checked
                let nibble =
                    engine.get_nibble_from_cache(column_x >> 4, curr_y >> 4, column_z >> 4);
                match nibble {
                    None => {
                        // advance currY to the the top of the section below
                        curr_y &= !15;
                        // note: this value ^ is actually 1 above the top, but the loop
                        // decrements by 1 so we actually end up there
                    }
                    Some(n) => {
                        if n.lock().unwrap().get_updating_xyz(column_x, curr_y, column_z) != 15 {
                            break;
                        }
                        // delay light set until after processing all sources to setup
                        engine.append_to_decrease_queue(
                            engine.encode_position(column_x, curr_y, column_z)
                                | (15i64 << super::star_light_engine::LEVEL_SHIFT)
                                | (propagate_direction << super::star_light_engine::DIR_SHIFT),
                            // do not set transparent blocks for the same reason we don't
                            // in the checkBlock method
                        );
                    }
                }
                curr_y -= 1;
            }
        }
        let _ = encode_offset;
    }

    // delayed light sets are processed here, and must be processed before checkBlock as
    // checkBlock reads immediate light value
    process_delayed_increases(engine);
    process_delayed_decreases(engine);

    for pos in positions {
        engine.check_block(light_access, pos.get_x(), pos.get_y(), pos.get_z());
    }

    engine.perform_light_decrease(light_access);
}

/// `lightChunk(lightAccess, chunk, needsEdgeChecks)` — sky-версия.
pub(crate) fn light_chunk(
    engine: &mut StarLightEngine,
    light_access: &dyn LightChunkGetter,
    chunk: &ChunkAccess,
    needs_edge_checks: bool,
) {
    rewrite_nibble_cache_for_skylight(engine);
    for v in engine.null_propagation_check_cache.iter_mut() {
        *v = false;
    }

    let chunk_pos = chunk.get_pos();
    let chunk_x = chunk_pos.x;
    let chunk_z = chunk_pos.z;

    let mut highest_non_empty_section = engine.max_section;
    loop {
        let section_is_empty = highest_non_empty_section == engine.min_section - 1
            || chunk
                .sections
                .get((highest_non_empty_section - engine.min_section) as usize)
                .map_or(true, |s| s.has_only_air());
        if !section_is_empty {
            break;
        }
        check_null_section(engine, chunk_x, highest_non_empty_section, chunk_z, false);
        // try propagate FULL to neighbours

        // check neighbours to see if we need to propagate into them
        for direction in AxisDirection::ONLY_HORIZONTAL {
            let neighbour_x = chunk_x + direction.x();
            let neighbour_z = chunk_z + direction.z();
            let neighbour_nibble =
                engine.get_nibble_from_cache(neighbour_x, highest_non_empty_section, neighbour_z);
            let neighbour_nibble = match neighbour_nibble {
                Some(n) => n,
                // unloaded neighbour
                // most of the time we fall here
                None => continue,
            };
            let _ = neighbour_nibble;

            // it looks like we need to propagate into the neighbour
            let (inc_x, inc_z, start_x, start_z);
            if direction.x() != 0 {
                // x direction
                inc_x = 0;
                inc_z = 1;
                start_x = if direction.x() < 0 {
                    chunk_x << 4
                } else {
                    (chunk_x << 4) | 15
                };
                start_z = chunk_z << 4;
            } else {
                // z direction
                inc_x = 1;
                inc_z = 0;
                start_z = if direction.z() < 0 {
                    chunk_z << 4
                } else {
                    (chunk_z << 4) | 15
                };
                start_x = chunk_x << 4;
            }

            let encode_offset = engine.coordinate_offset;
            let propagate_direction = 1i64 << direction.idx();

            let mut curr_y = highest_non_empty_section << 4;
            let max_y = curr_y | 15;
            while curr_y <= max_y {
                let mut i = 0;
                let mut curr_x = start_x;
                let mut curr_z = start_z;
                while i < 16 {
                    engine.append_to_increase_queue(
                        engine.encode_position(curr_x, curr_y, curr_z)
                            | (15i64 << super::star_light_engine::LEVEL_SHIFT) // we know we're at full lit here
                            | (propagate_direction << super::star_light_engine::DIR_SHIFT),
                        // no transparent flag, we know for a fact there are no blocks here
                        // that could be directionally transparent (as the section is EMPTY)
                    );
                    i += 1;
                    curr_x += inc_x;
                    curr_z += inc_z;
                }
                curr_y += 1;
            }
            let _ = encode_offset;
        }

        if highest_non_empty_section == engine.min_section - 1 {
            highest_non_empty_section -= 1;
            break;
        }
        highest_non_empty_section -= 1;
    }

    if highest_non_empty_section >= engine.min_section {
        // fill out our other sources
        let min_x = chunk_x << 4;
        let max_x = (chunk_x << 4) | 15;
        let min_z = chunk_z << 4;
        let max_z = (chunk_z << 4) | 15;
        let start_y = (highest_non_empty_section << 4) | 15;
        let mut curr_z = min_z;
        while curr_z <= max_z {
            let mut curr_x = min_x;
            while curr_x <= max_x {
                try_propagate_skylight(engine, curr_x, start_y + 1, curr_z, false, false);
                curr_x += 1;
            }
            curr_z += 1;
        }
    } // else: apparently the chunk is empty

    if needs_edge_checks {
        // not required to propagate here, but this will reduce the hit of the edge checks
        engine.perform_light_increase(light_access);

        let mut y = highest_non_empty_section;
        while y >= engine.min_light_section {
            check_null_section(engine, chunk_x, y, chunk_z, false);
            y -= 1;
        }
        // no need to rewrite the nibble cache again
        engine.check_chunk_edges_range_base(
            light_access,
            chunk,
            engine.min_light_section,
            highest_non_empty_section,
        );
    } else {
        let mut y = highest_non_empty_section;
        while y >= engine.min_light_section {
            check_null_section(engine, chunk_x, y, chunk_z, false);
            y -= 1;
        }
        engine.propagate_neighbour_levels(
            light_access,
            chunk,
            engine.min_light_section,
            highest_non_empty_section,
        );

        engine.perform_light_increase(light_access);
    }
}

/// `processDelayedIncreases` — применяет отложенные increase-записи.
pub(crate) fn process_delayed_increases(engine: &mut StarLightEngine) {
    // copied from performLightIncrease
    let decode_offset_x = -engine.encode_offset_x;
    let decode_offset_y = -engine.encode_offset_y;
    let decode_offset_z = -engine.encode_offset_z;

    for i in 0..engine.increase_queue_len {
        let queue_value = engine.increase_queue[i];

        let pos_x = ((queue_value & 63) as i32) + decode_offset_x;
        let pos_z = (((queue_value >> 6) & 63) as i32) + decode_offset_z;
        let pos_y = (((queue_value >> 12) & ((1i64 << 16) - 1)) as i32) + decode_offset_y;
        let propagated_light_level =
            ((queue_value >> super::star_light_engine::LEVEL_SHIFT) & 0xF) as u8;

        engine.set_light_level(pos_x, pos_y, pos_z, propagated_light_level);
    }
}

/// `processDelayedDecreases` — применяет отложенные decrease-записи.
pub(crate) fn process_delayed_decreases(engine: &mut StarLightEngine) {
    // copied from performLightDecrease
    let decode_offset_x = -engine.encode_offset_x;
    let decode_offset_y = -engine.encode_offset_y;
    let decode_offset_z = -engine.encode_offset_z;

    for i in 0..engine.decrease_queue_len {
        let queue_value = engine.decrease_queue[i];

        let pos_x = ((queue_value & 63) as i32) + decode_offset_x;
        let pos_z = (((queue_value >> 6) & 63) as i32) + decode_offset_z;
        let pos_y = (((queue_value >> 12) & ((1i64 << 16) - 1)) as i32) + decode_offset_y;

        engine.set_light_level(pos_x, pos_y, pos_z, 0);
    }
}

/// `tryPropagateSkylight` — вертикальное распространение 15 света вниз
/// по колонке; возвращает Y, до которого удалось распространить.
#[allow(clippy::too_many_arguments)]
pub(crate) fn try_propagate_skylight(
    engine: &mut StarLightEngine,
    world_x: i32,
    mut start_y: i32,
    world_z: i32,
    extrude_initialised: bool,
    delay_light_set: bool,
) -> i32 {
    let propagate_direction = AxisDirection::PositiveY.everything_but_this_direction();

    if get_light_level_extruded(engine, world_x, start_y + 1, world_z) != 15 {
        return start_y;
    }

    // ensure this section is always checked
    check_null_section(engine, world_x >> 4, start_y >> 4, world_z >> 4, extrude_initialised);

    let mut above = engine.get_block_state(world_x, start_y + 1, world_z);

    let min_block_y = engine.min_light_section << 4;
    while start_y >= min_block_y {
        if (start_y & 15) == 15 {
            // ensure this section is always checked
            check_null_section(engine, world_x >> 4, start_y >> 4, world_z >> 4, extrude_initialised);
        }
        let current = engine.get_block_state(world_x, start_y, world_z);

        let from_shape;
        if above.starlight_is_conditionally_full_opaque() {
            from_shape = above.face_occlusion_shape_unleveled(AxisDirection::NegativeY.nms());
            if crate::world::phys::shapes::face_shape_occludes(&VoxelShape::empty(), &from_shape) {
                // above wont let us propagate
                break;
            }
        } else {
            from_shape = VoxelShape::empty();
        }

        // does light propagate from the top down?
        let mut flags = 0i64;
        if current.starlight_is_conditionally_full_opaque() {
            let culling_face = current.face_occlusion_shape_unleveled(AxisDirection::PositiveY.nms());
            if crate::world::phys::shapes::face_shape_occludes(&from_shape, &culling_face) {
                // can't propagate here, we're done on this column.
                break;
            }
            flags |= FLAG_HAS_SIDED_TRANSPARENT_BLOCKS;
        }

        let opacity = current.get_light_block();
        if opacity > 0 {
            // let the queued value (if any) handle it from here.
            break;
        }

        // light set delayed until we determine if this nibble section is null
        engine.append_to_increase_queue(
            engine.encode_position(world_x, start_y, world_z)
                | (15i64 << super::star_light_engine::LEVEL_SHIFT) // we know we're at full lit here
                | (propagate_direction << super::star_light_engine::DIR_SHIFT)
                | flags,
        );

        above = current;

        if engine
            .get_nibble_from_cache(world_x >> 4, start_y >> 4, world_z >> 4)
            .is_none()
        {
            // we skip empty sections here, as this is just an easy way of making sure the above block
            // can propagate through air.

            // nothing can propagate in null sections, remove the queue entry for it
            engine.increase_queue_len -= 1;

            // advance currY to the the top of the section below
            start_y &= !15;
            // note: this value ^ is actually 1 above the top, but the loop decrements by 1 so we actually
            // end up there

            // make sure this is marked as AIR
            above = BlockState::air();
        } else if !delay_light_set {
            engine.set_light_level(world_x, start_y, world_z, 15);
        }

        start_y -= 1;
    }

    start_y
}
