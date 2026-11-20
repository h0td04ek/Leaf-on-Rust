//! Перенос `net.minecraft.world.level.levelgen.structure.Structure`
//! — структура генерации: settings, GenerationContext/Stub,
//! findValidGenerationPoint, generate, corner-height helpers.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::entity::mob_category::MobCategory;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::generation_step::GenerationStep;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::pieces_container::PiecesContainer;
use crate::world::level::levelgen::structure::structure_piece::StructurePiece;
use crate::world::level::levelgen::structure::structure_start::StructureStart;
use crate::world::level::levelgen::structure::structure_spawn_override::StructureSpawnOverride;
use crate::world::level::levelgen::structure::structure_type::StructureType;
use crate::world::level::levelgen::structure::terrain_adjustment::TerrainAdjustment;
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;
use crate::core::holder_set::HolderSet;
use crate::world::level::biome::biome::Biome;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock, RwLock};

/// `Structure.GenerationStub` — точка генерации + генератор
/// кусков.
pub struct GenerationStub {
    /// `position`.
    pub position: BlockPos,
    /// `generator` — заполняет `StructurePiecesBuilder`.
    pub generator: Box<dyn FnOnce(&mut Vec<Box<dyn StructurePiece>>)>,
}

impl GenerationStub {
    /// `new(position, generator)`.
    pub fn new(
        position: BlockPos,
        generator: impl FnOnce(&mut Vec<Box<dyn StructurePiece>>) + 'static,
    ) -> Self {
        Self {
            position,
            generator: Box::new(generator),
        }
    }
}

/// `Structure.GenerationContext` — контекст поиска точки
/// генерации. `level` покрывает heightAccessor/biomeSource
/// (через LevelReader-хуки).
pub struct GenerationContext<'a> {
    /// `seed`.
    pub seed: i64,
    /// `chunkPos`.
    pub chunk_pos: ChunkPos,
    /// `random` — WorldgenRandom с largeFeatureSeed.
    pub random: WorldgenRandom,
    /// `heightAccessor` + `biomeSource` — через LevelReader.
    pub level: &'a dyn LevelReader,
    /// `validBiome` — предикат допустимого биома.
    pub valid_biome: Box<dyn Fn(&Biome) -> bool + 'a>,
    /// `structureTemplateManager` — шаблоны структур
    /// (из LevelReader-хука; `None` если уровень их не даёт).
    pub template_manager:
        Option<std::sync::Arc<crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager>>,
}

impl<'a> GenerationContext<'a> {
    /// `makeRandom` — WorldgenRandom(seed, chunkPos) с
    /// `setLargeFeatureSeed` (Leaf SecureSeed/FasterRandom —
    /// опциональные модули, базовый путь ванильный).
    pub fn make_random(seed: i64, chunk_pos: ChunkPos) -> WorldgenRandom {
        let mut random = WorldgenRandom::new(Box::new(crate::random::legacy::LegacyRandomSource::new(0)));
        random.set_large_feature_seed(seed, chunk_pos.x, chunk_pos.z);
        random
    }

    /// `new` — полный конструктор (random через makeRandom).
    pub fn new(
        seed: i64,
        chunk_pos: ChunkPos,
        level: &'a dyn LevelReader,
        valid_biome: Box<dyn Fn(&Biome) -> bool + 'a>,
    ) -> Self {
        Self {
            seed,
            chunk_pos,
            random: Self::make_random(seed, chunk_pos),
            level,
            valid_biome,
            template_manager: level.structure_template_manager(),
        }
    }
}

/// `Structure.StructureSettings` — настройки структуры.
#[derive(Clone)]
pub struct StructureSettings {
    /// `biomes`.
    pub biomes: HolderSet<Biome>,
    /// `spawnOverrides`.
    pub spawn_overrides: HashMap<MobCategory, StructureSpawnOverride>,
    /// `step`.
    pub step: GenerationStep,
    /// `terrainAdaptation`.
    pub terrain_adaptation: TerrainAdjustment,
}

impl Default for StructureSettings {
    fn default() -> Self {
        Self {
            biomes: HolderSet::Direct(Vec::new()),
            spawn_overrides: HashMap::new(),
            step: GenerationStep::SurfaceStructures,
            terrain_adaptation: TerrainAdjustment::None,
        }
    }
}

/// `StructureSettings.Builder`.
pub struct StructureSettingsBuilder {
    biomes: HolderSet<Biome>,
    spawn_overrides: HashMap<MobCategory, StructureSpawnOverride>,
    step: GenerationStep,
    terrain_adaptation: TerrainAdjustment,
}

impl StructureSettingsBuilder {
    /// `new(biomes)`.
    pub fn new(biomes: HolderSet<Biome>) -> Self {
        Self {
            biomes,
            spawn_overrides: StructureSettings::default().spawn_overrides,
            step: GenerationStep::SurfaceStructures,
            terrain_adaptation: TerrainAdjustment::None,
        }
    }

    /// `spawnOverrides`.
    pub fn spawn_overrides(mut self, overrides: HashMap<MobCategory, StructureSpawnOverride>) -> Self {
        self.spawn_overrides = overrides;
        self
    }

    /// `generationStep`.
    pub fn generation_step(mut self, step: GenerationStep) -> Self {
        self.step = step;
        self
    }

    /// `terrainAdaptation`.
    pub fn terrain_adaptation(mut self, adaptation: TerrainAdjustment) -> Self {
        self.terrain_adaptation = adaptation;
        self
    }

    /// `build`.
    pub fn build(self) -> StructureSettings {
        StructureSettings {
            biomes: self.biomes,
            spawn_overrides: self.spawn_overrides,
            step: self.step,
            terrain_adaptation: self.terrain_adaptation,
        }
    }
}

/// `Structure` — база структур. Конкретные структуры реализуют
/// `find_generation_point` через `generator`-поле (аналог
/// abstract-метода Java).
pub struct Structure {
    /// `id` — identifier структуры.
    pub id: Identifier,
    /// `settings`.
    pub settings: StructureSettings,
    /// `type` — `StructureType` (id типа).
    pub structure_type: Arc<StructureType>,
    /// `findGenerationPoint` — реализация наследника.
    /// `None` → `Optional.empty()` (нет валидной точки).
    pub find_generation_point:
        Option<Arc<dyn Fn(&mut GenerationContext) -> Option<GenerationStub> + Send + Sync>>,
    /// `afterPlace` — пост-обработка после размещения кусков.
    pub after_place:
        Option<Arc<dyn Fn(&dyn LevelReader, &dyn crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess, &mut dyn RandomSource, &BoundingBox, ChunkPos, &PiecesContainer) + Send + Sync>>,
}

impl Clone for Structure {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            settings: self.settings.clone(),
            structure_type: self.structure_type.clone(),
            find_generation_point: self.find_generation_point.clone(),
            after_place: self.after_place.clone(),
        }
    }
}

impl Structure {
    /// `new(id, settings, type)` — базовый конструктор.
    pub fn new(id: Identifier, settings: StructureSettings, structure_type: Arc<StructureType>) -> Self {
        Self {
            id,
            settings,
            structure_type,
            find_generation_point: None,
            after_place: None,
        }
    }

    /// `biomes()`.
    pub fn biomes(&self) -> &HolderSet<Biome> {
        &self.settings.biomes
    }

    /// `spawnOverrides()`.
    pub fn spawn_overrides(&self) -> &HashMap<MobCategory, StructureSpawnOverride> {
        &self.settings.spawn_overrides
    }

    /// `step()`.
    pub fn step(&self) -> GenerationStep {
        self.settings.step
    }

    /// `terrainAdaptation()`.
    pub fn terrain_adaptation(&self) -> TerrainAdjustment {
        self.settings.terrain_adaptation
    }

    /// `adjustBoundingBox` — inflate(12) при terrainAdaptation != NONE.
    pub fn adjust_bounding_box(&self, bounding_box: BoundingBox) -> BoundingBox {
        if self.terrain_adaptation() != TerrainAdjustment::None {
            bounding_box.inflated_by(12)
        } else {
            bounding_box
        }
    }

    /// `generate` — полный цикл: findValidGenerationPoint →
    /// build pieces → StructureStart.
    pub fn generate(
        &self,
        chunk_pos: ChunkPos,
        references: i32,
        level: &dyn LevelReader,
        valid_biome: Box<dyn Fn(&Biome) -> bool + '_>,
    ) -> StructureStart {
        let mut context = GenerationContext::new(self.generate_seed(level), chunk_pos, level, valid_biome);
        let mut pieces: Vec<Box<dyn StructurePiece>> = Vec::new();
        let mut found = false;
        if let Some(finder) = &self.find_generation_point {
            if let Some(stub) = finder(&mut context) {
                (stub.generator)(&mut pieces);
                found = true;
            }
        }
        if !found {
            return StructureStart::invalid_start();
        }
        let container = PiecesContainer::new(pieces);
        let start = StructureStart::new(
            Some(Arc::new(Structure {
                id: self.id.clone(),
                settings: self.settings.clone(),
                structure_type: self.structure_type.clone(),
                find_generation_point: self.find_generation_point.clone(),
                after_place: self.after_place.clone(),
            })),
            chunk_pos,
            references,
            container,
        );
        if start.is_valid() {
            start
        } else {
            StructureStart::invalid_start()
        }
    }

    /// `generateSeed` — сид уровня из level-хука
    /// (`world_seed`); default 0.
    fn generate_seed(&self, level: &dyn LevelReader) -> i64 {
        level.world_seed()
    }

    /// `afterPlace` — делегирует в hook.
    pub fn after_place(
        &self,
        level: &dyn LevelReader,
        structure_manager: &dyn crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess,
        random: &mut dyn RandomSource,
        bounding_box: &BoundingBox,
        chunk_pos: ChunkPos,
        pieces: &PiecesContainer,
    ) {
        if let Some(f) = &self.after_place {
            f(level, structure_manager, random, bounding_box, chunk_pos, pieces);
        }
    }

    /// `findValidGenerationPoint` — findGenerationPoint +
    /// biome-фильтр (`isValidBiome`).
    pub fn find_valid_generation_point<'a>(
        &self,
        context: &mut GenerationContext<'a>,
    ) -> Option<GenerationStub> {
        let finder = self.find_generation_point.as_ref()?;
        let stub = finder(context)?;
        if Self::is_valid_biome(&stub, context) {
            Some(stub)
        } else {
            None
        }
    }

    /// `isValidBiome` — biome в позиции stub'а проходит
    /// `validBiome` предикат.
    fn is_valid_biome(stub: &GenerationStub, context: &GenerationContext) -> bool {
        let pos = stub.position;
        let biome = context
            .level
            .biome_at(pos)
            .and_then(|h| h.value().cloned());
        biome.map(|b| (context.valid_biome)(&b)).unwrap_or(false)
    }
}

/// `onTopOfChunkCenter` — stub на высоте heightmap в центре
/// чанка.
pub fn on_top_of_chunk_center(
    context: &mut GenerationContext,
    heightmap_types: HeightmapType,
    generator: impl FnOnce(&mut Vec<Box<dyn StructurePiece>>) + 'static,
) -> GenerationStub {
    let chunk_pos = context.chunk_pos;
    let middle_block_x = chunk_pos.get_middle_block_x();
    let middle_block_z = chunk_pos.get_middle_block_z();
    let first_occupied_height = context.level.get_height(
        heightmap_types,
        middle_block_x,
        middle_block_z,
    );
    GenerationStub::new(
        BlockPos::new(middle_block_x, first_occupied_height, middle_block_z),
        generator,
    )
}

/// `getCornerHeights` — 4 угла чанка на WORLD_SURFACE_WG.
fn get_corner_heights(
    context: &GenerationContext,
    min_x: i32,
    max_x: i32,
    min_z: i32,
    max_z: i32,
) -> [i32; 4] {
    [
        context.level.get_height(HeightmapType::WorldSurfaceWg, min_x, min_z),
        context.level.get_height(HeightmapType::WorldSurfaceWg, min_x, min_z + max_z),
        context.level.get_height(HeightmapType::WorldSurfaceWg, min_x + max_x, min_z),
        context.level.get_height(HeightmapType::WorldSurfaceWg, min_x + max_x, min_z + max_z),
    ]
}

/// `getMeanFirstOccupiedHeight`.
pub fn get_mean_first_occupied_height(
    context: &GenerationContext,
    min_x: i32,
    max_x: i32,
    min_z: i32,
    max_z: i32,
) -> i32 {
    let corner_heights = get_corner_heights(context, min_x, max_x, min_z, max_z);
    (corner_heights[0] + corner_heights[1] + corner_heights[2] + corner_heights[3]) / 4
}

/// `getLowestY` — минимальная высота углов.
pub fn get_lowest_y(context: &GenerationContext, max_x: i32, max_z: i32) -> i32 {
    let chunk_pos = context.chunk_pos;
    get_lowest_y_explicit(
        context,
        chunk_pos.get_min_block_x(),
        chunk_pos.get_min_block_z(),
        max_x,
        max_z,
    )
}

/// `getLowestY(minX, minZ, maxX, maxZ)`.
pub fn get_lowest_y_explicit(
    context: &GenerationContext,
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
) -> i32 {
    let corner_heights = get_corner_heights(context, min_x, max_x, min_z, max_z);
    corner_heights
        .iter()
        .min()
        .copied()
        .unwrap_or(context.level.min_build_height())
}

/// `getLowestYIn5by5BoxOffset7Blocks` — deprecated-вариант для
/// SinglePieceStructure-подобных структур.
pub fn get_lowest_y_in_5by5_box_offset_7_blocks(
    context: &GenerationContext,
    rotation: Rotation,
) -> BlockPos {
    let (i, i1) = match rotation {
        Rotation::CLOCKWISE_90 => (-5, 5),
        Rotation::CLOCKWISE_180 => (-5, -5),
        Rotation::COUNTERCLOCKWISE_90 => (5, -5),
        Rotation::NONE => (5, 5),
    };
    let chunk_pos = context.chunk_pos;
    let block_x = chunk_pos.get_block_x(0, 7);
    let block_z = chunk_pos.get_block_z(0, 7);
    let y = get_lowest_y_explicit(context, block_x, block_z, i, i1);
    BlockPos::new(block_x, y, block_z)
}

/// `Structure` — реестр структур: id → Arc<Structure>.
pub fn structure_registry() -> &'static RwLock<Vec<Arc<Structure>>> {
    static REG: OnceLock<RwLock<Vec<Arc<Structure>>>> = OnceLock::new();
    REG.get_or_init(|| RwLock::new(Vec::new()))
}

/// Регистрация структуры.
pub fn register_structure(structure: Arc<Structure>) {
    let mut reg = structure_registry().write().unwrap();
    if let Some(pos) = reg.iter().position(|s| s.id == structure.id) {
        reg[pos] = structure;
    } else {
        reg.push(structure);
    }
}

/// `byIdentifier`.
pub fn structure_by_identifier(id: &Identifier) -> Option<Structure> {
    structure_registry()
        .read()
        .unwrap()
        .iter()
        .find(|s| &s.id == id)
        .map(|s| Structure {
            id: s.id.clone(),
            settings: s.settings.clone(),
            structure_type: s.structure_type.clone(),
            find_generation_point: s.find_generation_point.clone(),
            after_place: s.after_place.clone(),
        })
}

/// `allStructures`.
pub fn all_structures() -> Vec<Arc<Structure>> {
    structure_registry().read().unwrap().clone()
}
