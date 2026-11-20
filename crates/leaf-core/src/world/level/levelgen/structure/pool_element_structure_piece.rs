//! Перенос `PoolElementStructurePiece` — jigsaw-кусок структуры
//! (элемент пула + позиция + junctions).

use super::bounding_box::BoundingBox;
use super::pieces_container::register_piece_type;
use super::structure_piece::{
    piece_set_orientation, StructurePiece, StructurePieceAccessor,
};
use super::templatesystem::liquid_settings::LiquidSettings;
use super::templatesystem::structure_template_manager::StructureTemplateManager;
use super::pools::structure_template_pool::JigsawJunction;
use super::pools::structure_pool_element::{PoolGeneratorAccess, StructurePoolElement};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::vec3i::Vec3i;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess;
use std::rc::Rc;
use std::sync::Arc;

/// `PoolElementStructurePiece`.
pub struct PoolElementStructurePiece {
    /// `element`.
    pub element: Rc<dyn StructurePoolElement>,
    /// `position`.
    pub position: BlockPos,
    /// `groundLevelDelta`.
    pub ground_level_delta: i32,
    /// `rotation`.
    pub rotation: Rotation,
    /// `junctions`.
    pub junctions: Vec<JigsawJunction>,
    /// `boundingBox` (как в StructurePiece).
    bounding_box: BoundingBox,
    /// `genDepth`.
    gen_depth: i32,
    /// `orientation` (mirror/rotation для postProcess-трансформов).
    orientation: Option<Direction>,
    /// `mirror`.
    mirror: Mirror,
    /// `structureTemplateManager`.
    structure_template_manager: std::sync::Arc<StructureTemplateManager>,
    /// `liquidSettings`.
    pub liquid_settings: LiquidSettings,
}

impl PoolElementStructurePiece {
    /// `new` (полный).
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        structure_template_manager: std::sync::Arc<StructureTemplateManager>,
        element: Rc<dyn StructurePoolElement>,
        position: BlockPos,
        ground_level_delta: i32,
        rotation: Rotation,
        bounding_box: BoundingBox,
        liquid_settings: LiquidSettings,
    ) -> Self {
        Self {
            element,
            position,
            ground_level_delta,
            rotation,
            junctions: Vec::new(),
            bounding_box,
            gen_depth: 0,
            orientation: None,
            mirror: Mirror::NONE,
            structure_template_manager,
            liquid_settings,
        }
    }

    /// `place(level, structureManager, generator, random, box,
    /// pos, keepJigsaws)`.
    #[allow(clippy::too_many_arguments)]
    pub fn place(
        &self,
        level: &dyn LevelReader,
        structure_manager: &dyn StructureManagerAccess,
        generator: &dyn PoolGeneratorAccess,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        pos: BlockPos,
        keep_jigsaws: bool,
    ) {
        self.element.place(
            &self.structure_template_manager,
            level,
            structure_manager,
            generator,
            self.position,
            pos,
            self.rotation,
            Some(box_.clone()),
            random,
            self.liquid_settings,
            keep_jigsaws,
        );
    }

    /// `move(x, y, z)` — сдвиг куска.
    pub fn move_by(&mut self, x: i32, y: i32, z: i32) {
        self.bounding_box.move_(x, y, z);
        self.position = self.position.offset(x, y, z);
    }

    /// `addJunction`.
    pub fn add_junction(&mut self, junction: JigsawJunction) {
        self.junctions.push(junction);
    }

    /// `getJunctions`.
    pub fn get_junctions(&self) -> &[JigsawJunction] {
        &self.junctions
    }

    /// `getElement`.
    pub fn get_element(&self) -> &Rc<dyn StructurePoolElement> {
        &self.element
    }

    /// `getPosition`.
    pub fn get_position(&self) -> BlockPos {
        self.position
    }

    /// `getGroundLevelDelta`.
    pub fn get_ground_level_delta(&self) -> i32 {
        self.ground_level_delta
    }
}

impl StructurePiece for PoolElementStructurePiece {
    fn bounding_box(&self) -> BoundingBox {
        self.bounding_box.clone()
    }

    fn gen_depth(&self) -> i32 {
        self.gen_depth
    }

    fn set_gen_depth(&mut self, gen_depth: i32) {
        self.gen_depth = gen_depth;
    }

    fn orientation(&self) -> Option<Direction> {
        self.orientation
    }

    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.orientation = orientation;
        piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
    }

    fn rotation(&self) -> Rotation {
        self.rotation
    }

    fn mirror(&self) -> Mirror {
        self.mirror
    }

    fn piece_type_id(&self) -> Identifier {
        Identifier::minecraft("jigsaw")
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("PosX", Tag::Int(self.position.x));
        tag.put("PosY", Tag::Int(self.position.y));
        tag.put("PosZ", Tag::Int(self.position.z));
        tag.put("ground_level_delta", Tag::Int(self.ground_level_delta));
        // pool_element/liquid_settings — через pool_element реестр
        // (NBT-кодек StructurePoolElement — отдельный этап).
        let mut junctions_tag: Vec<Tag> = Vec::new();
        for junction in &self.junctions {
            let mut j = CompoundTag::new();
            j.put("source_x", Tag::Int(junction.source_x));
            j.put("source_ground_y", Tag::Int(junction.source_ground_y));
            j.put("source_z", Tag::Int(junction.source_z));
            j.put("delta_y", Tag::Int(junction.delta_y));
            j.put("dest_proj", Tag::String(junction.dest_projection.get_name().to_string()));
            junctions_tag.push(Tag::Compound(j));
        }
        tag.put("junctions", Tag::List(junctions_tag));
    }

    fn load_additional_save_data(&mut self, _tag: &CompoundTag) {
        // pool_element-десериализация — через piece factory
        // (pool_element codec — отдельный этап).
    }

    fn add_children(
        &mut self,
        _piece: &mut dyn StructurePiece,
        _pieces: &mut dyn StructurePieceAccessor,
        _random: &mut dyn RandomSource,
    ) {
    }

    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        structure_manager: &dyn StructureManagerAccess,
        generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        pos: BlockPos,
    ) {
        // Java: place(level, structureManager, generator, random,
        // box, pos, keepJigsaws=false).
        if let Some(generator) = generator {
            self.element.place(
                &self.structure_template_manager,
                level,
                structure_manager,
                generator,
                self.position,
                pos,
                self.rotation,
                Some(box_.clone()),
                random,
                self.liquid_settings,
                false,
            );
        }
    }
}

/// Регистрация типа куска `jigsaw`.
pub fn register_pool_element_piece_type() {
    register_piece_type(
        Identifier::minecraft("jigsaw"),
        Arc::new(|_tag| None), // pool_element codec — отдельный этап
    );
}
