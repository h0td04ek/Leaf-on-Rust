//! Перенос `LegacySinglePoolElement` — single-элемент без обработки
//! data-маркеров, с `STRUCTURE_AND_AIR` игнором.

use super::single_pool_element::{SinglePoolElement, TemplateSource};
use super::structure_pool_element::{
    resolve_processor_list, PoolGeneratorAccess, StructurePoolElement,
};
use super::structure_template_pool::Projection;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::random::RandomSource;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::structure_manager::StructureManager;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::processors::{
    BlockIgnoreProcessor, JigsawReplacementProcessor, StructureProcessor,
};
use crate::world::level::levelgen::structure::templatesystem::structure_place_settings::StructurePlaceSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_processor_list::StructureProcessorList;
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::world::level::level_reader::LevelReader;
use std::rc::Rc;
use std::sync::Arc;

/// `LegacySinglePoolElement`.
pub struct LegacySinglePoolElement {
    /// Делегат-данные (как SinglePoolElement).
    inner: SinglePoolElement,
}

impl LegacySinglePoolElement {
    /// `new`.
    pub fn new(
        template: TemplateSource,
        processors: Holder<StructureProcessorList>,
        projection: Projection,
        override_liquid_settings: Option<LiquidSettings>,
    ) -> Self {
        Self {
            inner: SinglePoolElement::new(
                template,
                processors,
                projection,
                override_liquid_settings,
            ),
        }
    }

    /// `getSettings` — как у single, но `STRUCTURE_BLOCK` заменяется
    /// на `STRUCTURE_AND_AIR`.
    pub fn get_settings(
        &self,
        rotation: Rotation,
        bounding_box: Option<BoundingBox>,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> StructurePlaceSettings {
        let mut settings = self
            .inner
            .get_settings(rotation, bounding_box, liquid_settings, keep_jigsaws);
        // popProcessor(BlockIgnoreProcessor.STRUCTURE_BLOCK) —
        // удаляем последний BlockIgnoreProcessor с to_ignore ==
        // {structure_block}.
        if let Some(pos) = settings.processors.iter().rposition(|p| {
            p.as_block_ignore()
                .map(|b| b.to_ignore.len() == 1
                    && b.to_ignore[0].path == "structure_block")
                .unwrap_or(false)
        }) {
            settings.processors.remove(pos);
        }
        settings.add_processor(BlockIgnoreProcessor::structure_and_air());
        settings
    }
}

impl StructurePoolElement for LegacySinglePoolElement {
    fn get_size(&self, m: &StructureTemplateManager, r: Rotation) -> crate::core::vec3i::Vec3i {
        self.inner.get_size(m, r)
    }

    fn get_shuffled_jigsaw_blocks(
        &self,
        m: &StructureTemplateManager,
        pos: BlockPos,
        r: Rotation,
        random: &mut dyn RandomSource,
    ) -> Vec<crate::world::level::levelgen::structure::templatesystem::structure_template::JigsawBlockInfo> {
        self.inner.get_shuffled_jigsaw_blocks(m, pos, r, random)
    }

    fn get_bounding_box(&self, m: &StructureTemplateManager, pos: BlockPos, r: Rotation) -> BoundingBox {
        self.inner.get_bounding_box(m, pos, r)
    }

    fn place(
        &self,
        structure_template_manager: &StructureTemplateManager,
        level: &dyn LevelReader,
        _structure_manager: &StructureManager,
        _generator: &dyn PoolGeneratorAccess,
        offset: BlockPos,
        pos: BlockPos,
        rotation: Rotation,
        bounding_box: Option<BoundingBox>,
        random: &mut dyn RandomSource,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> bool {
        let template = self
            .inner
            .get_template(structure_template_manager);
        let settings =
            self.get_settings(rotation, bounding_box, liquid_settings, keep_jigsaws);
        if !template.place_in_world(level, offset, pos, &settings, random, 2 | 16) {
            return false;
        }
        // Legacy не обрабатывает data-маркеры.
        true
    }

    fn get_type_name(&self) -> &'static str {
        "legacy_single_pool_element"
    }

    fn get_projection(&self) -> Projection {
        self.inner.get_projection()
    }
}

/// `legacy(id)` фабрика.
pub fn legacy(
    id: &str,
    processors: Holder<StructureProcessorList>,
    projection: Projection,
    liquid_settings: Option<LiquidSettings>,
) -> Rc<dyn StructurePoolElement> {
    Rc::new(LegacySinglePoolElement::new(
        TemplateSource::Id(crate::resources::identifier::Identifier::parse(id).unwrap()),
        processors,
        projection,
        liquid_settings,
    ))
}
