use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use leaf_core::core::{block_pos::BlockPos, shared_constants::CURRENT_DATA_VERSION};
use leaf_core::nbt::{CompoundTag, Tag};
use leaf_core::world::level::chunk::palette::{IdMap, Strategy, StrategyKind};
use leaf_core::world::level::chunk::storage::region_file_storage::RegionFileStorage;
use leaf_core::world::level::chunk::PalettedContainer;
use leaf_core::world::level::chunk_pos::ChunkPos;
use leaf_core::world::ticks::SavedTick;

use crate::blocks::Blocks;
use crate::wire::invalid;

use super::terrain::{Section, Terrain};
use super::{Changes, Chunk, Fluid, NativeWorld, WorldEdits, AIR, MIN_Y, SECTIONS};

const FORMAT_VERSION: i32 = 1;

#[derive(Default)]
pub(super) struct LoadedWorld {
    pub chunks: WorldEdits,
    pub pending_fluids: Vec<(Chunk, Vec<SavedTick<Fluid>>)>,
    pub legacy_notifications: Vec<BlockPos>,
}

pub(super) struct WorldStorage {
    regions: RegionFileStorage,
    blocks: Arc<StateIds>,
    terrain: Arc<Terrain>,
}

impl WorldStorage {
    pub fn new(path: PathBuf, terrain: Arc<Terrain>) -> Self {
        Self {
            regions: RegionFileStorage::new(path),
            blocks: Arc::new(StateIds),
            terrain,
        }
    }

    pub fn load(&self) -> io::Result<LoadedWorld> {
        let mut world = LoadedWorld::default();
        for pos in self.regions.chunk_positions()? {
            let Some(tag) = self.regions.read(pos)? else {
                continue;
            };
            let changes = self.decode(pos, &tag).map_err(|error| {
                io::Error::new(error.kind(), format!("native chunk {pos:?}: {error}"))
            })?;
            let coord = (pos.x, pos.z);
            match tag.get("fluid_ticks") {
                Some(Tag::List(entries)) => {
                    let mut ticks = Vec::new();
                    for entry in entries {
                        let tick = entry
                            .as_compound()
                            .and_then(SavedTick::<Fluid>::read_nbt)
                            .filter(|tick| {
                                super::chunk(tick.pos) == coord && NativeWorld::editable(tick.pos)
                            })
                            .ok_or_else(|| invalid("invalid fluid tick in native chunk"))?;
                        ticks.push(tick);
                    }
                    if !ticks.is_empty() {
                        world.pending_fluids.push((coord, ticks));
                    }
                }
                Some(_) => return Err(invalid("invalid fluid ticks in native chunk")),
                None => world.legacy_notifications.extend(changes.keys().copied()),
            }
            if !changes.is_empty() {
                world.chunks.insert(coord, Arc::new(changes));
            }
        }
        Ok(world)
    }

    pub fn write(
        &self,
        (x, z): Chunk,
        changes: Option<&Changes>,
        game_time: i64,
        fluids: &[SavedTick<Fluid>],
    ) -> io::Result<()> {
        let pos = ChunkPos::new(x, z);
        let mut tag = CompoundTag::new();
        tag.put_int("LeafNativeFormat", FORMAT_VERSION);
        tag.put_int("DataVersion", CURRENT_DATA_VERSION);
        tag.put_int("xPos", x);
        tag.put_int("yPos", MIN_Y >> 4);
        tag.put_int("zPos", z);
        tag.put_string("Status", "minecraft:full");
        tag.put_long("LastUpdate", game_time);
        tag.put_long("InhabitedTime", 0);
        tag.put(
            "fluid_ticks",
            Tag::List(
                fluids
                    .iter()
                    .map(|tick| {
                        let mut entry = CompoundTag::new();
                        tick.write_nbt(&mut entry);
                        Tag::Compound(entry)
                    })
                    .collect(),
            ),
        );

        let generated = self.terrain.chunk((x, z));
        let mut sections = generated
            .sections
            .iter()
            .map(|section| match section {
                Section::Uniform(state) => self.palette(i32::from(*state)),
                Section::Mixed(states) => {
                    let mut palette = self.palette(i32::from(states[0]));
                    for (index, &state) in states.iter().enumerate() {
                        palette.set_by_index_unchecked(index, i32::from(state));
                    }
                    palette
                }
            })
            .collect::<Vec<_>>();
        for (&block, &state) in changes.into_iter().flatten() {
            let index = ((block.y - MIN_Y) >> 4) as usize;
            sections[index].set(block.x & 15, block.y & 15, block.z & 15, state);
        }
        let mut biome_states = PalettedContainer::new(
            generated.biomes[0] as i32,
            Strategy::new(StrategyKind::Biomes, super::Biome::NAMES.len()),
            Some(self.blocks.clone()),
        );
        for index in 0..64 {
            biome_states.set_by_index_unchecked(index, generated.biomes[index % 16] as i32);
        }
        let (values, words) = biome_states.pack();
        let mut biomes = CompoundTag::new();
        biomes.put(
            "palette",
            Tag::List(
                values
                    .into_iter()
                    .map(|biome| {
                        Tag::String(format!("minecraft:{}", super::Biome::NAMES[biome as usize]))
                    })
                    .collect(),
            ),
        );
        if !words.is_empty() {
            biomes.put_long_array("data", &words);
        }
        let sections = sections.into_iter().enumerate().map(|(index, section)| {
            let (values, words) = section.pack();
            let states = values
                .into_iter()
                .map(|state| Tag::Compound(Blocks::vanilla().encode_state(state)))
                .collect();
            let mut block_states = CompoundTag::new();
            block_states.put("palette", Tag::List(states));
            if !words.is_empty() {
                block_states.put_long_array("data", &words);
            }
            let mut tag = CompoundTag::new();
            tag.put_byte("Y", ((MIN_Y >> 4) + index as i32) as i8);
            tag.put("block_states", Tag::Compound(block_states));
            tag.put("biomes", Tag::Compound(biomes.clone()));
            Tag::Compound(tag)
        });
        tag.put("sections", Tag::List(sections.collect()));
        self.regions.write(pos, &tag)
    }

    fn decode(&self, pos: ChunkPos, tag: &CompoundTag) -> io::Result<Changes> {
        if tag.get_int("LeafNativeFormat") != Some(FORMAT_VERSION)
            || tag.get_int("DataVersion") != Some(CURRENT_DATA_VERSION)
        {
            return Err(invalid(
                "unsupported native world format or block data version",
            ));
        }
        if tag.get_int("xPos") != Some(pos.x)
            || tag.get_int("zPos") != Some(pos.z)
            || tag.get_int("yPos") != Some(MIN_Y >> 4)
        {
            return Err(invalid("invalid native chunk position"));
        }
        // A 26-bit horizontal block coordinate leaves 22 signed bits for its chunk.
        let chunk_bounds = -(1 << 21)..(1 << 21);
        if !chunk_bounds.contains(&pos.x) || !chunk_bounds.contains(&pos.z) {
            return Err(invalid("native chunk is outside the world"));
        }
        let list = tag
            .get("sections")
            .and_then(Tag::as_list)
            .filter(|list| list.len() == SECTIONS)
            .ok_or_else(|| invalid("invalid native chunk sections"))?;
        let generated = self.terrain.chunk((pos.x, pos.z));
        let mut changes = Changes::new();
        let mut seen = [false; SECTIONS];
        for entry in list {
            let section = entry
                .as_compound()
                .ok_or_else(|| invalid("invalid native chunk section"))?;
            let y = section
                .get_byte("Y")
                .map(i32::from)
                .ok_or_else(|| invalid("missing section Y"))?;
            let index = (y - (MIN_Y >> 4)) as usize;
            let Some(seen) = seen.get_mut(index) else {
                return Err(invalid("invalid section Y"));
            };
            if std::mem::replace(seen, true) {
                return Err(invalid("duplicate native chunk section"));
            }
            let block_states = section
                .get_compound("block_states")
                .ok_or_else(|| invalid("missing block states"))?;
            let values = block_states
                .get_list_or_empty("palette")
                .iter()
                .map(|tag| {
                    tag.as_compound()
                        .and_then(|tag| Blocks::vanilla().decode_state(tag))
                        .ok_or_else(|| invalid("unknown block state in native world"))
                })
                .collect::<io::Result<Vec<_>>>()?;
            let words = match block_states.get("data") {
                None => None,
                Some(Tag::LongArray(words)) => Some(words.as_slice()),
                Some(_) => return Err(invalid("invalid native palette indices")),
            };
            let mut palette = self.palette(AIR);
            palette.unpack(values, words).map_err(invalid)?;
            let baseline = &generated.sections[index];
            if let Section::Uniform(state) = baseline {
                if !palette.maybe_has(&|current| *current != i32::from(*state)) {
                    continue;
                }
            }
            for index in 0..4096 {
                let state = palette.get_by_index(index);
                if state != i32::from(baseline.state(index)) {
                    let block = BlockPos::new(
                        pos.x * 16 + (index & 15) as i32,
                        y * 16 + (index >> 8) as i32,
                        pos.z * 16 + ((index >> 4) & 15) as i32,
                    );
                    changes.insert(block, state);
                }
            }
        }
        Ok(changes)
    }

    fn palette(&self, initial: i32) -> PalettedContainer<i32> {
        PalettedContainer::new(
            initial,
            Strategy::new(StrategyKind::BlockStates, self.blocks.size()),
            Some(self.blocks.clone()),
        )
    }
}

struct StateIds;

impl IdMap<i32> for StateIds {
    fn by_id(&self, id: i32) -> Option<i32> {
        ((0..self.size() as i32).contains(&id)).then_some(id)
    }

    fn get_id(&self, state: &i32) -> i32 {
        self.by_id(*state).unwrap_or(-1)
    }

    fn size(&self) -> usize {
        Blocks::vanilla().state_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audience::Audience;
    use crate::tests::TestWorld;
    use crate::world::{set_state, FillMode, NativeWorld, STONE};

    #[test]
    fn block_state_names_and_properties_round_trip_for_the_entire_registry() {
        let blocks = Blocks::vanilla();
        assert_eq!(blocks.state_count(), 29_671);
        for id in 0..blocks.state_count() as i32 {
            assert_eq!(blocks.decode_state(&blocks.encode_state(id)), Some(id));
        }
    }

    #[test]
    fn retains_unsaved_chunks_after_io_failure_and_persists_removal_of_the_last_edit() {
        let directory = TestWorld::new();
        std::fs::create_dir_all(&directory.0).unwrap();
        let path = directory.0.join("native-region");
        let world =
            NativeWorld::open([0; 10], super::super::Generation::FLAT, path.clone()).unwrap();
        let pos = BlockPos::new(-513, 15, 15);
        let audience = Audience::default();
        assert!(world.break_block(pos, STONE, &audience));
        {
            // Force a global palette and verify that on-disk palettes remain local.
            let mut edits = world.edits.lock().unwrap();
            for id in 1..=300 {
                let block = BlockPos::new(-528 + (id % 16), 16 + (id / 256), (id / 16) % 16);
                set_state(&mut edits, block, id);
            }
        }
        std::fs::write(&path, "cannot create a region directory yet").unwrap();
        assert!(world.save(10).is_err());
        std::fs::remove_file(&path).unwrap();
        assert_eq!(world.save(20).unwrap(), 1);
        assert_eq!(world.save(21).unwrap(), 0);

        let world =
            NativeWorld::open([0; 10], super::super::Generation::FLAT, path.clone()).unwrap();
        assert_eq!(world.block(pos), AIR);
        for id in 1..=300 {
            let block = BlockPos::new(-528 + (id % 16), 16 + (id / 256), (id / 16) % 16);
            assert_eq!(world.block(block), id);
        }

        world
            .fill(pos, pos, STONE, FillMode::Replace(None), 1, &audience)
            .unwrap();
        world
            .fill(
                BlockPos::new(-528, 16, 0),
                BlockPos::new(-513, 17, 15),
                AIR,
                FillMode::Replace(None),
                512,
                &audience,
            )
            .unwrap();
        assert_eq!(world.save(30).unwrap(), 1);
        let restored = NativeWorld::open([0; 10], super::super::Generation::FLAT, path).unwrap();
        assert_eq!(restored.block(pos), STONE);
        assert!(restored.edits.lock().unwrap().is_empty());
    }
}
