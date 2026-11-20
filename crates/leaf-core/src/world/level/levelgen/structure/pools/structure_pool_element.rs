//! Перенос `StructurePoolElement` — базовый тип элемента пула
//! (single/legacy/list/empty/feature).
//!
//! Дженератор (`ChunkGenerator`) в Java — конкретный класс; здесь —
//! шов `PoolGeneratorAccess`, реализация подключится с worldgen-слоем.

use super::structure_template_pool::Projection;
use crate::core::block_pos::BlockPos;
use crate::core::vec3i::Vec3i;
use crate::random::RandomSource;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_template::{
    JigsawBlockInfo, StructureBlockInfo, StructureTemplate,
};
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::world::level::level_reader::LevelReader;
use std::rc::Rc;
use std::sync::Arc;

/// Шов `ChunkGenerator` для pool-элементов (heightmap/биомы —
/// подключатся с генератором).
pub trait PoolGeneratorAccess: Send + Sync {
    /// `getFirstFreeHeight(heightmap, x, z)`.
    fn get_first_free_height(&self, heightmap: crate::world::level::levelgen::heightmap::HeightmapType, x: i32, z: i32, level: &dyn LevelReader) -> i32;
    /// `getFirstOccupiedHeight(heightmap, x, z)`.
    fn get_first_occupied_height(&self, heightmap: crate::world::level::levelgen::heightmap::HeightmapType, x: i32, z: i32, level: &dyn LevelReader) -> i32;
    /// `getSeaLevel`.
    fn get_sea_level(&self) -> i32;
    /// `getMinY`.
    fn get_min_y(&self) -> i32;
    /// `getMaxY`.
    fn get_max_y(&self) -> i32;
}

/// `StructurePoolElementType` — сериализованное имя типа.
pub trait StructurePoolElementType: Send + Sync {
    /// `getSerializedName`.
    fn get_serialized_name(&self) -> &'static str;
}

/// `StructurePoolElement`.
pub trait StructurePoolElement {
    /// `getSize(manager, rotation)`.
    fn get_size(&self, structure_template_manager: &StructureTemplateManager, rotation: Rotation) -> Vec3i;

    /// `getShuffledJigsawBlocks(manager, pos, rotation, random)`.
    fn get_shuffled_jigsaw_blocks(
        &self,
        structure_template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo>;

    /// `getBoundingBox(manager, pos, rotation)`.
    fn get_bounding_box(
        &self,
        structure_template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
    ) -> BoundingBox;

    /// `place(...)`.
    #[allow(clippy::too_many_arguments)]
    fn place(
        &self,
        structure_template_manager: &StructureTemplateManager,
        level: &dyn LevelReader,
        structure_manager: &dyn StructureManagerAccess,
        generator: &dyn PoolGeneratorAccess,
        offset: BlockPos,
        pos: BlockPos,
        rotation: Rotation,
        bounding_box: Option<BoundingBox>,
        random: &mut dyn RandomSource,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> bool;

    /// `getType().getSerializedName()`.
    fn get_type_name(&self) -> &'static str;

    /// `handleDataMarker` — дефолт no-op.
    fn handle_data_marker(
        &self,
        _level: &dyn LevelReader,
        _block_info: &StructureBlockInfo,
        _pos: BlockPos,
        _rotation: Rotation,
        _random: &mut dyn RandomSource,
        _bounding_box: Option<&BoundingBox>,
    ) {
    }

    /// `getProjection`.
    fn get_projection(&self) -> Projection;

    /// `getGroundLevelDelta` — дефолт 1.
    fn get_ground_level_delta(&self) -> i32 {
        1
    }

    /// `isEmpty` — для фильтрации в getMaxSize
    /// (EmptyPoolElement).
    fn is_empty_element(&self) -> bool {
        false
    }
}

/// `clone_rc`-хелпер для `Rc<dyn StructurePoolElement>`-полей
/// внутри элементов (list).
pub trait ClonePoolElement {
    /// Клонирование Rc.
    fn clone_element(&self) -> Rc<dyn StructurePoolElement>;
}

/// Разрешение `Holder<StructureProcessorList>` в список процессоров
/// (`processors.value().list()`): Direct → значение,
/// Reference/Key → поиск в thread-local реестре.
pub fn resolve_processor_list(
    holder: &crate::core::holder::Holder<crate::world::level::levelgen::structure::templatesystem::structure_processor_list::StructureProcessorList>,
) -> Vec<Rc<dyn crate::world::level::levelgen::structure::templatesystem::processors::StructureProcessor>> {
    use crate::core::holder::Holder;
    match holder {
        Holder::Direct(list) => list.processors.clone(),
        Holder::Reference(id) => {
            let name = format!("#{}", id);
            crate::world::level::levelgen::structure::templatesystem::structure_processor_list::processor_list_by_name(&name)
                .map(|l| l.processors.clone())
                .unwrap_or_default()
        }
        Holder::Key(key) => {
            crate::world::level::levelgen::structure::templatesystem::structure_processor_list::processor_list_by_name(&key.id().path)
                .map(|l| l.processors.clone())
                .unwrap_or_default()
        }
    }
}
