//! Перенос `net.minecraft.world.level.levelgen.structure.StructureCheck`
//! — проверка наличия структуры в чанке через chunk-storage scan
//! + placement restrictions + feature-check кэш.
//!
//! Зависимости Java (ChunkScanAccess/DataFixer/SimpleRegionStorage)
//! заменены трейтами `ChunkScanAccess`/`StructurePlacementAccess` —
//! реальная реализация подключится с chunk-storage слоем.

use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::levelgen::structure::structure::Structure;
use crate::world::level::levelgen::structure::structure_check_result::StructureCheckResult;
use crate::world::level::levelgen::structure::structure_start::StructureStart;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// `ChunkScanAccess` — скан сохранённых данных чанка
/// (аналог `net.minecraft.world.level.chunk.storage.ChunkScanAccess`).
/// Возвращает `None` — чанк/данные недоступны.
pub trait ChunkScanAccess {
    /// `scanChunk` — читает NBT-фрагмент чанка (structures.starts).
    /// Возвращает `Option<CompoundTag>` — `structures` compound.
    fn scan_chunk_structures(
        &self,
        chunk_pos: ChunkPos,
    ) -> Option<crate::nbt::compound::CompoundTag>;
}

/// `StructurePlacementAccess` — placement-ограничения
/// (`StructurePlacement.applyAdditionalChunkRestrictions` +
/// `isStructureChunk`).
pub trait StructurePlacementAccess {
    /// `applyAdditionalChunkRestrictions(chunkX, chunkZ, seed, salt)`.
    fn apply_additional_chunk_restrictions(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        seed: i64,
        salt_override: Option<i32>,
    ) -> bool;

    /// `getPotentialStructureChunk`/`isStructureChunk` — чанк
    /// является потенциальным чанком структуры.
    fn is_structure_chunk(&self, chunk_x: i32, chunk_z: i32, seed: i64, salt: i32) -> bool;
}

/// `StructureCheck` — менеджер проверки структуры в чанке.
pub struct StructureCheck {
    /// `storageAccess`.
    pub storage_access: Box<dyn ChunkScanAccess + Send + Sync>,
    /// `placement` — StructurePlacement конкретной структуры.
    pub placement: Box<dyn StructurePlacementAccess + Send + Sync>,
    /// `seed`.
    pub seed: i64,
    /// `loadedChunks` — chunkPos packed → (structure id → references).
    loaded_chunks: Mutex<HashMap<i64, HashMap<String, i32>>>,
    /// `featureChecks` — structure id → (packed → canCreate).
    feature_checks: Mutex<HashMap<String, HashMap<i64, bool>>>,
    /// `saltOverride` — Paper-специфичный сид-соль (mineshaft и т.п.).
    pub salt_override: Option<i32>,
}

impl StructureCheck {
    /// `new(storageAccess, placement, seed, saltOverride)`.
    pub fn new(
        storage_access: Box<dyn ChunkScanAccess + Send + Sync>,
        placement: Box<dyn StructurePlacementAccess + Send + Sync>,
        seed: i64,
        salt_override: Option<i32>,
    ) -> Self {
        Self {
            storage_access,
            placement,
            seed,
            loaded_chunks: Mutex::new(HashMap::new()),
            feature_checks: Mutex::new(HashMap::new()),
            salt_override,
        }
    }

    /// `checkStart(chunkPos, structure, skipKnownStructures)`.
    pub fn check_start(
        &self,
        chunk_pos: ChunkPos,
        structure: &Arc<Structure>,
        skip_known_structures: bool,
        can_create: &dyn Fn(&ChunkPos, &Arc<Structure>) -> bool,
    ) -> StructureCheckResult {
        let packed = chunk_pos.as_long();
        {
            let loaded = self.loaded_chunks.lock().unwrap();
            if let Some(map) = loaded.get(&packed) {
                return Self::check_structure_info(map, structure, skip_known_structures);
            }
        }
        if let Some(result) = self.try_load_from_storage(chunk_pos, structure, skip_known_structures, packed) {
            return result;
        }
        if !self.placement.apply_additional_chunk_restrictions(
            chunk_pos.x,
            chunk_pos.z,
            self.seed,
            self.salt_override,
        ) {
            return StructureCheckResult::StartNotPresent;
        }
        let can_create = {
            let mut feature = self.feature_checks.lock().unwrap();
            let entry = feature.entry(structure.id.to_string()).or_default();
            *entry.entry(packed).or_insert_with(|| can_create(&chunk_pos, structure))
        };
        if !can_create {
            StructureCheckResult::StartNotPresent
        } else {
            StructureCheckResult::ChunkLoadNeeded
        }
    }

    /// `tryLoadFromStorage` — читает сохранённые `starts` из
    /// chunk NBT, кэширует, проверяет.
    fn try_load_from_storage(
        &self,
        chunk_pos: ChunkPos,
        structure: &Arc<Structure>,
        skip_known_structures: bool,
        packed: i64,
    ) -> Option<StructureCheckResult> {
        let tag = self.storage_access.scan_chunk_structures(chunk_pos)?;
        let map = Self::load_structures(&tag, self)?;
        self.store_full_results(packed, map.clone());
        Some(Self::check_structure_info(&map, structure, skip_known_structures))
    }

    /// `loadStructures` — парсит `structures.starts` compound:
    /// id → StructureStart → references count.
    fn load_structures(
        tag: &crate::nbt::compound::CompoundTag,
        _check: &StructureCheck,
    ) -> Option<HashMap<String, i32>> {
        let structures = tag.get_compound("structures")?;
        let starts = structures.get_compound("starts")?;
        let mut map = HashMap::new();
        let keys: Vec<String> = starts.keys().cloned().collect();
        for key in keys {
            let Some(value) = starts.get(&key) else { continue };
            let Some(start_tag) = value.as_compound() else { continue };
            let id_str = start_tag.get_string("id").unwrap_or("");
            if id_str == "INVALID" || id_str.is_empty() {
                continue;
            }
            // references из тега start'а; 0 если отсутствует.
            let references = start_tag.get_int("references").unwrap_or(0);
            map.insert(key.clone(), references);
        }
        Some(map)
    }

    /// `checkStructureInfo` — references == -1 или (skipKnown &&
    /// references != 0) → StartNotPresent; иначе StartPresent.
    fn check_structure_info(
        structure_chunks: &HashMap<String, i32>,
        structure: &Arc<Structure>,
        skip_known_structures: bool,
    ) -> StructureCheckResult {
        let references = structure_chunks
            .get(&structure.id.to_string())
            .copied()
            .unwrap_or(-1);
        if references == -1 || (skip_known_structures && references != 0) {
            StructureCheckResult::StartNotPresent
        } else {
            StructureCheckResult::StartPresent
        }
    }

    /// `onStructureLoad` — кэширует starts при загрузке чанка.
    pub fn on_structure_load(&self, chunk_pos: ChunkPos, starts: &HashMap<String, i32>) {
        let mut loaded = self.loaded_chunks.lock().unwrap();
        loaded.insert(chunk_pos.as_long(), starts.clone());
    }

    /// `incrementReference` — увеличивает references счётчик.
    pub fn increment_reference(&self, chunk_pos: ChunkPos, structure: &Arc<Structure>) {
        let mut loaded = self.loaded_chunks.lock().unwrap();
        let map = loaded.entry(chunk_pos.as_long()).or_default();
        *map.entry(structure.id.to_string()).or_insert(0) += 1;
    }

    /// `storeFullResults` — кэш + инвалидация featureChecks
    /// для этого чанка.
    fn store_full_results(&self, packed: i64, map: HashMap<String, i32>) {
        {
            let mut loaded = self.loaded_chunks.lock().unwrap();
            loaded.insert(packed, map);
        }
        let mut feature = self.feature_checks.lock().unwrap();
        for checks in feature.values_mut() {
            checks.remove(&packed);
        }
    }
}
