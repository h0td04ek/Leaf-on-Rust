//! Перенос `net.minecraft.world.level.chunk.ChunkAccess` — базовый класс чанка.

use std::collections::HashMap;
use std::sync::Arc;

use crate::core::block_pos::BlockPos;
use crate::core::section_pos::SectionPos;
use crate::nbt::compound::CompoundTag;
use crate::world::level::biome::biome::Biome;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::block::entity::block_entity::BlockEntityAccess;
use crate::world::level::chunk::level_chunk::ChunkStatus;
use crate::world::level::chunk::level_chunk_section::LevelChunkSection;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_height_accessor::LevelHeightAccessor;
use crate::world::level::levelgen::heightmap::{Heightmap, HeightmapType};

/// `ChunkAccess` — базовый класс для всех типов чанков.
pub struct ChunkAccess {
    /// `chunkPos` — позиция чанка.
    pub chunk_pos: ChunkPos,
    /// `coordinateKey` — packed coordinate.
    pub coordinate_key: i64,
    /// `locX` — X координата.
    pub loc_x: i32,
    /// `locZ` — Z координата.
    pub loc_z: i32,
    /// `inhabitedTime` — время обитания.
    pub inhabited_time: i64,
    /// `unsaved` — флаг несохранённых изменений.
    pub unsaved: bool,
    /// `isLightCorrect` — флаг корректности освещения.
    pub is_light_correct: bool,
    /// `sections` — массив секций чанка.
    pub sections: Vec<LevelChunkSection>,
    /// `heightmaps` — heightmap данные.
    pub heightmaps: HashMap<HeightmapType, Heightmap>,
    /// `blockEntities` — block entities.
    pub block_entities: HashMap<BlockPos, Arc<dyn BlockEntityAccess>>,
    /// `pendingBlockEntities` — ожидающие block entities.
    pub pending_block_entities: HashMap<BlockPos, CompoundTag>,
    /// `levelHeightAccessor` — доступ к высоте уровня.
    pub level_height_accessor: Box<dyn LevelHeightAccessor>,
    /// `upgradeData` — данные обновления.
    pub upgrade_data: Option<UpgradeData>,
    /// `blendingData` — данные смешивания.
    pub blending_data: Option<BlendingData>,
    /// `structureStarts` — structure starts.
    pub structure_starts: HashMap<String, StructureStart>,
    /// `structuresRefences` — structure references.
    pub structures_references: HashMap<String, Vec<i64>>,
    /// `blockNibbles` — StarLight block-light нибблы (Paper-патч).
    pub starlight_block_nibbles: std::sync::Mutex<
        Vec<Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::NibbleArrayRef>>,
    >,
    /// `skyNibbles` — StarLight sky-light нибблы.
    pub starlight_sky_nibbles: std::sync::Mutex<
        Vec<Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::NibbleArrayRef>>,
    >,
    /// `skyEmptinessMap` — карта пустых sky-секций.
    pub starlight_sky_emptiness_map: std::sync::Mutex<
        Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::EmptinessMapRef>,
    >,
    /// `blockEmptinessMap` — карта пустых block-секций.
    pub starlight_block_emptiness_map: std::sync::Mutex<
        Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::EmptinessMapRef>,
    >,
    /// `persistedStatus` — персистентный статус чанка
    /// (`getPersistedStatus`; LevelChunk синхронизирует со своим статусом).
    pub persisted_status: ChunkStatus,
}

/// `UpgradeData` — данные обновления чанка.
#[derive(Clone, Debug, Default)]
pub struct UpgradeData {
    /// `block_counts` — счётчики блоков.
    pub block_counts: HashMap<String, i32>,
    /// `palettes` — палитры для обновления.
    pub palettes: HashMap<String, Vec<String>>,
}

/// `BlendingData` — данные смешивания чанков.
#[derive(Clone, Debug)]
pub struct BlendingData {
    /// `min_section` — минимальная секция.
    pub min_section: i32,
    /// `max_section` — максимальная секция.
    pub max_section: i32,
    /// `blending_data` — данные смешивания.
    pub blending_data: Vec<u8>,
}

/// `StructureStart` — начало структуры.
#[derive(Clone, Debug)]
pub struct StructureStart {
    /// `structure` — тип структуры.
    pub structure: String,
    /// `chunk_pos` — позиция чанка.
    pub chunk_pos: ChunkPos,
    /// `references` — ссылки на чанки.
    pub references: Vec<i64>,
    /// `valid` — флаг валидности.
    pub valid: bool,
}

impl ChunkAccess {
    /// `new(...)` — создаёт новый ChunkAccess.
    pub fn new(
        chunk_pos: ChunkPos,
        level_height_accessor: Box<dyn LevelHeightAccessor>,
        sections: Vec<LevelChunkSection>,
    ) -> Self {
        let coordinate_key = ChunkPos::as_long_static(chunk_pos.x, chunk_pos.z);
        let light_section_count = level_height_accessor.get_sections_count() as usize + 2;
        Self {
            chunk_pos,
            coordinate_key,
            loc_x: chunk_pos.x,
            loc_z: chunk_pos.z,
            inhabited_time: 0,
            unsaved: false,
            is_light_correct: false,
            sections,
            heightmaps: HashMap::new(),
            block_entities: HashMap::new(),
            pending_block_entities: HashMap::new(),
            level_height_accessor,
            upgrade_data: None,
            blending_data: None,
            structure_starts: HashMap::new(),
            structures_references: HashMap::new(),
            starlight_block_nibbles: std::sync::Mutex::new(
                crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::get_filled_empty_light_vec(
                    light_section_count,
                ),
            ),
            starlight_sky_nibbles: std::sync::Mutex::new(
                crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::get_filled_empty_light_vec(
                    light_section_count,
                ),
            ),
            starlight_sky_emptiness_map: std::sync::Mutex::new(None),
            starlight_block_emptiness_map: std::sync::Mutex::new(None),
            persisted_status: ChunkStatus::Full,
        }
    }

    /// `getPersistedStatus`.
    pub fn get_persisted_status(&self) -> ChunkStatus {
        self.persisted_status
    }

    /// `getPos` — позиция чанка.
    pub fn get_pos(&self) -> ChunkPos {
        self.chunk_pos
    }

    /// `getMinSection` — минимальная секция.
    pub fn get_min_section(&self) -> i32 {
        self.level_height_accessor.get_min_section_y()
    }

    /// `getMaxSection` — максимальная секция.
    pub fn get_max_section(&self) -> i32 {
        self.level_height_accessor.get_max_section_y()
    }

    /// `getSection` — секция по индексу.
    pub fn get_section(&self, section: i32) -> Option<&LevelChunkSection> {
        let idx = self.get_section_index_from_section_y(section) as usize;
        self.sections.get(idx)
    }

    /// `getSection_mut` — изменяемая секция.
    pub fn get_section_mut(&mut self, section: i32) -> Option<&mut LevelChunkSection> {
        let idx = self.get_section_index_from_section_y(section) as usize;
        self.sections.get_mut(idx)
    }

    /// `getSectionYFromSectionIndex` — sectionY по 0-based индексу секции.
    pub fn get_section_y_from_section_index(&self, section_index: i32) -> i32 {
        self.level_height_accessor.get_section_y_from_section_index(section_index)
    }

    /// `getSectionIndexFromSectionY` — 0-based индекс секции по sectionY.
    pub fn get_section_index_from_section_y(&self, section_y: i32) -> i32 {
        self.level_height_accessor.get_section_index_from_section_y(section_y)
    }

    /// `getHighestFilledSectionIndex` — индекс (0-based) самой высокой
    /// непустой секции, -1 если все секции пусты.
    pub fn get_highest_filled_section_index(&self) -> i32 {
        for i in (0..self.sections.len()).rev() {
            if !self.sections[i].has_only_air() {
                return i as i32;
            }
        }
        -1
    }

    /// `findBlocks(predicate, output)` — обход всех блоков чанка,
    /// удовлетворяющих предикату.
    pub fn find_blocks(
        &self,
        predicate: &dyn Fn(&BlockState) -> bool,
        output: &mut dyn FnMut(BlockPos, &BlockState),
    ) {
        for (index, section) in self.sections.iter().enumerate() {
            let section_y = self.get_section_y_from_section_index(index as i32);
            if !section.maybe_has(predicate) {
                continue;
            }
            let origin = SectionPos::of_chunk(&self.chunk_pos, section_y).origin();
            for i in 0..16 {
                for i1 in 0..16 {
                    for i2 in 0..16 {
                        let state = section.get_block_state(i2, i, i1);
                        if predicate(&state) {
                            output(origin.offset(i2, i, i1), &state);
                        }
                    }
                }
            }
        }
    }

    /// `getBlockState` — состояние блока.
    pub fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        block_state_at(
            &self.sections,
            self.get_min_section(),
            SectionPos::section_relative(pos.get_x()),
            pos.get_y(),
            SectionPos::section_relative(pos.get_z()),
        )
    }

    /// `setBlockState` — установить состояние блока.
    pub fn set_block_state(&mut self, pos: &BlockPos, state: BlockState) -> bool {
        let section_y = SectionPos::block_to_section_coord(pos.get_y());
        let min_section = self.get_min_section();
        let section_idx = section_y - min_section;
        if section_idx < 0 || section_idx >= self.sections.len() as i32 {
            return false;
        }
        let section = &mut self.sections[section_idx as usize];
        let x = SectionPos::section_relative(pos.get_x());
        let y = SectionPos::section_relative(pos.get_y());
        let z = SectionPos::section_relative(pos.get_z());
        let (old, _) = section.set_block_state(x, y, z, state.clone());
        if old == state {
            return false;
        }
        let sections = &self.sections;
        for (&ty, heightmap) in &mut self.heightmaps {
            heightmap.update(x, pos.get_y(), z, &state, ty, |scan_y| {
                block_state_at(sections, min_section, x, scan_y, z)
            });
        }
        self.mark_unsaved();
        true
    }

    /// `getFluidState` — состояние жидкости.
    pub fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
        let state = self.get_block_state(pos);
        state.get_fluid_state()
    }

    /// `getHeight` — высота по типу heightmap.
    pub fn get_height(&self, heightmap: HeightmapType, x: i32, z: i32) -> i32 {
        if let Some(hm) = self.heightmaps.get(&heightmap) {
            hm.get_first_available(x, z)
        } else {
            self.level_height_accessor.get_min_y()
        }
    }

    /// Строит требуемые карты в один проход по каждой колонке.
    pub fn prime_heightmaps(&mut self, types: &[HeightmapType]) {
        if types.is_empty() {
            return;
        }
        let min_y = self.level_height_accessor.get_min_y();
        let min_section = self.get_min_section();
        let mut maps: Vec<_> = HeightmapType::VALUES
            .into_iter()
            .filter(|ty| types.contains(ty))
            .map(|ty| {
                let heightmap = Heightmap::new(
                    self.chunk_pos,
                    ty,
                    self.level_height_accessor.as_ref(),
                    min_y,
                );
                (ty, heightmap)
            })
            .collect();
        let all_found = (1 << maps.len()) - 1;

        for z in 0..16 {
            for x in 0..16 {
                let mut found = 0u32;
                'column: for (index, section) in self.sections.iter().enumerate().rev() {
                    if section.has_only_air() {
                        continue;
                    }
                    let section_min_y = (min_section + index as i32) << 4;
                    for y in (0..16).rev() {
                        let state = section.get_block_state(x, y, z);
                        for (map_index, (ty, heightmap)) in maps.iter_mut().enumerate() {
                            let bit = 1 << map_index;
                            if found & bit == 0 && ty.is_block_counted(&state) {
                                heightmap.set_height(x, z, section_min_y + y + 1);
                                found |= bit;
                            }
                        }
                        if found == all_found {
                            break 'column;
                        }
                    }
                }
            }
        }
        self.heightmaps.extend(maps);
    }

    /// `setHeight` — установить высоту.
    pub fn set_height(&mut self, heightmap: HeightmapType, x: i32, z: i32, height: i32) {
        let min_y = self.level_height_accessor.get_min_y();
        let chunk_pos = self.chunk_pos;
        let accessor = self.level_height_accessor.as_ref();
        self.heightmaps.entry(heightmap).or_insert_with(|| Heightmap::new(chunk_pos, heightmap, accessor, min_y));
        if let Some(hm) = self.heightmaps.get_mut(&heightmap) {
            hm.set_height(x, z, height);
        }
    }

    /// `isEmpty` — пустой ли чанк.
    pub fn is_empty(&self) -> bool {
        self.sections.iter().all(|s| s.has_only_air())
    }

    /// `getBlockEntities` — block entities.
    pub fn get_block_entities(&self) -> &HashMap<BlockPos, Arc<dyn BlockEntityAccess>> {
        &self.block_entities
    }

    /// `setBlockEntity` — установить block entity.
    pub fn set_block_entity(&mut self, pos: BlockPos, entity: Arc<dyn BlockEntityAccess>) {
        self.block_entities.insert(pos, entity);
    }

    /// `removeBlockEntity` — удалить block entity.
    pub fn remove_block_entity(&mut self, pos: &BlockPos) -> Option<Arc<dyn BlockEntityAccess>> {
        self.block_entities.remove(pos)
    }

    /// `getPendingBlockEntities` — ожидающие block entities.
    pub fn get_pending_block_entities(&self) -> &HashMap<BlockPos, CompoundTag> {
        &self.pending_block_entities
    }

    /// `setPendingBlockEntity` — установить ожидающий block entity.
    pub fn set_pending_block_entity(&mut self, pos: BlockPos, tag: CompoundTag) {
        self.pending_block_entities.insert(pos, tag);
    }

    /// `getStructureStart` — structure start.
    pub fn get_structure_start(&self, structure: &str) -> Option<&StructureStart> {
        self.structure_starts.get(structure)
    }

    /// `setStructureStart` — установить structure start.
    pub fn set_structure_start(&mut self, structure: String, start: StructureStart) {
        self.structure_starts.insert(structure, start);
    }

    /// `getStructuresRefences` — structure references.
    pub fn get_structures_references(&self, structure: &str) -> Option<&Vec<i64>> {
        self.structures_references.get(structure)
    }

    /// `setStructuresReferences` — установить structure references.
    pub fn set_structures_references(&mut self, structure: String, references: Vec<i64>) {
        self.structures_references.insert(structure, references);
    }

    /// `markUnsaved` — пометить как несохранённый.
    pub fn mark_unsaved(&mut self) {
        self.unsaved = true;
    }

    /// `isUnsaved` — проверка несохранённости.
    pub fn is_unsaved(&self) -> bool {
        self.unsaved
    }

    /// `setUnsaved` — установить флаг несохранённости.
    pub fn set_unsaved(&mut self, unsaved: bool) {
        self.unsaved = unsaved;
    }

    /// `isLightCorrect` — проверка корректности освещения.
    pub fn is_light_correct(&self) -> bool {
        self.is_light_correct
    }

    /// `setLightCorrect` — установить флаг корректности освещения.
    pub fn set_light_correct(&mut self, correct: bool) {
        self.is_light_correct = correct;
    }

    /// `getInhabitedTime` — время обитания.
    pub fn get_inhabited_time(&self) -> i64 {
        self.inhabited_time
    }

    /// `setInhabitedTime` — установить время обитания.
    pub fn set_inhabited_time(&mut self, time: i64) {
        self.inhabited_time = time;
    }

    /// `getUpgradeData` — данные обновления.
    pub fn get_upgrade_data(&self) -> Option<&UpgradeData> {
        self.upgrade_data.as_ref()
    }

    /// `setUpgradeData` — установить данные обновления.
    pub fn set_upgrade_data(&mut self, data: UpgradeData) {
        self.upgrade_data = Some(data);
    }

    /// `getBlendingData` — данные смешивания.
    pub fn get_blending_data(&self) -> Option<&BlendingData> {
        self.blending_data.as_ref()
    }

    /// `setBlendingData` — установить данные смешивания.
    pub fn set_blending_data(&mut self, data: BlendingData) {
        self.blending_data = Some(data);
    }

    /// `getNoiseChunk` — noise chunk.
    pub fn get_noise_chunk(&self) -> Option<&NoiseChunk> {
        None
    }

    /// `setNoiseChunk` — установить noise chunk.
    pub fn set_noise_chunk(&mut self, _chunk: NoiseChunk) {}

    /// `getCarverBiomeSettings` — настройки карвинга биомов.
    pub fn get_carver_biome_settings(&self) -> Option<&BiomeGenerationSettings> {
        None
    }

    /// `setCarverBiomeSettings` — установить настройки карвинга.
    pub fn set_carver_biome_settings(&mut self, _settings: BiomeGenerationSettings) {}
}

/// `NoiseChunk` — noise chunk для генерации.
#[derive(Clone, Debug)]
pub struct NoiseChunk {
    /// `noise` — noise данные.
    pub noise: Vec<f64>,
}

/// `BiomeGenerationSettings` — настройки генерации биомов.
#[derive(Clone)]
pub struct BiomeGenerationSettings {
    /// `biomes` — биомы.
    pub biomes: Vec<crate::core::holder::Holder<Biome>>,
}

impl crate::world::level::chunk::light_chunk::LightChunk for ChunkAccess {
    /// `findBlockLightSources` — `findBlocks(state -> lightEmission != 0)`.
    fn find_block_light_sources(&self, output: &mut dyn FnMut(BlockPos, &BlockState)) {
        self.find_blocks(
            &|state: &BlockState| state.get_light_emission() != 0,
            output,
        );
    }

    /// `getSkyLightSources` — `null` в Paper/Leaf.
    fn get_sky_light_sources(&self) -> Option<&crate::world::level::lighting::chunk_sky_light_sources::ChunkSkyLightSources> {
        None
    }

    /// `getBlockState(pos)` (из `BlockGetter`).
    fn get_block_state(&self, pos: &BlockPos) -> BlockState {
        ChunkAccess::get_block_state(self, pos)
    }
}

impl crate::ca::moonrise::patches::starlight::chunk::StarlightChunk for ChunkAccess {
    fn starlight_get_block_nibbles(
        &self,
    ) -> Vec<Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::NibbleArrayRef>> {
        self.starlight_block_nibbles.lock().unwrap().clone()
    }

    fn starlight_set_block_nibbles(
        &self,
        nibbles: Vec<Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::NibbleArrayRef>>,
    ) {
        *self.starlight_block_nibbles.lock().unwrap() = nibbles;
    }

    fn starlight_get_sky_nibbles(
        &self,
    ) -> Vec<Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::NibbleArrayRef>> {
        self.starlight_sky_nibbles.lock().unwrap().clone()
    }

    fn starlight_set_sky_nibbles(
        &self,
        nibbles: Vec<Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::NibbleArrayRef>>,
    ) {
        *self.starlight_sky_nibbles.lock().unwrap() = nibbles;
    }

    fn starlight_get_sky_emptiness_map(
        &self,
    ) -> Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::EmptinessMapRef> {
        self.starlight_sky_emptiness_map.lock().unwrap().clone()
    }

    fn starlight_set_sky_emptiness_map(
        &self,
        map: Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::EmptinessMapRef>,
    ) {
        *self.starlight_sky_emptiness_map.lock().unwrap() = map;
    }

    fn starlight_get_block_emptiness_map(
        &self,
    ) -> Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::EmptinessMapRef> {
        self.starlight_block_emptiness_map.lock().unwrap().clone()
    }

    fn starlight_set_block_emptiness_map(
        &self,
        map: Option<crate::ca::moonrise::patches::starlight::light::swmr_nibble_array::EmptinessMapRef>,
    ) {
        *self.starlight_block_emptiness_map.lock().unwrap() = map;
    }
}

fn block_state_at(
    sections: &[LevelChunkSection],
    min_section: i32,
    x: i32,
    y: i32,
    z: i32,
) -> BlockState {
    let section_index = SectionPos::block_to_section_coord(y) - min_section;
    sections.get(section_index as usize).map_or_else(BlockState::air, |section| {
        section.get_block_state(x, SectionPos::section_relative(y), z)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::holder::Holder;
    use crate::resources::Identifier;
    use crate::world::level::block::Block;
    use crate::world::level::chunk::level_chunk_section::{
        create_biomes_container, create_block_states_container,
    };
    use crate::world::level::level_height_accessor::SimpleLevelHeightAccessor;

    #[test]
    fn finds_blocks_in_all_sections_at_negative_world_height() {
        let height = SimpleLevelHeightAccessor::create(-64, 80);
        assert_eq!(height.get_section_index(-63), 0);
        assert_eq!(height.get_section_index_from_section_y(0), 4);
        assert_eq!(height.get_section_y_from_section_index(4), 0);

        let pos = ChunkPos::new(-3, 5);
        let sections = (0..height.get_sections_count())
            .map(|_| LevelChunkSection::new(
                create_block_states_container(BlockState::air()),
                create_biomes_container(Holder::Reference(0)),
            ))
            .collect();
        let mut chunk = ChunkAccess::new(pos, Box::new(height), sections);
        let stone = BlockState::without_properties(Arc::new(Block::new(Identifier::minecraft("stone"))));
        let positions = [
            BlockPos::new(-48, -64, 80),
            BlockPos::new(-44, -30, 89),
            BlockPos::new(-34, 15, 94),
        ];
        for position in &positions {
            assert!(chunk.set_block_state(position, stone.clone()));
        }
        let mut found = Vec::new();
        chunk.find_blocks(&|state| state.is_block_id("minecraft:stone"), &mut |pos, _| found.push(pos));
        assert_eq!(found, positions);
    }
}
