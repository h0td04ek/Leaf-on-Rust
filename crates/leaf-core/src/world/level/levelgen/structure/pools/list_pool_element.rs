//! Перенос `ListPoolElement` — список элементов (размещаются все
//! по очереди, провал любого = провал всего).

use super::empty_pool_element::empty_element_rc;
use super::structure_pool_element::{PoolGeneratorAccess, StructurePoolElement};
use super::structure_template_pool::Projection;
use crate::core::block_pos::BlockPos;
use crate::core::vec3i::Vec3i;
use crate::random::RandomSource;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_template::{
    JigsawBlockInfo, StructureBlockInfo,
};
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::world::level::level_reader::LevelReader;
use std::rc::Rc;

/// `ListPoolElement`.
pub struct ListPoolElement {
    /// `elements`.
    pub elements: Vec<Rc<dyn StructurePoolElement>>,
    /// `projection`.
    pub projection: Projection,
}

impl ListPoolElement {
    /// `new` — бросает при пустом списке (как Java).
    pub fn new(elements: Vec<Rc<dyn StructurePoolElement>>, projection: Projection) -> Self {
        assert!(!elements.is_empty(), "Elements are empty");
        Self {
            elements,
            projection,
        }
    }
}

impl StructurePoolElement for ListPoolElement {
    fn get_size(&self, m: &StructureTemplateManager, r: Rotation) -> Vec3i {
        let mut size = Vec3i::ZERO;
        for element in &self.elements {
            let s = element.get_size(m, r);
            size.x = size.x.max(s.x);
            size.y = size.y.max(s.y);
            size.z = size.z.max(s.z);
        }
        size
    }

    fn get_shuffled_jigsaw_blocks(
        &self,
        m: &StructureTemplateManager,
        pos: BlockPos,
        r: Rotation,
        random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo> {
        self.elements[0].get_shuffled_jigsaw_blocks(m, pos, r, random)
    }

    fn get_bounding_box(
        &self,
        m: &StructureTemplateManager,
        pos: BlockPos,
        r: Rotation,
    ) -> BoundingBox {
        let boxes: Vec<BoundingBox> = self
            .elements
            .iter()
            .filter(|e| !e.is_empty_element())
            .map(|e| e.get_bounding_box(m, pos, r))
            .collect();
        let refs: Vec<&BoundingBox> = boxes.iter().collect();
        BoundingBox::encapsulating_boxes(refs.into_iter())
            .expect("Unable to calculate boundingbox for ListPoolElement")
    }

    fn place(
        &self,
        m: &StructureTemplateManager,
        level: &dyn LevelReader,
        sm: &dyn StructureManagerAccess,
        g: &dyn PoolGeneratorAccess,
        offset: BlockPos,
        pos: BlockPos,
        rotation: Rotation,
        bounding_box: Option<BoundingBox>,
        random: &mut dyn RandomSource,
        liquid_settings: LiquidSettings,
        keep_jigsaws: bool,
    ) -> bool {
        for element in &self.elements {
            if !element.place(
                m, level, sm, g, offset, pos, rotation, bounding_box.clone(),
                random, liquid_settings, keep_jigsaws,
            ) {
                return false;
            }
        }
        true
    }

    fn get_type_name(&self) -> &'static str {
        "list_pool_element"
    }

    fn get_projection(&self) -> Projection {
        self.projection
    }

    fn handle_data_marker(
        &self,
        level: &dyn LevelReader,
        block_info: &StructureBlockInfo,
        pos: BlockPos,
        rotation: Rotation,
        random: &mut dyn RandomSource,
        bounding_box: Option<&BoundingBox>,
    ) {
        for element in &self.elements {
            element.handle_data_marker(level, block_info, pos, rotation, random, bounding_box);
        }
    }
}

/// `list(elements)` фабрика.
pub fn list(elements: Vec<Rc<dyn StructurePoolElement>>, projection: Projection) -> Rc<dyn StructurePoolElement> {
    Rc::new(ListPoolElement::new(elements, projection))
}

/// `empty` реэкспорт для удобства фабрик.
pub fn empty() -> Rc<dyn StructurePoolElement> {
    empty_element_rc()
}
