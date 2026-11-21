//! Восстановление палитр секций, высот и света чанка из NBT Anvil.

use std::collections::BTreeMap;
use std::io;
use std::sync::{Arc, Mutex};

use crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::{
    NibbleArrayRef, SWMRNibbleArray, ARRAY_SIZE, INIT_STATE_HIDDEN, INIT_STATE_INIT,
};
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::registry_access::RegistryAccess;
use crate::core::shared_constants::CURRENT_DATA_VERSION;
use crate::nbt::{CompoundTag, Tag};
use crate::resources::Identifier;
use crate::util::bit_storage::SimpleBitStorage;
use crate::util::datafix::{DataFixTypes, DataFixer};
use crate::world::level::biome::biome::Biome;
use crate::world::level::block::state::property::StateValue;
use crate::world::level::block::state::BlockState;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::level_chunk::{ChunkStatus, LevelChunk};
use crate::world::level::chunk::level_chunk_section::{
    create_biomes_container, create_block_states_container, LevelChunkSection,
};
use crate::world::level::chunk::packed_ticks::PackedTicks;
use crate::world::level::chunk::palette::ceillog2;
use crate::world::level::chunk::paletted_container::PalettedContainer;
use crate::world::level::chunk::proto_chunk::ProtoChunk;
use crate::world::level::chunk::RuntimeChunk;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_height_accessor::{LevelHeightAccessor, SimpleLevelHeightAccessor};
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::{Heightmap, HeightmapType};

use super::region_file_storage::RegionFileStorage;

pub trait ChunkPaletteLookup {
    fn block_state(&self, nbt: &CompoundTag) -> Option<BlockState>;
    fn biome(&self, id: &Identifier) -> Option<Holder<Biome>>;
    fn default_biome(&self) -> Holder<Biome>;

    fn create_section(&self) -> LevelChunkSection {
        LevelChunkSection::new(
            create_block_states_container(BlockState::air()),
            create_biomes_container(self.default_biome()),
        )
    }
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
    pub packed_ticks: PackedTicks,
    pub post_processing: Vec<BlockPos>,
    /// Содержит сохранённые сущности и данные структур для следующих стадий загрузки.
    pub original_tag: CompoundTag,
}

pub struct LoadedChunk {
    pub chunk: RuntimeChunk,
    pub last_update_time: i64,
    pub original_tag: CompoundTag,
}

const STARLIGHT_VERSION: i32 = 10;
const STARLIGHT_VERSION_KEY: &str = "starlight.light_version";
const BLOCK_LIGHT_STATE: &str = "starlight.blocklight_state";
const SKY_LIGHT_STATE: &str = "starlight.skylight_state";

impl LoadedChunk {
    pub fn to_nbt(
        &self,
        game_time: i64,
        registries: &RegistryAccess,
        has_sky_light: bool,
    ) -> io::Result<CompoundTag> {
        if crate::nbt::utils::get_data_version(&self.original_tag) != CURRENT_DATA_VERSION {
            return Err(invalid_data("chunk requires a data upgrade before saving"));
        }

        let access = self.chunk.access();
        let pos = access.get_pos();
        let mut tag = self.original_tag.clone();
        tag.put_int("xPos", pos.x);
        tag.put_int("yPos", access.get_min_section());
        tag.put_int("zPos", pos.z);
        tag.put_long("LastUpdate", game_time);
        tag.put_long("InhabitedTime", access.inhabited_time);
        tag.put_string(
            "Status",
            &format!("minecraft:{}", access.persisted_status.path()),
        );
        let light_correct =
            access.is_light_correct && access.persisted_status.is_or_after(ChunkStatus::Light);
        if light_correct {
            // Paper помечает свет как StarLight и заставляет vanilla пересчитать его.
            tag.put_bool("isLightOn", false);
            tag.put_int(STARLIGHT_VERSION_KEY, STARLIGHT_VERSION);
        } else {
            tag.remove("isLightOn");
            tag.remove(STARLIGHT_VERSION_KEY);
        }
        let sections = write_sections(access, &tag, registries, light_correct, has_sky_light)?;
        tag.put("sections", sections);
        write_heightmaps(access, &mut tag)?;

        let mut block_entities = access.pending_block_entities.clone();
        for (pos, entity) in &access.block_entities {
            block_entities.insert(*pos, entity.save_full_metadata(registries));
        }
        let mut block_entities: Vec<_> = block_entities.into_iter().collect();
        block_entities.sort_unstable_by_key(|(pos, _)| pos.as_long());
        tag.put(
            "block_entities",
            Tag::List(
                block_entities
                    .into_iter()
                    .map(|(_, tag)| Tag::Compound(tag))
                    .collect(),
            ),
        );

        let (ticks, post_processing) = match &self.chunk {
            RuntimeChunk::Full(chunk) => (
                chunk.pack_ticks(game_time),
                chunk.post_processing.as_slice(),
            ),
            RuntimeChunk::Proto(chunk) => {
                let entities = chunk.entities.iter().cloned().map(Tag::Compound).collect();
                tag.put("entities", Tag::List(entities));
                if let Some(mask) = &chunk.carving_mask {
                    tag.put_long_array("carving_mask", &pack_carving_mask(mask));
                } else {
                    tag.remove("carving_mask");
                }
                (chunk.pack_ticks(), chunk.post_processing.as_slice())
            }
        };
        ticks.write_nbt(&mut tag);
        tag.put(
            "PostProcessing",
            write_post_processing(access, post_processing),
        );
        Ok(tag)
    }
}

impl SerializableChunkData {
    pub fn into_runtime(self, level: Option<Arc<dyn LevelReader>>) -> LoadedChunk {
        let chunk = if self.chunk.persisted_status == ChunkStatus::Full {
            RuntimeChunk::Full(LevelChunk::from_access(
                self.chunk,
                self.packed_ticks,
                self.post_processing,
                level,
            ))
        } else {
            let entities = self
                .original_tag
                .get_list_or_empty("entities")
                .iter()
                .filter_map(Tag::as_compound)
                .cloned()
                .collect();
            let carving_mask = self
                .original_tag
                .get_long_array("carving_mask")
                .map(unpack_carving_mask);
            let mut chunk = ProtoChunk::from_access(
                self.chunk,
                self.packed_ticks,
                self.post_processing,
                entities,
            );
            chunk.carving_mask = carving_mask;
            RuntimeChunk::Proto(chunk)
        };
        LoadedChunk {
            chunk,
            last_update_time: self.last_update_time,
            original_tag: self.original_tag,
        }
    }

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
                tag.put_int("DataVersion", CURRENT_DATA_VERSION);
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
            .map(|_| context.palettes.create_section())
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
            && tag.get_int(STARLIGHT_VERSION_KEY) == Some(STARLIGHT_VERSION);

        if chunk.is_light_correct {
            let block_light = chunk.starlight_block_nibbles.get_mut().unwrap();
            let sky_light = chunk.starlight_sky_nibbles.get_mut().unwrap();
            for section in encoded_sections.iter().filter_map(Tag::as_compound) {
                let y = i32::from(section.get_byte("Y").unwrap());
                let index = (y - min_light_section) as usize;
                if let Some(block) = block_light.get_mut(index) {
                    *block = Some(read_light(section, "BlockLight", BLOCK_LIGHT_STATE)?);
                    if context.has_sky_light {
                        sky_light[index] = Some(read_light(section, "SkyLight", SKY_LIGHT_STATE)?);
                    }
                }
            }
        }

        read_heightmaps(&mut chunk, tag.get_compound("Heightmaps"));
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
            packed_ticks: PackedTicks::from_nbt(&tag, pos),
            post_processing: read_post_processing(&tag, pos, height),
            original_tag: tag,
        })
    }
}

fn write_sections(
    access: &ChunkAccess,
    original: &CompoundTag,
    registries: &RegistryAccess,
    light_correct: bool,
    has_sky_light: bool,
) -> io::Result<Tag> {
    let mut sections: BTreeMap<_, _> = original
        .get_list_or_empty("sections")
        .iter()
        .filter_map(Tag::as_compound)
        .filter_map(|section| Some((i32::from(section.get_byte("Y")?), section.clone())))
        .collect();
    for section in sections.values_mut() {
        for key in [
            "Y",
            "BlockLight",
            "SkyLight",
            BLOCK_LIGHT_STATE,
            SKY_LIGHT_STATE,
        ] {
            section.remove(key);
        }
    }

    let biome_registry = registries.lookup::<Biome>(&crate::core::registry::registries::biome());
    let min_section = access.get_min_section();
    for (index, section) in access.sections.iter().enumerate() {
        let tag = sections.entry(min_section + index as i32).or_default();
        tag.put(
            "block_states",
            write_palette(&section.states, |state| {
                Ok(Tag::Compound(write_block_state(state)))
            })?,
        );
        tag.put(
            "biomes",
            write_palette(&section.biomes, |biome| {
                let key = match &biome {
                    Holder::Key(key) => Some(key),
                    Holder::Reference(id) => biome_registry
                        .as_ref()
                        .and_then(|registry| registry.get_key_by_id(*id)),
                    Holder::Direct(value) => biome_registry
                        .as_ref()
                        .and_then(|registry| registry.get_key(value)),
                }
                .ok_or_else(|| invalid_data("chunk biome lacks a registry key"))?;
                Ok(Tag::String(key.id().to_string()))
            })?,
        );
    }

    if light_correct {
        let block = access.starlight_block_nibbles.lock().unwrap();
        write_light_sections(
            &mut sections,
            min_section - 1,
            &block,
            "BlockLight",
            BLOCK_LIGHT_STATE,
        );
        drop(block);
        if has_sky_light {
            let sky = access.starlight_sky_nibbles.lock().unwrap();
            write_light_sections(
                &mut sections,
                min_section - 1,
                &sky,
                "SkyLight",
                SKY_LIGHT_STATE,
            );
        }
    }

    let sections = sections
        .into_iter()
        .filter(|(_, tag)| !tag.is_empty())
        .map(|(y, mut tag)| {
            let y = i8::try_from(y)
                .map_err(|_| invalid_data("chunk section Y exceeds Anvil byte range"))?;
            tag.put_byte("Y", y);
            Ok(Tag::Compound(tag))
        })
        .collect::<io::Result<Vec<_>>>()?;
    Ok(Tag::List(sections))
}

fn write_light_sections(
    sections: &mut BTreeMap<i32, CompoundTag>,
    min_light_section: i32,
    nibbles: &[Option<NibbleArrayRef>],
    data_key: &str,
    state_key: &str,
) {
    for (index, entry) in nibbles.iter().enumerate() {
        let Some(nibble) = entry else { continue };
        let Some(state) = nibble.lock().unwrap().get_save_state() else {
            continue;
        };
        let tag = sections
            .entry(min_light_section + index as i32)
            .or_default();
        tag.put_int(state_key, i32::from(state.state));
        if let Some(data) = state.data {
            tag.put(
                data_key,
                Tag::ByteArray(data.iter().map(|byte| *byte as i8).collect()),
            );
        }
    }
}

fn write_palette<T: Clone + Eq + std::hash::Hash + 'static>(
    container: &PalettedContainer<T>,
    encode: impl Fn(T) -> io::Result<Tag>,
) -> io::Result<Tag> {
    let (values, words) = container.pack();
    let mut tag = CompoundTag::new();
    let palette = values
        .into_iter()
        .map(encode)
        .collect::<io::Result<Vec<_>>>()?;
    tag.put("palette", Tag::List(palette));
    if !words.is_empty() {
        tag.put("data", Tag::LongArray(words));
    }
    Ok(Tag::Compound(tag))
}

fn write_block_state(state: BlockState) -> CompoundTag {
    let mut tag = CompoundTag::new();
    tag.put_string("Name", &state.block.id.to_string());
    if !state.values.is_empty() {
        let mut properties = CompoundTag::new();
        for (name, value) in state.values.entries() {
            let value = match value {
                StateValue::Int(value) => value.to_string(),
                StateValue::Bool(value) => value.to_string(),
                StateValue::Str(value) => value.clone(),
            };
            properties.put_string(name, &value);
        }
        tag.put("Properties", Tag::Compound(properties));
    }
    tag
}

fn read_heightmaps(chunk: &mut ChunkAccess, tag: Option<&CompoundTag>) {
    let height = chunk.level_height_accessor.as_ref();
    let bits = ceillog2(height.get_height() as usize + 1) as i32;
    if let Some(tag) = tag {
        for ty in HeightmapType::VALUES {
            let Some(words) = tag.get_long_array(heightmap_key(ty)) else {
                continue;
            };
            let Ok(data) = SimpleBitStorage::from_raw(bits, 256, words.to_vec()) else {
                continue;
            };
            if (0..256).any(|index| data.get(index) > height.get_height()) {
                continue;
            }

            let min_y = height.get_min_y();
            let mut heightmap = Heightmap::new(chunk.get_pos(), ty, height, min_y);
            for z in 0..16 {
                for x in 0..16 {
                    heightmap.set_height(x, z, min_y + data.get((z * 16 + x) as usize));
                }
            }
            chunk.heightmaps.insert(ty, heightmap);
        }
    }

    let missing: Vec<_> = chunk
        .persisted_status
        .heightmaps_after()
        .iter()
        .copied()
        .filter(|ty| !chunk.heightmaps.contains_key(ty))
        .collect();
    chunk.prime_heightmaps(&missing);
}

fn write_heightmaps(access: &ChunkAccess, tag: &mut CompoundTag) -> io::Result<()> {
    let mut heightmaps = tag.get_compound("Heightmaps").cloned().unwrap_or_default();
    let height = access.level_height_accessor.get_height();
    let min_y = access.level_height_accessor.get_min_y();
    let bits = ceillog2(height as usize + 1) as i32;
    for ty in HeightmapType::VALUES {
        let key = heightmap_key(ty);
        let Some(heightmap) = access.heightmaps.get(&ty) else {
            heightmaps.remove(key);
            continue;
        };
        let mut storage = SimpleBitStorage::new(bits, 256);
        for z in 0..16 {
            for x in 0..16 {
                let relative_y = heightmap.get_first_available(x, z).checked_sub(min_y);
                let Some(relative_y) = relative_y.filter(|y| (0..=height).contains(y)) else {
                    return Err(invalid_data(format!(
                        "heightmap {key} exceeds build height"
                    )));
                };
                storage.set((z * 16 + x) as usize, relative_y);
            }
        }
        heightmaps.put_long_array(
            key,
            crate::util::bit_storage::BitStorage::Simple(storage).get_raw(),
        );
    }
    tag.put("Heightmaps", Tag::Compound(heightmaps));
    Ok(())
}

fn write_post_processing(access: &ChunkAccess, positions: &[BlockPos]) -> Tag {
    let mut sections = vec![Vec::new(); access.sections.len()];
    let pos = access.get_pos();
    for block in positions {
        let (x, y, z) = (block.get_x(), block.get_y(), block.get_z());
        if (x >> 4, z >> 4) == (pos.x, pos.z) {
            let index = (y >> 4) - access.get_min_section();
            if let Some(section) = sections.get_mut(index as usize) {
                section.push(Tag::Short(
                    ((x & 15) | ((y & 15) << 4) | ((z & 15) << 8)) as i16,
                ));
            }
        }
    }
    Tag::List(sections.into_iter().map(Tag::List).collect())
}

fn unpack_carving_mask(words: &[i64]) -> Vec<bool> {
    words
        .iter()
        .flat_map(|word| (0..64).map(move |bit| word & (1i64 << bit) != 0))
        .collect()
}

fn pack_carving_mask(mask: &[bool]) -> Vec<i64> {
    let Some(last) = mask.iter().rposition(|set| *set) else {
        return Vec::new();
    };
    mask[..=last]
        .chunks(64)
        .map(|bits| {
            bits.iter()
                .enumerate()
                .fold(0, |word, (bit, set)| word | (i64::from(*set) << bit))
        })
        .collect()
}

fn read_post_processing(
    tag: &CompoundTag,
    chunk: ChunkPos,
    height: &dyn LevelHeightAccessor,
) -> Vec<BlockPos> {
    let sections = tag.get_list_or_empty("PostProcessing");
    let mut positions = Vec::new();
    for (index, section) in sections
        .iter()
        .take(height.get_sections_count() as usize)
        .enumerate()
    {
        let Tag::List(offsets) = section else {
            continue;
        };
        let min_y = (height.get_min_section_y() + index as i32) << 4;
        positions.extend(offsets.iter().filter_map(Tag::as_short).map(|offset| {
            let offset = i32::from(offset);
            BlockPos::new(
                chunk.get_min_block_x() + (offset & 15),
                min_y + ((offset >> 4) & 15),
                chunk.get_min_block_z() + ((offset >> 8) & 15),
            )
        }));
    }
    positions
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
    fn loaded_chunk_registers_and_repacks_block_and_fluid_ticks() {
        use crate::world::level::scheduled_tick_access::{BlockTickType, FluidTickType};
        use crate::world::ticks::{LevelTicks, SavedTick, TickAccess, TickPriority};

        let pos = ChunkPos::new(-1, 2);
        let height = SimpleLevelHeightAccessor::create(-64, 32);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &TestPalettes,
            data_fixer: None,
            has_sky_light: true,
        };
        let block_pos = BlockPos::new(-15, -47, 34);
        let fluid_pos = BlockPos::new(-14, -48, 34);
        let tick = |id: &str, pos, delay, priority| {
            SavedTick::new(BlockTickType { id: id.into() }, pos, delay, priority)
        };
        let packed = PackedTicks {
            blocks: vec![
                tick("observer", block_pos, 5, TickPriority::Normal),
                tick("minecraft:repeater", block_pos, 5, TickPriority::High),
                tick("minecraft:stone", BlockPos::ZERO, 0, TickPriority::High),
            ],
            fluids: vec![SavedTick::new(
                FluidTickType {
                    id: "minecraft:water".into(),
                },
                fluid_pos,
                -2,
                TickPriority::Low,
            )],
        };
        let mut tag = CompoundTag::new();
        crate::nbt::utils::add_current_data_version(&mut tag);
        tag.put_int("xPos", pos.x);
        tag.put_int("zPos", pos.z);
        tag.put_long("LastUpdate", 15);
        tag.put_string("extra", "kept for the remaining chunk loaders");
        tag.put_string("Status", "minecraft:full");
        tag.put(
            "PostProcessing",
            Tag::List(vec![
                Tag::List(Vec::new()),
                Tag::List(vec![Tag::Short(0x211)]),
            ]),
        );
        packed.write_nbt(&mut tag);

        let loaded = SerializableChunkData::parse(tag.clone(), pos, &context)
            .unwrap()
            .into_runtime(None);
        assert_eq!(loaded.last_update_time, 15);
        assert_eq!(
            loaded.original_tag.get_string("extra"),
            Some("kept for the remaining chunk loaders")
        );
        let RuntimeChunk::Full(chunk) = &loaded.chunk else {
            panic!("expected LevelChunk")
        };
        assert_eq!(chunk.get_post_processing(), &[block_pos]);
        assert_eq!(chunk.get_block_ticks().count(), 2);

        let blocks = LevelTicks::new(Arc::new(|_| true));
        let fluids = LevelTicks::new(Arc::new(|_| true));
        chunk.register_tick_containers(&blocks, &fluids, 100);
        let mut block_ids = Vec::new();
        let mut fluid_ids = Vec::new();
        fluids.tick(100, 8, &mut |_, kind| fluid_ids.push(kind.id));
        blocks.tick(104, 8, &mut |_, kind| block_ids.push(kind.id));
        assert_eq!(fluid_ids, ["minecraft:water"]);
        assert!(block_ids.is_empty());
        let saved_again = loaded.to_nbt(104, &RegistryAccess::new(), true).unwrap();
        assert_eq!(saved_again.get_long("LastUpdate"), Some(104));
        assert_eq!(saved_again.get("PostProcessing"), tag.get("PostProcessing"));
        let saved_again = PackedTicks::from_nbt(&saved_again, pos);
        assert!(saved_again.fluids.is_empty());
        assert_eq!(
            saved_again
                .blocks
                .iter()
                .map(|tick| tick.delay)
                .collect::<Vec<_>>(),
            [1, 1]
        );

        blocks.tick(105, 8, &mut |_, kind| block_ids.push(kind.id));
        assert_eq!(block_ids, ["minecraft:repeater", "minecraft:observer"]);
        let replacement = blocks.add_container(pos, None);
        chunk.unregister_tick_containers(&blocks, &fluids);
        assert!(Arc::ptr_eq(
            &blocks.get_container(pos).unwrap(),
            &replacement
        ));
        assert!(fluids.get_container(pos).is_none());

        tag.put_string("Status", "minecraft:features");
        tag.put_long_array("carving_mask", &[i64::MIN, 2]);
        let loaded = SerializableChunkData::parse(tag, pos, &context)
            .unwrap()
            .into_runtime(None);
        let RuntimeChunk::Proto(chunk) = &loaded.chunk else {
            panic!("expected ProtoChunk")
        };
        assert_eq!(chunk.chunk_status, ChunkStatus::Features);
        assert_eq!(chunk.post_processing, [block_pos]);
        assert_eq!(chunk.pack_ticks().fluids[0].delay, -2);
        let bits = chunk.carving_mask.as_ref().unwrap();
        assert!(bits[63]);
        assert!(bits[65]);
        let saved = loaded.to_nbt(120, &RegistryAccess::new(), true).unwrap();
        assert_eq!(
            saved.get_long_array("carving_mask"),
            Some([i64::MIN, 2].as_slice())
        );
        assert_eq!(saved.get_string("Status"), Some("minecraft:features"));
    }

    #[test]
    fn rebuilds_missing_and_invalid_heightmaps_and_tracks_column_changes() {
        let pos = ChunkPos::new(-1, 2);
        let height = SimpleLevelHeightAccessor::create(-64, 32);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &TestPalettes,
            data_fixer: None,
            has_sky_light: false,
        };
        let mut section = CompoundTag::new();
        section.put_byte("Y", -4);
        let mut indices = SimpleBitStorage::new(4, 4096);
        for y in [0, 5] {
            indices.set(y * 256, 1);
        }
        let mut states = CompoundTag::new();
        states.put(
            "palette",
            Tag::List(
                ["minecraft:air", "minecraft:stone"]
                    .into_iter()
                    .map(|id| {
                        let mut state = CompoundTag::new();
                        state.put_string("Name", id);
                        Tag::Compound(state)
                    })
                    .collect(),
            ),
        );
        states.put_long_array(
            "data",
            crate::util::bit_storage::BitStorage::Simple(indices).get_raw(),
        );
        section.put("block_states", Tag::Compound(states));
        let mut maps = CompoundTag::new();
        maps.put_long_array("WORLD_SURFACE", &[0]);
        let mut bad_indices = SimpleBitStorage::new(6, 256);
        bad_indices.set(0, 63);
        maps.put_long_array(
            "MOTION_BLOCKING",
            crate::util::bit_storage::BitStorage::Simple(bad_indices).get_raw(),
        );

        let mut tag = CompoundTag::new();
        crate::nbt::utils::add_current_data_version(&mut tag);
        tag.put_int("xPos", pos.x);
        tag.put_int("zPos", pos.z);
        tag.put_string("Status", "minecraft:full");
        tag.put("sections", Tag::List(vec![Tag::Compound(section)]));
        tag.put("Heightmaps", Tag::Compound(maps));

        let mut chunk = SerializableChunkData::parse(tag, pos, &context)
            .unwrap()
            .chunk;
        let x = pos.get_min_block_x();
        let z = pos.get_min_block_z();
        assert_eq!(chunk.heightmaps.len(), 4);
        for &ty in ChunkStatus::Full.heightmaps_after() {
            assert_eq!(chunk.get_height(ty, x, z), -58);
            assert_eq!(chunk.get_height(ty, x + 1, z), -64);
        }

        let mut leaves = Block::new(Identifier::minecraft("oak_leaves"))
            .with_kind(crate::world::level::block::BlockKind::Leaves);
        leaves.properties.has_collision = true;
        let leaves = BlockState::without_properties(Arc::new(leaves));
        assert!(chunk.set_block_state(&BlockPos::new(x, -35, z), leaves));
        assert!(chunk.is_unsaved());
        assert_eq!(chunk.get_height(HeightmapType::WorldSurface, x, z), -34);
        assert_eq!(
            chunk.get_height(HeightmapType::MotionBlockingNoLeaves, x, z),
            -58
        );
        let torch =
            BlockState::without_properties(Arc::new(Block::new(Identifier::minecraft("torch"))));
        let top = BlockPos::new(x, -35, z);
        assert!(chunk.set_block_state(&top, torch));
        assert_eq!(chunk.get_height(HeightmapType::WorldSurface, x, z), -34);
        assert_eq!(chunk.get_height(HeightmapType::OceanFloor, x, z), -58);

        for y in [-35, -59, -64] {
            assert!(chunk.set_block_state(&BlockPos::new(x, y, z), BlockState::air()));
        }
        for &ty in ChunkStatus::Full.heightmaps_after() {
            assert_eq!(chunk.get_height(ty, x, z), -64);
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
        section.put_string("fixture:section_data", "preserved");
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

        drop(nibbles);
        let mut loaded = data.into_runtime(None);
        let access = loaded.chunk.access_mut();
        let other_pos = BlockPos::new(-14, -64, 32);
        access.sections[0].set_block_state(2, 0, 0, block);
        access
            .heightmaps
            .get_mut(&HeightmapType::WorldSurface)
            .unwrap()
            .set_height(2, 0, -63);
        let biome = TestPalettes
            .biome(&Identifier::minecraft("forest"))
            .unwrap();
        access.sections[0].biomes.set(1, 0, 0, biome);
        let mut pending = CompoundTag::new();
        pending.put_int("x", -14);
        pending.put_int("y", -64);
        pending.put_int("z", 32);
        pending.put_string("id", "minecraft:chest");
        access.set_pending_block_entity(other_pos, pending);
        let saved = loaded.to_nbt(240, &RegistryAccess::new(), true).unwrap();
        let bytes = crate::nbt::io::write(&saved).unwrap();
        let reread = crate::nbt::io::read_bytes(
            &bytes,
            &mut crate::nbt::accounter::NbtAccounter::uncompressed_quota(),
        )
        .unwrap();
        let data = SerializableChunkData::parse(reread, pos, &context).unwrap();
        assert_eq!(data.last_update_time, 240);
        assert!(data
            .chunk
            .get_block_state(&other_pos)
            .is_block_id("minecraft:stone"));
        assert_eq!(
            data.chunk.heightmaps[&HeightmapType::WorldSurface].get_first_available(2, 0),
            -63
        );
        assert_eq!(
            data.chunk.sections[0]
                .biomes
                .get(1, 0, 0)
                .key()
                .unwrap()
                .id(),
            &Identifier::minecraft("forest")
        );
        assert!(data.chunk.pending_block_entities.contains_key(&other_pos));
        assert!(data.chunk.is_light_correct);
        let nibbles = data.chunk.starlight_block_nibbles.lock().unwrap();
        assert_eq!(
            nibbles[1].as_ref().unwrap().lock().unwrap().get_visible(0),
            7
        );
        let section = saved.get_list_or_empty("sections")[0]
            .as_compound()
            .unwrap();
        assert_eq!(
            section.get_string("fixture:section_data"),
            Some("preserved")
        );

        let error = SerializableChunkData::parse(tag, ChunkPos::new(0, 0), &context)
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    }
}
