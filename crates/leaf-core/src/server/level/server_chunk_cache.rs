//! Загруженные чанки измерения и очереди тиков, привязанные к их жизненному циклу.

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::{Arc, Mutex};

use crate::ca::moonrise::patches::starlight::light::star_light_engine::{
    EngineKind, StarLightEngine,
};
use crate::core::block_pos::BlockPos;
use crate::core::registry_access::RegistryAccess;
use crate::core::section_pos::SectionPos;
use crate::nbt::{utils, CompoundTag, Tag};
use crate::world::level::block::state::BlockState;
use crate::world::level::chunk::chunk_access::ChunkAccess;
use crate::world::level::chunk::chunk_pyramid::ChunkStep;
use crate::world::level::chunk::level_chunk::{ChunkStatus, LevelChunk};
use crate::world::level::chunk::storage::region_file_storage::RegionFileStorage;
use crate::world::level::chunk::storage::serializable_chunk_data::{
    ChunkLoadingContext, LoadedChunk, SerializableChunkData,
};
use crate::world::level::chunk::LightChunkGetter;
use crate::world::level::chunk::RuntimeChunk;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::light_layer::LightLayer;
use crate::world::level::lighting::light_engine::has_different_light_properties;
use crate::world::level::scheduled_tick_access::{BlockTickType, FluidTickType};
use crate::world::ticks::LevelTicks;

use super::chunk_generation_region::{ChunkGenerationRegion, ChunkStageExecutor, GenerationEdits};
use super::chunk_generation_task::{ChunkGenerationTask, ChunkStepMode, GeneratingChunkMap};

pub enum ChunkLoadOutcome<'a> {
    Ready(&'a LoadedChunk),
    NeedsGeneration { persisted: Option<ChunkStatus> },
}

pub struct ServerChunkCache {
    storage: RegionFileStorage,
    chunks: HashMap<ChunkPos, CachedChunk>,
    block_ticks: LevelTicks<BlockTickType>,
    fluid_ticks: LevelTicks<FluidTickType>,
}

struct CachedChunk {
    data: LoadedChunk,
    completed_stage: ChunkStatus,
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
                    completed_stage: ChunkStatus::Empty,
                    has_sky_light: context.has_sky_light,
                    light_prepared: false,
                    ticking: false,
                    needs_save: false,
                })
            }
        };
        Ok(Some(&chunk.data))
    }

    /// Готовит сохранённый чанк до нужной стадии. При отсутствии записи или
    /// стадий генерации возвращает сохранённый статус, если он имеется.
    pub fn load_through(
        &mut self,
        task: ChunkGenerationTask,
        context: &ChunkLoadingContext<'_>,
    ) -> io::Result<ChunkLoadOutcome<'_>> {
        let mut map = CachedChunkMap {
            cache: self,
            context,
            executor: None,
        };
        if !task.try_load(&mut map)? {
            return Ok(ChunkLoadOutcome::NeedsGeneration {
                persisted: map.persisted_status(task.center),
            });
        }
        Ok(ChunkLoadOutcome::Ready(&self.chunks[&task.center].data))
    }

    /// Выполняет путь загрузки или генерации до запрошенного статуса.
    /// Стадии рельефа и контента выполняет переданный генератор.
    pub fn generate_through(
        &mut self,
        task: ChunkGenerationTask,
        context: &ChunkLoadingContext<'_>,
        executor: &mut dyn ChunkStageExecutor,
    ) -> io::Result<&LoadedChunk> {
        task.run(&mut CachedChunkMap {
            cache: self,
            context,
            executor: Some(executor),
        })?;
        Ok(&self.chunks[&task.center].data)
    }

    pub fn completed_status(&self, pos: ChunkPos) -> Option<ChunkStatus> {
        Some(self.chunks.get(&pos)?.completed_stage)
    }

    fn apply_generation_edits(&mut self, pos: ChunkPos, edits: GenerationEdits) {
        for (block, state) in edits.blocks {
            self.set_block_state(block, state);
        }
        let cached = self.chunks.get_mut(&pos).unwrap();
        let access = cached.data.chunk.access_mut();
        for (block, biome) in edits.biomes {
            access
                .get_section_mut(block.get_y() >> 4)
                .unwrap()
                .set_biome(
                    (block.get_x() & 15) >> 2,
                    (block.get_y() & 15) >> 2,
                    (block.get_z() & 15) >> 2,
                    biome,
                );
        }
        if let RuntimeChunk::Proto(chunk) = &mut cached.data.chunk {
            chunk.entities.extend(edits.entities);
        }
    }

    fn promote_full(&mut self, pos: ChunkPos) -> io::Result<()> {
        let data = &self.chunks[&pos].data;
        if !matches!(&data.chunk, RuntimeChunk::Proto(chunk)
            if chunk.get_chunk_status() == ChunkStatus::Spawn && chunk.chunk_access.is_light_correct)
        {
            return Err(io::Error::other(format!(
                "chunk {pos:?} must finish spawn and lighting before promotion"
            )));
        }

        let mut cached = self.chunks.remove(&pos).unwrap();
        let RuntimeChunk::Proto(mut proto) = cached.data.chunk else {
            unreachable!("verified proto chunk before removal");
        };
        let ticks = proto.pack_ticks();
        proto.set_chunk_status(ChunkStatus::Full);
        let mut chunk =
            LevelChunk::from_access(proto.chunk_access, ticks, proto.post_processing, None);
        chunk.light_engine = proto.light_engine;
        // Full может прочитать inline-сущности до подключения отдельного EntityStorage.
        if proto.entities.is_empty() {
            cached.data.original_tag.remove("entities");
        } else {
            let entities = proto.entities.into_iter().map(Tag::Compound).collect();
            cached
                .data
                .original_tag
                .put("entities", Tag::List(entities));
        }
        cached.data.original_tag.remove("carving_mask");
        cached.data.chunk = RuntimeChunk::Full(chunk);
        cached.needs_save = true;
        self.chunks.insert(pos, cached);
        Ok(())
    }

    pub fn get_loaded(&self, pos: ChunkPos) -> Option<&LoadedChunk> {
        self.chunks.get(&pos).map(|chunk| &chunk.data)
    }

    /// Доступ к данным для стадий генерации и обновления метаданных.
    /// Игровые изменения блоков проходят через set_block_state.
    pub fn get_chunk_access_mut(&mut self, pos: ChunkPos) -> Option<&mut ChunkAccess> {
        let chunk = self.chunks.get_mut(&pos)?;
        chunk.needs_save = true;
        Some(chunk.data.chunk.access_mut())
    }

    pub fn set_block_state(&mut self, pos: BlockPos, state: BlockState) -> bool {
        let chunk_pos = ChunkPos::new(pos.get_x() >> 4, pos.get_z() >> 4);
        let Some(cached) = self.chunks.get(&chunk_pos) else {
            return false;
        };
        let chunk = cached.data.chunk.access();
        let section_y = pos.get_y() >> 4;
        let Some(section) = chunk.get_section(section_y) else {
            return false;
        };
        let old = chunk.get_block_state(&pos);
        if old == state {
            return false;
        }
        let was_empty = section.has_only_air();
        let section_index = chunk.get_section_index_from_section_y(section_y) as usize;
        let light_changed = has_different_light_properties(&old, &state);
        let check_light = (light_changed || old.is_air() != state.is_air())
            && chunk.persisted_status.is_or_after(ChunkStatus::Light);
        if check_light {
            // Для чанка с дисковым светом сначала восстанавливаем карту пустых секций.
            self.prepare_light(chunk_pos);
        }

        let cached = self.chunks.get_mut(&chunk_pos).unwrap();
        let chunk = cached.data.chunk.access_mut();
        if !chunk.set_block_state(&pos, state) {
            return false;
        }
        cached.needs_save = true;
        let is_empty = chunk.sections[section_index].has_only_air();
        if check_light && (light_changed || is_empty != was_empty) {
            let positions = if light_changed {
                HashSet::from([pos])
            } else {
                HashSet::new()
            };
            let mut changed_sections = (is_empty != was_empty).then(|| {
                let mut changes = vec![None; chunk.sections.len()];
                changes[section_index] = Some(is_empty);
                changes
            });
            self.run_light(chunk_pos, |engine, access, _| {
                engine.blocks_changed_in_chunk(
                    access,
                    chunk_pos.x,
                    chunk_pos.z,
                    &positions,
                    changed_sections.as_deref_mut(),
                );
            });
        }
        true
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

        let mut empty_sections = StarLightEngine::get_empty_sections_for_chunk(chunk);
        self.run_light(pos, |engine, access, chunk| {
            if already_lit {
                engine.force_handle_empty_section_changes(access, chunk, &mut empty_sections);
                engine.check_chunk_edges(access, pos.x, pos.z);
            } else {
                engine.light(access, chunk, &mut empty_sections);
            }
        });
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

    fn run_light(
        &mut self,
        pos: ChunkPos,
        mut pass: impl FnMut(&mut StarLightEngine, &dyn LightChunkGetter, &ChunkAccess),
    ) {
        let cached = &self.chunks[&pos];
        let chunk = cached.data.chunk.access();
        let access = CacheLightingAccess {
            cache: self,
            min_section: chunk.get_min_section(),
            max_section: chunk.get_max_section(),
            has_sky_light: cached.has_sky_light,
            modified: Mutex::new(HashSet::new()),
        };
        let kinds = [EngineKind::Sky, EngineKind::Block];
        for kind in kinds.into_iter().skip(usize::from(!access.has_sky_light)) {
            let mut engine = StarLightEngine::new(kind);
            engine.set_section_range(access.min_section, access.max_section, false);
            pass(&mut engine, &access, chunk);
        }

        for modified in access.modified.into_inner().unwrap() {
            if let Some(cached) = self.chunks.get_mut(&modified) {
                cached.needs_save = true;
            }
        }
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

struct CachedChunkMap<'a, 'b, 'c> {
    cache: &'a mut ServerChunkCache,
    context: &'b ChunkLoadingContext<'c>,
    executor: Option<&'a mut dyn ChunkStageExecutor>,
}

impl GeneratingChunkMap for CachedChunkMap<'_, '_, '_> {
    type Error = io::Error;

    fn persisted_status(&self, pos: ChunkPos) -> Option<ChunkStatus> {
        Some(self.cache.get_loaded(pos)?.chunk.access().persisted_status)
    }

    fn completed_status(&self, pos: ChunkPos) -> Option<ChunkStatus> {
        self.cache.completed_status(pos)
    }

    fn apply_step(
        &mut self,
        pos: ChunkPos,
        step: &ChunkStep,
        mode: ChunkStepMode,
    ) -> io::Result<()> {
        let status = step.target_status;
        if status == ChunkStatus::Empty {
            if self.cache.load_existing(pos, self.context)?.is_none() && self.executor.is_some() {
                let mut nbt = CompoundTag::new();
                utils::add_current_data_version(&mut nbt);
                nbt.put_int("xPos", pos.x);
                nbt.put_int("zPos", pos.z);
                nbt.put_string("Status", "minecraft:empty");
                let data = SerializableChunkData::parse(nbt, pos, self.context)?.into_runtime(None);
                self.cache.chunks.insert(
                    pos,
                    CachedChunk {
                        data,
                        completed_stage: ChunkStatus::Empty,
                        has_sky_light: self.context.has_sky_light,
                        light_prepared: false,
                        ticking: false,
                        needs_save: false,
                    },
                );
            }
            return Ok(());
        }

        match (status, mode) {
            (ChunkStatus::Light, _) => {
                if !self.cache.prepare_light(pos) {
                    return Err(io::Error::other(format!("light stage failed for {pos:?}")));
                }
            }
            (ChunkStatus::Full, ChunkStepMode::Generate) => self.cache.promote_full(pos)?,
            (ChunkStatus::InitializeLight, _) | (_, ChunkStepMode::Load) => {}
            (_, ChunkStepMode::Generate) => {
                let mut region = ChunkGenerationRegion::new(self.cache, step, pos);
                self.executor.as_deref_mut().unwrap().execute(&mut region)?;
                let edits = region.into_edits();
                self.cache.apply_generation_edits(pos, edits);
            }
        }

        let cached = self.cache.chunks.get_mut(&pos).unwrap();
        if mode == ChunkStepMode::Generate {
            if let RuntimeChunk::Proto(chunk) = &mut cached.data.chunk {
                if !chunk.get_chunk_status().is_or_after(status) {
                    chunk.set_chunk_status(status);
                    let access = &mut chunk.chunk_access;
                    let missing: Vec<_> = status
                        .heightmaps_after()
                        .iter()
                        .copied()
                        .filter(|ty| !access.heightmaps.contains_key(ty))
                        .collect();
                    access.prime_heightmaps(&missing);
                    cached.needs_save = true;
                }
            }
        }
        cached.completed_stage = status;
        Ok(())
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
    use crate::core::holder::Holder;
    use crate::core::registry::registries;
    use crate::resources::{Identifier, ResourceKey};
    use crate::world::level::biome::biome::Biome;
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
        let neighbour = ChunkPos::new(-2, 0);
        let torch = BlockPos::new(-16, -56, 8);
        let across_border = BlockPos::new(-17, -56, 8);
        let adjacent = BlockPos::new(-15, -56, 8);
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
        tag.put_int("xPos", neighbour.x);
        storage.write(neighbour, &tag).unwrap();

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
        cache.load_existing(neighbour, &context).unwrap().unwrap();
        assert!(cache.prepare_light(neighbour));
        let block_light = |cache: &ServerChunkCache, pos: BlockPos| {
            let at = ChunkPos::new(pos.get_x() >> 4, pos.get_z() >> 4);
            let chunk = cache.get_loaded(at).unwrap().chunk.access();
            let block_light = chunk.starlight_block_nibbles.lock().unwrap();
            let nibble = block_light[1].as_ref().unwrap().lock().unwrap();
            nibble.get_visible_xyz(pos.get_x(), pos.get_y(), pos.get_z())
        };
        assert_eq!(block_light(&cache, across_border), 13);

        assert!(cache.set_block_state(torch, BlockState::air()));
        assert_eq!(block_light(&cache, torch), 0);
        assert_eq!(block_light(&cache, across_border), 0);
        assert!(cache.set_block_state(torch, Palettes.block_state(&state).unwrap()));
        assert_eq!(block_light(&cache, torch), 14);
        assert_eq!(block_light(&cache, across_border), 13);
        assert!(cache.unload(neighbour, 25, &registries).unwrap());
        assert!(cache.unload(pos, 25, &registries).unwrap());

        let loaded = cache.load_existing(pos, &context).unwrap().unwrap();
        verify_light(loaded.chunk.access());
        assert!(cache.prepare_light(pos));
        let access = cache.get_loaded(pos).unwrap().chunk.access();
        assert!(access.starlight_sky_emptiness_map.lock().unwrap().is_some());
        verify_light(access);
        cache.load_existing(neighbour, &context).unwrap().unwrap();
        assert_eq!(block_light(&cache, across_border), 13);
    }

    #[test]
    fn runs_saved_load_stages_and_distinguishes_generation_from_corruption() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = TestDirectory(
            std::env::temp_dir().join(format!("leaf-load-stages-{}-{nonce}", std::process::id(),)),
        );
        let storage = RegionFileStorage::new(&directory.0);
        let full = ChunkPos::new(-1, 0);
        let proto = ChunkPos::new(-2, 0);
        let missing = ChunkPos::new(-3, 0);
        let bad = ChunkPos::new(-4, 0);
        for (pos, status) in [
            (full, "minecraft:full"),
            (proto, "minecraft:features"),
            (bad, "fixture:bad_status"),
        ] {
            let mut nbt = CompoundTag::new();
            utils::add_current_data_version(&mut nbt);
            nbt.put_int("xPos", pos.x);
            nbt.put_int("zPos", pos.z);
            nbt.put_string("Status", status);
            storage.write(pos, &nbt).unwrap();
        }

        let mut cache = ServerChunkCache::new(storage);
        let height = SimpleLevelHeightAccessor::create(-64, 16);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &Palettes,
            data_fixer: None,
            has_sky_light: false,
        };
        let request = |center, target_status| ChunkGenerationTask {
            center,
            target_status,
        };

        assert!(matches!(
            cache
                .load_through(request(missing, ChunkStatus::Empty), &context)
                .unwrap(),
            ChunkLoadOutcome::NeedsGeneration { persisted: None }
        ));
        assert!(matches!(
            cache
                .load_through(request(proto, ChunkStatus::Light), &context)
                .unwrap(),
            ChunkLoadOutcome::NeedsGeneration {
                persisted: Some(ChunkStatus::Features)
            }
        ));
        let error = cache
            .load_through(request(bad, ChunkStatus::Full), &context)
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(cache.get_loaded(bad).is_none());

        let ChunkLoadOutcome::Ready(chunk) = cache
            .load_through(request(full, ChunkStatus::Features), &context)
            .unwrap()
        else {
            panic!("expected a saved full chunk");
        };
        assert!(!chunk.chunk.access().is_light_correct);
        assert_eq!(cache.chunks[&full].completed_stage, ChunkStatus::Features);

        let ChunkLoadOutcome::Ready(chunk) = cache
            .load_through(request(full, ChunkStatus::Full), &context)
            .unwrap()
        else {
            panic!("expected a loaded full chunk");
        };
        assert!(chunk.chunk.access().is_light_correct);
        assert_eq!(cache.chunks[&full].completed_stage, ChunkStatus::Full);
        assert!(cache.block_ticks().get_container(full).is_none());
        assert_eq!(cache.chunks.len(), 2);
    }

    #[test]
    fn commits_successful_generation_stages_and_promotes_proto_without_losing_data() {
        use crate::world::level::levelgen::heightmap::HeightmapType;

        struct Generator {
            center: ChunkPos,
            source: BlockPos,
            fail_noise: bool,
            failures: usize,
        }

        impl ChunkStageExecutor for Generator {
            fn execute(&mut self, region: &mut ChunkGenerationRegion<'_>) -> io::Result<()> {
                if region.center != self.center {
                    return Ok(());
                }
                match region.step.target_status {
                    ChunkStatus::Biomes => {
                        region.set_biome(
                            self.source,
                            Palettes.biome(&Identifier::minecraft("forest")).unwrap(),
                        )?;
                    }
                    ChunkStatus::Noise => {
                        let mut nbt = CompoundTag::new();
                        nbt.put_string("Name", "minecraft:torch");
                        region.set_block_state(self.source, Palettes.block_state(&nbt).unwrap())?;
                        assert!(region
                            .block_state(self.source)
                            .unwrap()
                            .is_block_id("minecraft:torch"));
                        if self.fail_noise {
                            self.failures += 1;
                            self.fail_noise = false;
                            return Err(io::Error::other("fixture noise failure"));
                        }
                    }
                    ChunkStatus::Features => {
                        let outside_radius = self.source.offset(32, 0, 0);
                        let error = region
                            .set_block_state(outside_radius, BlockState::air())
                            .unwrap_err();
                        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
                    }
                    ChunkStatus::Spawn => {
                        let mut entity = CompoundTag::new();
                        entity.put_string("id", "minecraft:pig");
                        region.add_entity(entity)?;
                    }
                    _ => {}
                }
                Ok(())
            }
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = TestDirectory(std::env::temp_dir().join(format!(
            "leaf-generate-stages-{}-{nonce}",
            std::process::id(),
        )));
        let storage = RegionFileStorage::new(&directory.0);
        let center = ChunkPos::new(-1, 0);
        let source = BlockPos::new(-16, -60, 0);
        let mut tag = CompoundTag::new();
        utils::add_current_data_version(&mut tag);
        tag.put_int("xPos", center.x);
        tag.put_int("zPos", center.z);
        tag.put_string("Status", "minecraft:empty");
        let ticks = PackedTicks {
            blocks: vec![SavedTick::new(
                BlockTickType {
                    id: "minecraft:observer".into(),
                },
                source,
                4,
                TickPriority::Normal,
            )],
            fluids: Vec::new(),
        };
        ticks.write_nbt(&mut tag);
        storage.write(center, &tag).unwrap();

        let mut cache = ServerChunkCache::new(storage);
        let height = SimpleLevelHeightAccessor::create(-64, 16);
        let context = ChunkLoadingContext {
            height: &height,
            palettes: &Palettes,
            data_fixer: None,
            has_sky_light: false,
        };
        let request = ChunkGenerationTask {
            center,
            target_status: ChunkStatus::Full,
        };
        let mut generator = Generator {
            center,
            source,
            fail_noise: true,
            failures: 0,
        };
        let error = cache
            .generate_through(request, &context, &mut generator)
            .err()
            .unwrap();
        assert_eq!(error.to_string(), "fixture noise failure");
        assert_eq!(cache.completed_status(center), Some(ChunkStatus::Biomes));
        assert!(cache
            .get_loaded(center)
            .unwrap()
            .chunk
            .access()
            .get_block_state(&source)
            .is_air());

        let loaded = cache
            .generate_through(request, &context, &mut generator)
            .unwrap();
        assert_eq!(generator.failures, 1);
        let RuntimeChunk::Full(chunk) = &loaded.chunk else {
            panic!("expected proto chunk to be promoted");
        };
        assert!(chunk.chunk_access.is_light_correct);
        assert!(chunk
            .get_block_state(&source)
            .is_block_id("minecraft:torch"));
        assert_eq!(
            chunk.get_height(HeightmapType::WorldSurface, source.get_x(), source.get_z()),
            -59
        );
        assert_eq!(loaded.original_tag.get_list_or_empty("entities").len(), 1);
        assert_eq!(cache.completed_status(center), Some(ChunkStatus::Full));

        assert!(cache.start_ticking(center, 20));
        let mut executed = Vec::new();
        cache
            .block_ticks()
            .tick(24, 4, &mut |pos, _| executed.push(pos));
        assert_eq!(executed, [source]);

        let registries = RegistryAccess::new();
        assert!(cache.unload(center, 25, &registries).unwrap());
        let loaded = cache.load_existing(center, &context).unwrap().unwrap();
        assert!(matches!(loaded.chunk, RuntimeChunk::Full(_)));
        assert_eq!(loaded.original_tag.get_list_or_empty("entities").len(), 1);
        let biome = loaded.chunk.access().sections[0].biomes.get(0, 1, 0);
        assert_eq!(biome.key().unwrap().id(), &Identifier::minecraft("forest"));
    }
}
