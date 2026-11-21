//! Загруженные чанки измерения и очереди тиков, привязанные к их жизненному циклу.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io;
use std::sync::Arc;

use crate::core::registry_access::RegistryAccess;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::storage::region_file_storage::RegionFileStorage;
use crate::world::level::chunk::storage::serializable_chunk_data::{
    ChunkLoadingContext, LoadedChunk, SerializableChunkData,
};
use crate::world::level::chunk::RuntimeChunk;
use crate::world::level::chunk_pos::ChunkPos;
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

    /// Подключает очереди после перехода полного освещённого чанка в ticking.
    pub fn start_ticking(&mut self, pos: ChunkPos, game_time: i64) -> bool {
        let Some(cached) = self.chunks.get_mut(&pos) else {
            return false;
        };
        let RuntimeChunk::Full(chunk) = &cached.data.chunk else {
            return false;
        };
        if !chunk.chunk_access.is_light_correct {
            return false;
        }
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
    use crate::world::level::chunk::packed_ticks::PackedTicks;
    use crate::world::level::chunk::storage::serializable_chunk_data::ChunkPaletteLookup;
    use crate::world::level::level_height_accessor::SimpleLevelHeightAccessor;
    use crate::world::ticks::{SavedTick, TickPriority};

    struct Palettes;

    impl ChunkPaletteLookup for Palettes {
        fn block_state(&self, nbt: &CompoundTag) -> Option<BlockState> {
            (nbt.get_string("Name") == Some("minecraft:air")).then(BlockState::air)
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
}
