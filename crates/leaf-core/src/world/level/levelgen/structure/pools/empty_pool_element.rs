//! Перенос `EmptyPoolElement` — пустой элемент пула.

use super::structure_pool_element::{PoolGeneratorAccess, StructurePoolElement};
use super::structure_template_pool::Projection;
use crate::core::block_pos::BlockPos;
use crate::core::vec3i::Vec3i;
use crate::random::RandomSource;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::levelgen::structure::bounding_box::BoundingBox;
use crate::world::level::levelgen::structure::structure_manager::StructureManager;
use crate::world::level::levelgen::structure::templatesystem::liquid_settings::LiquidSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_template::{
    JigsawBlockInfo, StructureBlockInfo,
};
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::world::level::level_reader::LevelReader;
use std::rc::Rc;

/// `EmptyPoolElement` — синглтон.
pub struct EmptyPoolElement;

impl StructurePoolElement for EmptyPoolElement {
    fn get_size(&self, _m: &StructureTemplateManager, _r: Rotation) -> Vec3i {
        Vec3i::ZERO
    }

    fn get_shuffled_jigsaw_blocks(
        &self,
        _m: &StructureTemplateManager,
        _pos: BlockPos,
        _r: Rotation,
        _random: &mut dyn RandomSource,
    ) -> Vec<JigsawBlockInfo> {
        Vec::new()
    }

    fn get_bounding_box(
        &self,
        _m: &StructureTemplateManager,
        _pos: BlockPos,
        _r: Rotation,
    ) -> BoundingBox {
        panic!("Invalid call to EmptyPoolElement.getBoundingBox, filter me!")
    }

    fn place(
        &self,
        _m: &StructureTemplateManager,
        _level: &dyn LevelReader,
        _sm: &StructureManager,
        _g: &dyn PoolGeneratorAccess,
        _offset: BlockPos,
        _pos: BlockPos,
        _rotation: Rotation,
        _box: Option<BoundingBox>,
        _random: &mut dyn RandomSource,
        _liquid: LiquidSettings,
        _keep_jigsaws: bool,
    ) -> bool {
        true
    }

    fn get_type_name(&self) -> &'static str {
        "empty_pool_element"
    }

    fn get_projection(&self) -> Projection {
        Projection::TerrainMatching
    }

    fn is_empty_element(&self) -> bool {
        true
    }
}

/// `EmptyPoolElement.INSTANCE` — `empty()` фабрика.
pub fn empty_element_rc() -> Rc<dyn StructurePoolElement> {
    thread_local! {
        static INSTANCE: Rc<dyn StructurePoolElement> = Rc::new(EmptyPoolElement);
    }
    INSTANCE.with(|i| i.clone())
}
