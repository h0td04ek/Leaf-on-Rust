//! Восстановление палитр секций, высот и света чанка из NBT Anvil.

use std::io;
use std::sync::{Arc, Mutex};

use crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::{
    NibbleArrayRef, SWMRNibbleArray, ARRAY_SIZE, INIT_STATE_HIDDEN, INIT_STATE_INIT,
};
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::shared_constants::CURRENT_DATA_VERSION;
use crate::nbt::{CompoundTag, Tag};
use crate::resources::Identifier;
use crate::util::bit_storage::SimpleBitStorage;
use crate::util::datafix::{DataFixTypes, DataFixer};
use crate::world::level::biome::biome::Biome;
use crate::world::level::block::state::BlockState;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::level_chunk::ChunkStatus;
use crate::world::level::chunk::level_chunk_section::{
    create_biomes_container, create_block_states_container, LevelChunkSection,
};
use crate::world::level::chunk::palette::ceillog2;
use crate::world::level::chunk::paletted_container::PalettedContainer;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_height_accessor::{LevelHeightAccessor, SimpleLevelHeightAccessor};
use crate::world::level::levelgen::heightmap::{Heightmap, HeightmapType};

use super::region_file_storage::RegionFileStorage;

pub trait ChunkPaletteLookup {
    fn block_state(&self, nbt: &CompoundTag) -> Option<BlockState>;
    fn biome(&self, id: &Identifier) -> Option<Holder<Biome>>;
    fn default_biome(&self) -> Holder<Biome>;
}

pub struct ChunkLoadingContext<'a> {
    pub height: &'a dyn LevelHeightAccessor,
    pub palettes: &'a dyn ChunkPaletteLookup,
    pub data_fixer: Option<&'a dyn DataFixer>,
    pub has_sky_light: bool,
}

pub struct SerializableChunkData {
    pub chunk: ChunkAccess,
    pub last_update_time: i64,
    /// Tick queues, entities and structure payloads are passed through to their loaders.
    pub original_tag: CompoundTag,
}

impl SerializableChunkData {
    pub fn read(
        storage: &RegionFileStorage,
        pos: ChunkPos,
        context: &ChunkLoadingContext<'_>,
    ) -> io::Result<Option<Self>> {
        storage
            .read(pos)?
            .map(|tag| Self::parse(tag, pos, context))
            .transpose()
    }

    pub fn parse(
        mut tag: CompoundTag,
        expected_pos: ChunkPos,
        context: &ChunkLoadingContext<'_>,
    ) -> io::Result<Self> {
        let version = crate::nbt::utils::get_data_version(&tag);
        if version > CURRENT_DATA_VERSION {
            return Err(invalid_data(format!(
                "chunk data version {version} is newer than {CURRENT_DATA_VERSION}"
            )));
        }
        if version < CURRENT_DATA_VERSION {
            if let Some(fixer) = context.data_fixer {
                tag = DataFixTypes::Chunk.update_to_current_version(fixer, tag, version);
            }
        }

        let pos = ChunkPos::new(
            tag.get_int("xPos")
                .ok_or_else(|| invalid_data("chunk lacks xPos after upgrade"))?,
            tag.get_int("zPos")
                .ok_or_else(|| invalid_data("chunk lacks zPos after upgrade"))?,
        );
        if pos != expected_pos {
            return Err(invalid_data(format!(
                "expected chunk {expected_pos:?}, found {pos:?}"
            )));
        }

        let status = tag
            .get_string("Status")
            .and_then(ChunkStatus::by_name)
            .ok_or_else(|| invalid_data("missing or unrecognized chunk status"))?;
        let height = context.height;
        if height.get_height() <= 0 || height.get_height() % 16 != 0 || height.get_min_y() % 16 != 0
        {
            return Err(invalid_data(
                "chunk height and minimum Y must align to 16-block sections",
            ));
        }
        let section_count = height.get_sections_count() as usize;
        let mut sections: Vec<_> = (0..section_count)
            .map(|_| create_section(context.palettes))
            .collect();
        let min_section = height.get_min_section_y();
        let min_light_section = min_section - 1;
        let encoded_sections = tag.get_list_or_empty("sections");

        for section in encoded_sections {
            let section = section
                .as_compound()
                .ok_or_else(|| invalid_data("chunk section must be a compound"))?;
            let y = i32::from(
                section
                    .get_byte("Y")
                    .ok_or_else(|| invalid_data("chunk section lacks Y"))?,
            );
            if let Some(destination) = sections.get_mut((y - min_section) as usize) {
                read_section(destination, section, context.palettes)?;
            }
        }

        let world_height =
            SimpleLevelHeightAccessor::create(height.get_min_y(), height.get_height());
        let mut chunk = ChunkAccess::new(pos, Box::new(world_height), sections);
        chunk.inhabited_time = tag.get_long_or("InhabitedTime", 0);
        chunk.persisted_status = status;
        chunk.is_light_correct = status.is_or_after(ChunkStatus::Light)
            && tag.get("isLightOn").is_some()
            && tag.get_int("starlight.light_version") == Some(10);

        if chunk.is_light_correct {
            let block_light = chunk.starlight_block_nibbles.get_mut().unwrap();
            let sky_light = chunk.starlight_sky_nibbles.get_mut().unwrap();
            for section in encoded_sections.iter().filter_map(Tag::as_compound) {
                let y = i32::from(section.get_byte("Y").unwrap());
                let index = (y - min_light_section) as usize;
                if let Some(block) = block_light.get_mut(index) {
                    *block = Some(read_light(
                        section,
                        "BlockLight",
                        "starlight.blocklight_state",
                    )?);
                    if context.has_sky_light {
                        sky_light[index] =
                            Some(read_light(section, "SkyLight", "starlight.skylight_state")?);
                    }
                }
            }
        }

        if let Some(heightmaps) = tag.get_compound("Heightmaps") {
            for ty in HeightmapType::VALUES {
                if let Some(words) = heightmaps.get_long_array(heightmap_key(ty)) {
                    let bits = ceillog2(height.get_height() as usize + 1) as i32;
                    let data = SimpleBitStorage::from_raw(bits, 256, words.to_vec())
                        .map_err(invalid_data)?;
                    let mut heightmap = Heightmap::new(pos, ty, height, height.get_min_y());
                    for z in 0..16 {
                        for x in 0..16 {
                            let relative_y = data.get((z * 16 + x) as usize);
                            heightmap.set_height(x, z, height.get_min_y() + relative_y);
                        }
                    }
                    chunk.heightmaps.insert(ty, heightmap);
                }
            }
        }
        for block_entity in tag
            .get_list_or_empty("block_entities")
            .iter()
            .filter_map(Tag::as_compound)
        {
            let (Some(x), Some(y), Some(z)) = (
                block_entity.get_int("x"),
                block_entity.get_int("y"),
                block_entity.get_int("z"),
            ) else {
                continue;
            };
            if x >> 4 == pos.x && z >> 4 == pos.z && height.is_inside_build_height(y) {
                chunk.set_pending_block_entity(BlockPos::new(x, y, z), block_entity.clone());
            }
        }

        Ok(Self {
            chunk,
            last_update_time: tag.get_long_or("LastUpdate", 0),
            original_tag: tag,
        })
    }
}

fn create_section(palettes: &dyn ChunkPaletteLookup) -> LevelChunkSection {
    LevelChunkSection::new(
        create_block_states_container(BlockState::air()),
        create_biomes_container(palettes.default_biome()),
    )
}

fn read_section(
    destination: &mut LevelChunkSection,
    tag: &CompoundTag,
    palettes: &dyn ChunkPaletteLookup,
) -> io::Result<()> {
    if let Some(states) = tag.get_compound("block_states") {
        read_palette(&mut destination.states, states, |entry| {
            palettes.block_state(entry.as_compound()?)
        })?;
        let mut non_empty = 0;
        destination.states.count(|state, count| {
            if !state.is_air() || !state.get_fluid_state().is_empty() {
                non_empty += count;
            }
        });
        destination.non_empty_block_count = non_empty as i16;
    }
    if let Some(biomes) = tag.get_compound("biomes") {
        read_palette(&mut destination.biomes, biomes, |entry| {
            let id = Identifier::parse(entry.as_string()?).ok()?;
            palettes.biome(&id)
        })?;
    }
    Ok(())
}

fn read_palette<T: Clone + Eq + std::hash::Hash + 'static>(
    destination: &mut PalettedContainer<T>,
    tag: &CompoundTag,
    resolve: impl Fn(&Tag) -> Option<T>,
) -> io::Result<()> {
    let values = tag
        .get_list_or_empty("palette")
        .iter()
        .map(|value| resolve(value).ok_or_else(|| invalid_data("unknown saved palette value")))
        .collect::<io::Result<Vec<T>>>()?;
    destination
        .unpack(values, tag.get_long_array("data"))
        .map_err(invalid_data)
}

fn read_light(tag: &CompoundTag, array_key: &str, state_key: &str) -> io::Result<NibbleArrayRef> {
    let state = tag.get_int_or(state_key, 0);
    if !(0..=i32::from(INIT_STATE_HIDDEN)).contains(&state) {
        return Err(invalid_data("invalid StarLight section state"));
    }

    let bytes = match tag.get_byte_array(array_key) {
        Some(data) if data.len() == ARRAY_SIZE => {
            let mut bytes = Box::new([0; ARRAY_SIZE]);
            for (dest, source) in bytes.iter_mut().zip(data) {
                *dest = *source as u8;
            }
            Some(bytes)
        }
        Some(_) => return Err(invalid_data("invalid StarLight section byte count")),
        None => None,
    };
    if (state >= i32::from(INIT_STATE_INIT)) != bytes.is_some() {
        return Err(invalid_data(
            "StarLight section state and byte array disagree",
        ));
    }
    Ok(Arc::new(Mutex::new(SWMRNibbleArray::with_bytes_and_state(
        bytes,
        state as u8,
    ))))
}

fn heightmap_key(ty: HeightmapType) -> &'static str {
    match ty {
        HeightmapType::WorldSurfaceWg => "WORLD_SURFACE_WG",
        HeightmapType::WorldSurface => "WORLD_SURFACE",
        HeightmapType::OceanFloorWg => "OCEAN_FLOOR_WG",
        HeightmapType::OceanFloor => "OCEAN_FLOOR",
        HeightmapType::MotionBlocking => "MOTION_BLOCKING",
        HeightmapType::MotionBlockingNoLeaves => "MOTION_BLOCKING_NO_LEAVES",
    }
}

fn invalid_data(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::registry::registries;
    use crate::resources::ResourceKey;
    use crate::world::level::block::Block;

    struct TestPalettes;

    impl ChunkPaletteLookup for TestPalettes {
        fn block_state(&self, nbt: &CompoundTag) -> Option<BlockState> {
            match nbt.get_string("Name")? {
                "minecraft:air" => Some(BlockState::air()),
                "minecraft:stone" => {
                    let mut block = Block::new(Identifier::minecraft("stone"));
                    block.properties.has_collision = true;
                    Some(BlockState::without_properties(Arc::new(block)))
                }
                _ => None,
            }
        }

        fn biome(&self, id: &Identifier) -> Option<Holder<Biome>> {
            Some(Holder::Key(ResourceKey::create_with_registry_name(
                registries::biome(),
                id.clone(),
            )))
        }

        fn default_biome(&self) -> Holder<Biome> {
            self.biome(&Identifier::minecraft("plains")).unwrap()
        }
    }

    #[test]
    fn restores_sections_heightmaps_and_visible_starlight() {
        let pos = ChunkPos::new(-1, 2);
        let height = SimpleLevelHeightAccessor::create(-64, 32);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &TestPalettes,
            data_fixer: None,
            has_sky_light: true,
        };

        let mut state_indices = SimpleBitStorage::new(4, 4096);
        state_indices.set(1, 1);
        let state_tag = |id| {
            let mut tag = CompoundTag::new();
            tag.put_string("Name", id);
            Tag::Compound(tag)
        };
        let mut states = CompoundTag::new();
        states.put(
            "palette",
            Tag::List(vec![
                state_tag("minecraft:air"),
                state_tag("minecraft:stone"),
            ]),
        );
        states.put_long_array(
            "data",
            crate::util::bit_storage::BitStorage::Simple(state_indices).get_raw(),
        );

        let mut section = CompoundTag::new();
        section.put_byte("Y", -4);
        section.put("block_states", Tag::Compound(states));
        let mut light_data = [0; ARRAY_SIZE];
        light_data[0] = 7;
        section.put_byte_array("BlockLight", &light_data);
        section.put_int("starlight.blocklight_state", i32::from(INIT_STATE_INIT));

        let mut height_indices = SimpleBitStorage::new(6, 256);
        height_indices.set(1, 17);
        let mut heightmaps = CompoundTag::new();
        heightmaps.put_long_array(
            "WORLD_SURFACE",
            crate::util::bit_storage::BitStorage::Simple(height_indices).get_raw(),
        );

        let mut tag = CompoundTag::new();
        crate::nbt::utils::add_current_data_version(&mut tag);
        tag.put_int("xPos", pos.x);
        tag.put_int("zPos", pos.z);
        tag.put_string("Status", "minecraft:full");
        tag.put_bool("isLightOn", false);
        tag.put_int("starlight.light_version", 10);
        tag.put("sections", Tag::List(vec![Tag::Compound(section)]));
        tag.put("Heightmaps", Tag::Compound(heightmaps));

        let data = SerializableChunkData::parse(tag.clone(), pos, &context).unwrap();
        assert_eq!(data.chunk.persisted_status, ChunkStatus::Full);
        let block = data.chunk.get_block_state(&BlockPos::new(-15, -64, 32));
        assert!(block.is_block_id("minecraft:stone"));
        assert_eq!(data.chunk.sections[0].non_empty_block_count, 1);
        assert_eq!(
            data.chunk.heightmaps[&HeightmapType::WorldSurface].get_first_available(1, 0),
            -47
        );
        assert!(data.chunk.is_light_correct);
        let nibbles = data.chunk.starlight_block_nibbles.lock().unwrap();
        assert_eq!(nibbles.len(), 4);
        assert_eq!(
            nibbles[1].as_ref().unwrap().lock().unwrap().get_visible(0),
            7
        );

        let error = SerializableChunkData::parse(tag, ChunkPos::new(0, 0), &context)
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
