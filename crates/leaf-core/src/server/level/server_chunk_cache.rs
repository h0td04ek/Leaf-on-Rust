//! Загруженные чанки измерения и очереди тиков, привязанные к их жизненному циклу.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{Arc, Mutex};

use crate::ca::moonrise::patches::starlight::light::star_light_engine::{
    EngineKind, StarLightEngine,
};
use crate::core::registry_access::RegistryAccess;
use crate::core::section_pos::SectionPos;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::level_chunk::ChunkStatus;
use crate::world::level::chunk::storage::region_file_storage::RegionFileStorage;
use crate::world::level::chunk::storage::serializable_chunk_data::{
    ChunkLoadingContext, LoadedChunk, SerializableChunkData,
};
use crate::world::level::chunk::LightChunkGetter;
use crate::world::level::chunk::RuntimeChunk;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::light_layer::LightLayer;
use crate::world::level::scheduled_tick_access::{BlockTickType, FluidTickType};
use crate::world::ticks::LevelTicks;

pub struct ServerChunkCache {
    storage: RegionFileStorage,
    chunks: HashMap<ChunkPos, CachedChunk>,
    block_ticks: LevelTicks<BlockTickType>,
    fluid_ticks: LevelTicks<FluidTickType>,
}

struct CachedChunk {
    data: LoadedChunk,
    has_sky_light: bool,
    light_prepared: bool,
    ticking: bool,
    needs_save: bool,
}

impl ServerChunkCache {
    pub fn new(storage: RegionFileStorage) -> Self {
        Self {
            storage,
            chunks: HashMap::new(),
            block_ticks: LevelTicks::new(Arc::new(|_| true)),
            fluid_ticks: LevelTicks::new(Arc::new(|_| true)),
        }
    }

    /// Отсутствие записи позволяет вызвать генерацию. Ошибка чтения оставляет
    /// запись на диске и передаётся вызывающему коду для повторной попытки.
    pub fn load_existing(
        &mut self,
        pos: ChunkPos,
        context: &ChunkLoadingContext<'_>,
    ) -> io::Result<Option<&LoadedChunk>> {
        let chunk = match self.chunks.entry(pos) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let Some(data) = SerializableChunkData::read(&self.storage, pos, context)? else {
                    return Ok(None);
                };
                entry.insert(CachedChunk {
                    data: data.into_runtime(None),
                    has_sky_light: context.has_sky_light,
                    light_prepared: false,
                    ticking: false,
                    needs_save: false,
                })
            }
        };
        Ok(Some(&chunk.data))
    }

    pub fn get_loaded(&self, pos: ChunkPos) -> Option<&LoadedChunk> {
        self.chunks.get(&pos).map(|chunk| &chunk.data)
    }

    pub fn get_chunk_access_mut(&mut self, pos: ChunkPos) -> Option<&mut ChunkAccess> {
        let chunk = self.chunks.get_mut(&pos)?;
        chunk.needs_save = true;
        Some(chunk.data.chunk.access_mut())
    }

    pub fn block_ticks(&self) -> &LevelTicks<BlockTickType> {
        &self.block_ticks
    }

    pub fn fluid_ticks(&self) -> &LevelTicks<FluidTickType> {
        &self.fluid_ticks
    }

    /// Выполняет Light для сохранённого чанка с доступными соседями.
    pub fn prepare_light(&mut self, pos: ChunkPos) -> bool {
        let Some(cached) = self.chunks.get(&pos) else {
            return false;
        };
        let chunk = cached.data.chunk.access();
        let status = chunk.persisted_status;
        if !status.is_or_after(ChunkStatus::InitializeLight) {
            return false;
        }
        let already_lit = status.is_or_after(ChunkStatus::Light) && chunk.is_light_correct;
        if cached.light_prepared && already_lit {
            return true;
        }

        let min_section = chunk.get_min_section();
        let max_section = chunk.get_max_section();
        let has_sky_light = cached.has_sky_light;
        let access = CacheLightingAccess {
            cache: self,
            min_section,
            max_section,
            has_sky_light,
            modified: Mutex::new(HashSet::new()),
        };
        let mut empty_sections = StarLightEngine::get_empty_sections_for_chunk(chunk);
        let engines = [EngineKind::Sky, EngineKind::Block];
        for kind in engines.into_iter().skip(usize::from(!has_sky_light)) {
            let mut engine = StarLightEngine::new(kind);
            engine.set_section_range(min_section, max_section, false);
            if already_lit {
                engine.force_handle_empty_section_changes(&access, chunk, &mut empty_sections);
                engine.check_chunk_edges(&access, pos.x, pos.z);
            } else {
                engine.light(&access, chunk, &mut empty_sections);
            }
        }

        for modified in access.modified.into_inner().unwrap() {
            if let Some(cached) = self.chunks.get_mut(&modified) {
                cached.needs_save = true;
            }
        }
        let cached = self.chunks.get_mut(&pos).unwrap();
        cached.data.chunk.access_mut().is_light_correct = true;
        cached.light_prepared = true;
        cached.needs_save |= !already_lit;
        if let RuntimeChunk::Proto(chunk) = &mut cached.data.chunk {
            if status == ChunkStatus::InitializeLight {
                chunk.set_chunk_status(ChunkStatus::Light);
            }
        }
        true
    }

    /// Подключает очереди после перехода полного освещённого чанка в ticking.
    pub fn start_ticking(&mut self, pos: ChunkPos, game_time: i64) -> bool {
        if !matches!(
            self.get_loaded(pos).map(|data| &data.chunk),
            Some(RuntimeChunk::Full(_))
        ) || !self.prepare_light(pos)
        {
            return false;
        }
        let cached = self.chunks.get_mut(&pos).unwrap();
        let RuntimeChunk::Full(chunk) = &cached.data.chunk else {
            unreachable!("lighting does not replace full chunks");
        };
        if !cached.ticking {
            chunk.register_tick_containers(&self.block_ticks, &self.fluid_ticks, game_time);
            cached.ticking = true;
            cached.needs_save = true;
        }
        true
    }

    pub fn stop_ticking(&mut self, pos: ChunkPos) {
        let Some(cached) = self.chunks.get_mut(&pos) else {
            return;
        };
        if let RuntimeChunk::Full(chunk) = &cached.data.chunk {
            chunk.unregister_tick_containers(&self.block_ticks, &self.fluid_ticks);
        }
        cached.needs_save |= cached.ticking;
        cached.ticking = false;
    }

    pub fn save(
        &mut self,
        pos: ChunkPos,
        game_time: i64,
        registries: &RegistryAccess,
    ) -> io::Result<bool> {
        let Some(cached) = self.chunks.get_mut(&pos) else {
            return Ok(false);
        };
        if cached.needs_save || cached.ticking {
            let tag = cached
                .data
                .to_nbt(game_time, registries, cached.has_sky_light)?;
            self.storage.write(pos, &tag)?;
            cached.data.last_update_time = game_time;
            cached.data.original_tag = tag;
            match &mut cached.data.chunk {
                RuntimeChunk::Full(chunk) => chunk.set_unsaved(false),
                RuntimeChunk::Proto(chunk) => chunk.chunk_access.set_unsaved(false),
            }
            cached.needs_save = false;
        }
        Ok(true)
    }

    /// При ошибке записи оставляет чанк и его очереди в кэше.
    pub fn unload(
        &mut self,
        pos: ChunkPos,
        game_time: i64,
        registries: &RegistryAccess,
    ) -> io::Result<bool> {
        if !self.save(pos, game_time, registries)? {
            return Ok(false);
        }
        self.stop_ticking(pos);
        self.chunks.remove(&pos);
        Ok(true)
    }
}

struct CacheLightingAccess<'a> {
    cache: &'a ServerChunkCache,
    min_section: i32,
    max_section: i32,
    has_sky_light: bool,
    modified: Mutex<HashSet<ChunkPos>>,
}

impl LightChunkGetter for CacheLightingAccess<'_> {
    fn get_chunk_for_lighting(&self, x: i32, z: i32) -> Option<&ChunkAccess> {
        let cached = self.cache.chunks.get(&ChunkPos::new(x, z))?;
        let chunk = cached.data.chunk.access();
        (cached.has_sky_light == self.has_sky_light
            && chunk.get_min_section() == self.min_section
            && chunk.get_max_section() == self.max_section)
            .then_some(chunk)
    }

    fn on_light_update(&self, _layer: LightLayer, section: SectionPos) {
        self.modified
            .lock()
            .unwrap()
            .insert(ChunkPos::new(section.get_x(), section.get_z()));
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;
    use crate::core::block_pos::BlockPos;
    use crate::core::holder::Holder;
    use crate::core::registry::registries;
    use crate::nbt::{utils, CompoundTag};
    use crate::resources::{Identifier, ResourceKey};
    use crate::world::level::biome::biome::Biome;
    use crate::world::level::block::state::BlockState;
    use crate::world::level::block::Block;
    use crate::world::level::chunk::packed_ticks::PackedTicks;
    use crate::world::level::chunk::storage::serializable_chunk_data::ChunkPaletteLookup;
    use crate::world::level::level_height_accessor::SimpleLevelHeightAccessor;
    use crate::world::ticks::{SavedTick, TickPriority};

    struct Palettes;

    impl ChunkPaletteLookup for Palettes {
        fn block_state(&self, nbt: &CompoundTag) -> Option<BlockState> {
            match nbt.get_string("Name")? {
                "minecraft:air" => Some(BlockState::air()),
                "minecraft:torch" => {
                    let mut block = Block::new(Identifier::minecraft("torch"));
                    block.properties.light_emission = Some(Arc::new(|_| 14));
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

    struct TestDirectory(PathBuf);

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_file(self.0.join("r.-1.0.mca"));
            let _ = fs::remove_dir(&self.0);
        }
    }

    #[test]
    fn keeps_failed_unloads_resident_and_restores_ticks_after_reload() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = TestDirectory(
            std::env::temp_dir().join(format!("leaf-chunk-cache-{}-{nonce}", std::process::id())),
        );
        let storage = RegionFileStorage::new(&directory.0);
        let pos = ChunkPos::new(-1, 0);
        let block = BlockPos::new(-16, -64, 0);
        let registries = RegistryAccess::new();
        let height = SimpleLevelHeightAccessor::create(-64, 16);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &Palettes,
            data_fixer: None,
            has_sky_light: false,
        };

        let mut tag = CompoundTag::new();
        utils::add_current_data_version(&mut tag);
        tag.put_int("xPos", pos.x);
        tag.put_int("zPos", pos.z);
        tag.put_string("Status", "minecraft:full");
        tag.put_bool("isLightOn", false);
        tag.put_int("starlight.light_version", 10);
        let ticks = PackedTicks {
            blocks: vec![SavedTick::new(
                BlockTickType {
                    id: "minecraft:observer".into(),
                },
                block,
                6,
                TickPriority::High,
            )],
            fluids: Vec::new(),
        };
        ticks.write_nbt(&mut tag);
        storage.write(pos, &tag).unwrap();

        let mut cache = ServerChunkCache::new(storage);
        assert!(cache
            .load_existing(ChunkPos::new(0, 0), &context)
            .unwrap()
            .is_none());
        assert!(cache.load_existing(pos, &context).unwrap().is_some());
        assert!(cache.block_ticks().get_container(pos).is_none());
        assert!(cache.start_ticking(pos, 24));
        assert!(cache.start_ticking(pos, 28));

        let access = cache.get_chunk_access_mut(pos).unwrap();
        access.sections[0]
            .biomes
            .set(0, 0, 0, Holder::Reference(999));
        let error = cache.unload(pos, 28, &registries).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(cache.get_loaded(pos).is_some());
        assert!(cache.block_ticks().get_container(pos).is_some());

        let access = cache.get_chunk_access_mut(pos).unwrap();
        access.sections[0]
            .biomes
            .set(0, 0, 0, Palettes.default_biome());
        access.inhabited_time = 40;
        assert!(cache.unload(pos, 28, &registries).unwrap());
        assert!(cache.get_loaded(pos).is_none());
        assert!(cache.block_ticks().get_container(pos).is_none());

        let loaded = cache.load_existing(pos, &context).unwrap().unwrap();
        assert_eq!(loaded.last_update_time, 28);
        assert_eq!(loaded.chunk.access().inhabited_time, 40);
        assert!(cache.start_ticking(pos, 100));
        let mut ran = Vec::new();
        cache
            .block_ticks()
            .tick(101, 8, &mut |_, _| panic!("tick ran early"));
        cache
            .block_ticks()
            .tick(102, 8, &mut |pos, kind| ran.push((pos, kind.id)));
        assert_eq!(ran, [(block, "minecraft:observer".into())]);
    }

    #[test]
    fn lights_loaded_chunk_before_ticking_and_reloads_the_visible_result() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = TestDirectory(
            std::env::temp_dir().join(format!("leaf-chunk-light-{}-{nonce}", std::process::id())),
        );
        let storage = RegionFileStorage::new(&directory.0);
        let pos = ChunkPos::new(-1, 0);
        let torch = BlockPos::new(-8, -56, 8);
        let adjacent = BlockPos::new(-7, -56, 8);
        let height = SimpleLevelHeightAccessor::create(-64, 32);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &Palettes,
            data_fixer: None,
            has_sky_light: true,
        };
        let registries = RegistryAccess::new();
        let mut tag = CompoundTag::new();
        utils::add_current_data_version(&mut tag);
        tag.put_int("xPos", pos.x);
        tag.put_int("zPos", pos.z);
        tag.put_string("Status", "minecraft:full");
        storage.write(pos, &tag).unwrap();

        let mut cache = ServerChunkCache::new(storage);
        let loaded = cache.load_existing(pos, &context).unwrap().unwrap();
        assert!(!loaded.chunk.access().is_light_correct);
        let mut state = CompoundTag::new();
        state.put_string("Name", "minecraft:torch");
        assert!(cache
            .get_chunk_access_mut(pos)
            .unwrap()
            .set_block_state(&torch, Palettes.block_state(&state).unwrap(),));
        assert!(cache.start_ticking(pos, 24));
        let verify_light = |chunk: &ChunkAccess| {
            assert!(chunk.is_light_correct);
            let block_light = chunk.starlight_block_nibbles.lock().unwrap();
            let section = block_light[1].as_ref().unwrap().lock().unwrap();
            assert_eq!(
                section.get_visible_xyz(torch.get_x(), torch.get_y(), torch.get_z()),
                14
            );
            assert_eq!(
                section.get_visible_xyz(adjacent.get_x(), adjacent.get_y(), adjacent.get_z()),
                13
            );
        };
        verify_light(cache.get_loaded(pos).unwrap().chunk.access());
        assert!(cache.unload(pos, 25, &registries).unwrap());

        let loaded = cache.load_existing(pos, &context).unwrap().unwrap();
        verify_light(loaded.chunk.access());
        assert!(cache.prepare_light(pos));
        let access = cache.get_loaded(pos).unwrap().chunk.access();
        assert!(access.starlight_sky_emptiness_map.lock().unwrap().is_some());
        verify_light(access);
    }
}
