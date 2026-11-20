//! Перенос `net.minecraft.world.level.levelgen.structure.StructureManager`
//! — доступ к стартам/частям/референсам структур из LevelReader.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::resources::identifier::Identifier;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::structure::Structure;
use crate::world::level::levelgen::structure::structure_start::{StructureCheckResult, StructureStart};
use std::collections::HashMap;
use std::sync::Arc;

/// `StructureManagerAccess` — интерфейс доступа к менеджеру
/// структур (аналог Java `StructureManager`, передаваемого в
/// `postProcess` кусков).
pub trait StructureManagerAccess {
    /// `getStructureAt(pos, structure)` — start в позиции.
    fn get_structure_at(
        &self,
        pos: BlockPos,
        structure: &Arc<Structure>,
    ) -> Option<StructureStart>;

    /// `getStructureWithPieceAt(pos, predicate)` — start, чья
    /// часть содержит позицию.
    fn get_structure_with_piece_at(
        &self,
        pos: BlockPos,
        predicate: &dyn Fn(&Arc<Structure>) -> bool,
    ) -> Option<StructureStart> {
        let _ = (pos, predicate);
        None
    }
}

/// `StructureManager` — менеджер структур уровня.
///
/// Хранит `starts` по чанкам (chunkPos packed → StructureStart)
/// и `references` (structure id → позиции-референсы).
pub struct StructureManager {
    /// `starts` — packed chunk pos → StructureStart.
    pub starts: HashMap<i64, StructureStart>,
    /// `references` — id структуры → список BlockPos-референсов.
    pub references: HashMap<Identifier, Vec<BlockPos>>,
    /// `possibleStructureSets` — фильтр по StructureSet.
    possible_structure_sets: Option<Arc<dyn Fn(&Identifier) -> bool + Send + Sync>>,
}

impl StructureManager {
    /// `new(possibleStructureSets)`.
    pub fn new(
        possible_structure_sets: Option<Arc<dyn Fn(&Identifier) -> bool + Send + Sync>>,
    ) -> Self {
        Self {
            starts: HashMap::new(),
            references: HashMap::new(),
            possible_structure_sets,
        }
    }

    /// `fillStartsForChunk(chunkPos, startsConsumer)`.
    pub fn fill_starts_for_chunk(&self, chunk_pos: ChunkPos, out: &mut Vec<StructureStart>) {
        if let Some(start) = self.starts.get(&chunk_pos.as_long()) {
            if start.is_valid() {
                out.push(start.shallow_clone());
            }
        }
    }

    /// `getStartForStructure(structure, chunkPos)`.
    pub fn get_start_for_structure(
        &self,
        structure: &Arc<Structure>,
        chunk_pos: ChunkPos,
    ) -> Option<StructureStart> {
        let start = self.starts.get(&chunk_pos.as_long())?;
        if start.is_valid()
            && start
                .get_structure()
                .map(|s| s.id == structure.id)
                .unwrap_or(false)
        {
            Some(start.shallow_clone())
        } else {
            None
        }
    }

    /// `startsForStructure(chunkPos, structure)`.
    pub fn starts_for_structure(
        &self,
        chunk_pos: ChunkPos,
        structure: &Arc<Structure>,
    ) -> Vec<StructureStart> {
        self.get_start_for_structure(structure, chunk_pos)
            .into_iter()
            .collect()
    }

    /// `getStructureWithPieceAt(pos, predicate)`.
    pub fn get_structure_with_piece_at(
        &self,
        pos: BlockPos,
        predicate: &dyn Fn(&Arc<Structure>) -> bool,
    ) -> Option<StructureStart> {
        let _ = (pos, predicate);
        None
    }

    /// `getAllStructuresAt(pos)`.
    pub fn get_all_structures_at(&self, pos: BlockPos) -> Vec<StructureStart> {
        let _ = pos;
        Vec::new()
    }

    /// `checkStructurePresence(chunkPos, structure, skipKnownStarts)`.
    pub fn check_structure_presence(
        &self,
        level: &dyn LevelReader,
        chunk_pos: ChunkPos,
        structure: &Arc<Structure>,
        skip_known_starts: bool,
    ) -> StructureCheckResult {
        let packed = chunk_pos.as_long();
        if !skip_known_starts {
            if let Some(start) = self.starts.get(&packed) {
                if start.is_valid()
                    && start
                        .get_structure()
                        .map(|s| s.id == structure.id)
                        .unwrap_or(false)
                {
                    return StructureCheckResult::StartPresent;
                }
            }
        }
        if let Some(refs) = self.references.get(&structure.id) {
            for pos in refs {
                let ref_chunk = ChunkPos::new(pos.x >> 4, pos.z >> 4);
                if ref_chunk.as_long() == packed {
                    return StructureCheckResult::StartPresent;
                }
            }
        }
        if !level.is_chunk_loaded(chunk_pos) {
            return StructureCheckResult::ChunkLoadNeeded;
        }
        StructureCheckResult::StartNotPresent
    }

    /// `addReference(structure, pos)`.
    pub fn add_reference(&mut self, structure: &Arc<Structure>, pos: BlockPos) {
        self.references
            .entry(structure.id.clone())
            .or_default()
            .push(pos);
    }

    /// `getAllReferences`.
    pub fn get_all_references(&self) -> &HashMap<Identifier, Vec<BlockPos>> {
        &self.references
    }

    /// `setAllReferences`.
    pub fn set_all_references(&mut self, references: HashMap<Identifier, Vec<BlockPos>>) {
        self.references = references;
    }

    /// `getStructureAt(pos, structure)`.
    pub fn get_structure_at(
        &self,
        pos: BlockPos,
        structure: &Arc<Structure>,
    ) -> Option<StructureStart> {
        let chunk_pos = ChunkPos::new(pos.x >> 4, pos.z >> 4);
        self.get_start_for_structure(structure, chunk_pos)
    }

    /// `possibleStructureSet` — фильтр StructureSet.
    pub fn is_possible_structure_set(&self, id: &Identifier) -> bool {
        self.possible_structure_sets
            .as_ref()
            .map(|f| f(id))
            .unwrap_or(true)
    }

    /// `save` → NBT (`StructureFeatureIndex`-совместимый вид).
    pub fn save(&self) -> CompoundTag {
        let mut tag = CompoundTag::new();
        let mut children = Vec::new();
        for start in self.starts.values() {
            if !start.is_valid() {
                continue;
            }
            children.push(Tag::Compound(start.create_tag(start.chunk_pos)));
        }
        tag.put("Children", Tag::List(children));
        tag
    }

    /// `load` — обратная операция.
    pub fn load(tag: &CompoundTag) -> Self {
        let mut manager = Self::new(None);
        if let Some(children) = tag.get_list("Children") {
            for item in children {
                if let Some(entry) = item.as_compound() {
                    let cx = entry.get_int("ChunkX").unwrap_or(0);
                    let cz = entry.get_int("ChunkZ").unwrap_or(0);
                    let chunk_pos = ChunkPos::new(cx, cz);
                    if let Some(start) = StructureStart::load_static_start(entry, 0) {
                        manager
                            .starts
                            .insert(chunk_pos.as_long(), start);
                    }
                }
            }
        }
        manager
    }
}

impl StructureManagerAccess for StructureManager {
    fn get_structure_at(
        &self,
        pos: BlockPos,
        structure: &Arc<Structure>,
    ) -> Option<StructureStart> {
        StructureManager::get_structure_at(self, pos, structure)
    }
}
