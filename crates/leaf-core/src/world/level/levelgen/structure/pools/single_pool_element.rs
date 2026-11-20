//! Перенос `SinglePoolElement` — элемент пула с одним шаблоном.

use super::structure_pool_element::{
    resolve_processor_list, PoolGeneratorAccess, StructurePoolElement,
};
use super::structure_template_pool::Projection;
use crate::core::block_pos::BlockPos;
use crate::core::holder::Holder;
use crate::core::vec3i::Vec3i;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::structure_manager::StructureManager;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::processors::{
    BlockIgnoreProcessor, JigsawReplacementProcessor, StructureProcessor,
};
use crate::world::level::levelgen::structure::templatesystem::structure_place_settings::StructurePlaceSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_processor_list::StructureProcessorList;
use crate::world::level::levelgen::structure::templatesystem::structure_template::{
    JigsawBlockInfo, StructureBlockInfo, StructureTemplate,
};
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::world::level::level_reader::LevelReader;
use std::rc::Rc;
use std::sync::Arc;

/// `template` — `Either<Identifier, StructureTemplate>`.
#[derive(Clone)]
pub enum TemplateSource {
    /// `left` — id шаблона.
    Id(Identifier),
    /// `right` — runtime-шаблон.
    Template(Arc<StructureTemplate>),
}

/// `SinglePoolElement`.
pub struct SinglePoolElement {
    /// `template`.
    pub template: TemplateSource,
    /// `processors` — holder на список.
    pub processors: Holder<StructureProcessorList>,
    /// `projection`.
    pub projection: Projection,
    /// `overrideLiquidSettings`.
    pub override_liquid_settings: Option<LiquidSettings>,
}

impl SinglePoolElement {
    /// `new`.
    pub fn new(
        template: TemplateSource,
        processors: Holder<StructureProcessorList>,
        projection: Projection,
        override_liquid_settings: Option<LiquidSettings>,
    ) -> Self {
        Self {
            template,
            processors,
            projection,
            override_liquid_settings,
        }
    }

    /// `getTemplate(manager)`.
    pub(crate) fn get_template(&self, structure_template_manager: &StructureTemplateManager) -> Arc<StructureTemplate> {
        match &self.template {
            TemplateSource::Id(id) => structure_template_manager.get_or_create(id),
            TemplateSource::Template(t) => t.clone(),
        }
    }

    /// `getTemplateLocation`.
    pub fn get_template_location(&self) -> Identifier {
        match &self.template {
            TemplateSource::Id(id) => id.clone(),
            TemplateSource::Template(_) => Identifier::parse("minecraft:unknown").unwrap(),
        }
    }

    /// `getSettings(rotation, boundingBox, liquidSettings, keepJigsaws)`.
    pub fn get_settings(
        &self,
        rotation: Rotation,
        bounding_box: Option<BoundingBox>,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> StructurePlaceSettings {
        let mut settings = StructurePlaceSettings::default();
        if let Some(bb) = bounding_box {
            settings.set_bounding_box(bb);
        }
        settings.set_rotation(rotation);
        settings.set_known_shape(true);
        settings.set_ignore_entities(false);
        settings.add_processor(Rc::new(BlockIgnoreProcessor {
            to_ignore: vec![Identifier::minecraft("structure_block")],
        }));
        settings.set_finalize_entities(true);
        settings.set_liquid_settings(self.override_liquid_settings.unwrap_or(liquid_settings));
        if !keep_jigsaws {
            settings.add_processor(Rc::new(JigsawReplacementProcessor));
        }
        for processor in resolve_processor_list(&self.processors) {
            settings.add_processor(processor);
        }
        for processor in self.projection.get_processors() {
            settings.add_processor(processor);
        }
        settings
    }

    /// `getDataMarkers(manager, pos, rotation, relativePosition)` —
    /// structure-блоки с `mode=data`.
    pub fn get_data_markers(
        &self,
        structure_template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        relative_position: bool,
    ) -> Vec<StructureBlockInfo> {
        let template = self.get_template(structure_template_manager);
        let mut settings = StructurePlaceSettings::default();
        settings.set_rotation(rotation);
        let mut markers = Vec::new();
        // filterBlocks(STRUCTURE_BLOCK) — block lookup по id.
        if let Some(block_arc) =
            crate::world::level::block::block_by_identifier(&Identifier::minecraft("structure_block"))
        {
            for info in template.filter_blocks(pos, &settings, &block_arc) {
                if let Some(nbt) = &info.nbt {
                    if nbt.get_string("mode").unwrap_or("") == "data" {
                        markers.push(info.clone());
                    }
                }
            }
        }
        let _ = relative_position;
        markers
    }

    /// `sortBySelectionPriority` — jigsags с `selection_priority`
    /// сортируются по убыванию.
    pub fn sort_by_selection_priority(jigsaws: &mut [JigsawBlockInfo]) {
        jigsaws.sort_by(|a, b| {
            let pa = a
                .info
                .nbt
                .as_ref()
                .and_then(|n| n.get_int("selection_priority"))
                .unwrap_or(0);
            let pb = b
                .info
                .nbt
                .as_ref()
                .and_then(|n| n.get_int("selection_priority"))
                .unwrap_or(0);
            pb.cmp(&pa)
        });
    }
}

impl StructurePoolElement for SinglePoolElement {
    fn get_size(&self, structure_template_manager: &StructureTemplateManager, rotation: Rotation) -> Vec3i {
        self.get_template(structure_template_manager)
            .get_size_rotated(rotation)
    }

    fn get_shuffled_jigsaw_blocks(
        &self,
        structure_template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
        random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo> {
        let mut jigsaws = self
            .get_template(structure_template_manager)
            .get_jigsaws(pos, rotation);
        // Util.shuffle — Fisher–Yates.
        for i in (1..jigsaws.len()).rev() {
            let j = random.next_int_bound((i + 1) as i32) as usize;
            jigsaws.swap(i, j);
        }
        Self::sort_by_selection_priority(&mut jigsaws);
        jigsaws
    }

    fn get_bounding_box(
        &self,
        structure_template_manager: &StructureTemplateManager,
        pos: BlockPos,
        rotation: Rotation,
    ) -> BoundingBox {
        let template = self.get_template(structure_template_manager);
        let mut settings = StructurePlaceSettings::default();
        settings.set_rotation(rotation);
        template.get_bounding_box(&settings, pos)
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
        let template = self.get_template(structure_template_manager);
        let settings = self.get_settings(rotation, bounding_box.clone(), liquid_settings, keep_jigsaws);
        // Block.UPDATE_CLIENTS | UPDATE_KNOWN_SHAPE = 2 | 16.
        if !template.place_in_world(level, offset, pos, &settings, random, 2 | 16) {
            return false;
        }
        let data_markers = self.get_data_markers(structure_template_manager, offset, rotation, false);
        for marker in StructureTemplate::process_block_infos(level, offset, pos, &settings, &data_markers) {
            self.handle_data_marker(level, &marker, offset, rotation, random, bounding_box.as_ref());
        }
        true
    }

    fn get_type_name(&self) -> &'static str {
        "single_pool_element"
    }

    fn get_projection(&self) -> Projection {
        self.projection
    }
}

/// Фабрика `StructurePoolElement.single(id)` → Rc-элемент.
pub fn single(
    id: &str,
    processors: Holder<StructureProcessorList>,
    projection: Projection,
    liquid_settings: Option<LiquidSettings>,
) -> Rc<dyn StructurePoolElement> {
    Rc::new(SinglePoolElement::new(
        TemplateSource::Id(Identifier::parse(id).unwrap_or_else(|_| Identifier::minecraft("empty"))),
        processors,
        projection,
        liquid_settings,
    ))
}
