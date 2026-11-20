//! Перенос `TemplateStructurePiece` — базовый класс кусков,
//! размещающих `StructureTemplate` (NetherFossil/Igloo/Shipwreck/
//! JungleTemple/DesertPyramid/OceanRuin/EndCity/RuinedPortal и др.).
//!
//! Java — abstract class; здесь — встраиваемая структура
//! `TemplateStructurePiece`, реализующая `StructurePiece`.
//! Наследники держат её в поле `base` и делегируют методы трейта.

use super::bounding_box::BoundingBox;
use super::structure_piece::StructurePiece;
use super::templatesystem::structure_place_settings::StructurePlaceSettings;
use super::templatesystem::structure_template::StructureTemplate;
use super::templatesystem::structure_template_manager::StructureTemplateManager;
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::structure_manager::StructureManagerAccess;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use std::rc::Rc;
use std::sync::Arc;

/// `Block.UPDATE_CLIENTS`.
pub const UPDATE_CLIENTS: i32 = 2;
/// `Block.UPDATE_ALL`.
pub const UPDATE_ALL: i32 = 3;

/// `handleDataMarker(name, pos, level, random, box)` — обработчик
/// data-маркеров шаблона (дочерние куски переопределяют).
pub type DataMarkerHandler = Rc<
    dyn Fn(&str, BlockPos, &dyn LevelReader, &mut dyn RandomSource, &BoundingBox),
>;

/// `TemplateStructurePiece` — встраиваемая база template-кусков.
pub struct TemplateStructurePiece {
    /// `templateName`.
    pub template_name: String,
    /// `template`.
    pub template: Arc<StructureTemplate>,
    /// `placeSettings`.
    pub place_settings: StructurePlaceSettings,
    /// `templatePosition`.
    pub template_position: BlockPos,
    /// `boundingBox`.
    pub bounding_box: BoundingBox,
    /// `genDepth`.
    pub gen_depth: i32,
    /// `orientation`.
    pub orientation: Option<Direction>,
    /// `mirror` (для StructurePiece-трансформаций).
    pub mirror: Mirror,
    /// `rotation` (для StructurePiece-трансформаций).
    pub rotation: Rotation,
    /// `pieceTypeId` — `StructurePieceType` идентификатор.
    pub piece_type_id: Identifier,
    /// `handleDataMarker` — переопределяется наследником.
    pub data_marker_handler: Option<DataMarkerHandler>,
}

impl TemplateStructurePiece {
    /// `TemplateStructurePiece(type, genDepth, manager, location,
    /// templateName, placeSettings, templatePosition)`.
    pub fn new(
        piece_type_id: Identifier,
        gen_depth: i32,
        structure_template_manager: &StructureTemplateManager,
        location: &Identifier,
        template_name: String,
        mut place_settings: StructurePlaceSettings,
        template_position: BlockPos,
    ) -> Self {
        let template = structure_template_manager.get_or_create(location);
        let bounding_box = template.get_bounding_box(&place_settings, template_position);
        // Java: super(type, genDepth, bb); setOrientation(NORTH).
        place_settings.set_bounding_box(bounding_box.clone());
        Self {
            template_name,
            template,
            place_settings,
            template_position,
            bounding_box,
            gen_depth,
            orientation: Some(Direction::NORTH),
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            piece_type_id,
            data_marker_handler: None,
        }
    }

    /// `TemplateStructurePiece(type, tag, manager, factory)` —
    /// загрузка из NBT.
    pub fn from_nbt(
        piece_type_id: Identifier,
        tag: &CompoundTag,
        structure_template_manager: &StructureTemplateManager,
        place_settings_factory: impl Fn(&Identifier) -> StructurePlaceSettings,
    ) -> Self {
        let template_name = tag.get_string("Template").unwrap_or("").to_string();
        let template_position = BlockPos::new(
            tag.get_int("TPX").unwrap_or(0),
            tag.get_int("TPY").unwrap_or(0),
            tag.get_int("TPZ").unwrap_or(0),
        );
        let location = Self::make_template_location(&template_name);
        let template = structure_template_manager.get_or_create(&location);
        let place_settings = place_settings_factory(&location);
        let bounding_box = template.get_bounding_box(&place_settings, template_position);
        Self {
            template_name,
            template,
            place_settings,
            template_position,
            bounding_box,
            gen_depth: 0,
            orientation: Some(Direction::NORTH),
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            piece_type_id,
            data_marker_handler: None,
        }
    }

    /// `makeTemplateLocation` — `Identifier.parse(templateName)`.
    pub fn make_template_location(template_name: &str) -> Identifier {
        Identifier::parse(template_name)
            .unwrap_or_else(|_| Identifier::minecraft(template_name))
    }

    /// `template()`.
    pub fn template(&self) -> &Arc<StructureTemplate> {
        &self.template
    }

    /// `templatePosition()`.
    pub fn template_position(&self) -> BlockPos {
        self.template_position
    }

    /// `placeSettings()`.
    pub fn place_settings(&self) -> &StructurePlaceSettings {
        &self.place_settings
    }

    /// `move(x, y, z)` — сдвиг куска + templatePosition.
    pub fn move_by(&mut self, x: i32, y: i32, z: i32) {
        self.bounding_box.move_(x, y, z);
        self.template_position = self.template_position.offset(x, y, z);
    }

    /// `StructurePiece.move(x, y, z)` — АБСОЛЮТНОЕ перемещение:
    /// boundingBox.min := (x,y,z), templatePosition := (x,y,z).
    /// (Java `StructurePiece.move` передаёт дельту в
    /// `BoundingBox.move`; `TemplateStructurePiece.move`
    /// выставляет templatePosition напрямую.)
    pub fn move_to(&mut self, x: i32, y: i32, z: i32) {
        self.bounding_box.move_(
            x - self.bounding_box.min_x,
            y - self.bounding_box.min_y,
            z - self.bounding_box.min_z,
        );
        self.template_position = BlockPos::new(x, y, z);
    }

    /// Общая часть `postProcess` — вызывается из `post_process`
    /// наследника (после его собственной логики или вместо неё).
    #[allow(clippy::too_many_arguments)]
    pub fn post_process_template(
        &mut self,
        level: &dyn LevelReader,
        _structure_manager: &dyn StructureManagerAccess,
        _generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        pos: BlockPos,
    ) {
        self.place_settings.set_bounding_box(box_.clone());
        self.bounding_box = self
            .template
            .get_bounding_box(&self.place_settings, self.template_position);
        if !self.template.place_in_world(
            level,
            self.template_position,
            pos,
            &self.place_settings,
            random,
            UPDATE_CLIENTS,
        ) {
            return;
        }

        let structure_block_id = Identifier::minecraft("structure_block");
        let jigsaw_id = Identifier::minecraft("jigsaw");
        for info in self
            .template
            .filter_blocks(self.template_position, &self.place_settings, &structure_block_id_block())
        {
            if let Some(nbt) = &info.nbt {
                if nbt.get_string("mode") == Some("data") {
                    let metadata = nbt.get_string("metadata").unwrap_or("").to_string();
                    let handler = self.data_marker_handler.clone();
                    if let Some(handler) = handler {
                        handler(&metadata, info.pos, level, random, box_);
                    }
                }
            }
        }
        for info in self
            .template
            .filter_blocks(self.template_position, &self.place_settings, &jigsaw_id_block())
        {
            if let Some(nbt) = &info.nbt {
                let final_state = nbt
                    .get_string("final_state")
                    .unwrap_or("minecraft:air")
                    .to_string();
                let state = if let Ok(id) = Identifier::parse(&final_state) {
                    crate::world::level::block::block_by_identifier(&id)
                        .and_then(|b| b.default_block_state())
                        .unwrap_or_else(crate::world::level::block::state::block_state::BlockState::air)
                } else {
                    crate::world::level::block::state::block_state::BlockState::air()
                };
                level.set_block_state(info.pos, &state);
            }
        }
    }
}

/// `filterBlocks` принимает `&Block`; наши блоки ищутся по id —
/// helper возвращает блок-референс, но если блок не зарегистрирован
/// фильтрация идёт по `state.block.id` напрямую ниже.
fn structure_block_id_block() -> crate::world::level::block::Block {
    crate::world::level::block::block_by_identifier(&Identifier::minecraft("structure_block"))
        .map(|b| (*b).clone())
        .unwrap_or_else(|| crate::world::level::block::Block::new(Identifier::minecraft("structure_block")))
}

fn jigsaw_id_block() -> crate::world::level::block::Block {
    crate::world::level::block::block_by_identifier(&Identifier::minecraft("jigsaw"))
        .map(|b| (*b).clone())
        .unwrap_or_else(|| crate::world::level::block::Block::new(Identifier::minecraft("jigsaw")))
}

impl StructurePiece for TemplateStructurePiece {
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
        super::structure_piece::piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
    }

    fn rotation(&self) -> Rotation {
        // Java TemplateStructurePiece.getRotation → placeSettings.getRotation().
        self.place_settings.get_rotation()
    }

    fn mirror(&self) -> Mirror {
        self.place_settings.get_mirror()
    }

    fn piece_type_id(&self) -> Identifier {
        self.piece_type_id.clone()
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("TPX", Tag::Int(self.template_position.x));
        tag.put("TPY", Tag::Int(self.template_position.y));
        tag.put("TPZ", Tag::Int(self.template_position.z));
        tag.put("Template", Tag::String(self.template_name.clone()));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        // Загрузка — через from_nbt у конкретного куска.
        let _ = tag;
    }

    fn add_children(
        &mut self,
        _piece: &mut dyn StructurePiece,
        _pieces: &mut dyn super::structure_piece::StructurePieceAccessor,
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
        chunk_pos: ChunkPos,
        pos: BlockPos,
    ) {
        self.post_process_template(level, structure_manager, generator, random, box_, chunk_pos, pos);
    }
}
