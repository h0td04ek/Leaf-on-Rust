//! Перенос `structures/IglooStructure.java` + `structures/IglooPieces.java`
//! — иглу (шаблоны igloo/top|middle|bottom, data-marker "chest",
//! корректировка высоты по WORLD_SURFACE_WG в postProcess).

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    on_top_of_chunk_center, GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{StructurePiece, StructurePieceAccessor};
use super::super::structure_type::StructureType;
use super::super::template_structure_piece::{
    TemplateStructurePiece, UPDATE_CLIENTS,
};
use super::super::templatesystem::liquid_settings::LiquidSettings;
use super::super::templatesystem::processors::block_ignore_processor::BlockIgnoreProcessor;
use super::super::templatesystem::structure_place_settings::StructurePlaceSettings;
use super::super::templatesystem::structure_template::StructureTemplate;
use super::super::templatesystem::structure_template_manager::StructureTemplateManager;
use super::super::pools::structure_pool_element::PoolGeneratorAccess;
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
use crate::world::level::levelgen::heightmap::HeightmapType;
use std::rc::Rc;
use std::sync::Arc;

/// `STRUCTURE_LOCATION_IGLOO`.
pub const STRUCTURE_LOCATION_IGLOO: &str = "igloo/top";
const STRUCTURE_LOCATION_LADDER: &str = "igloo/middle";
const STRUCTURE_LOCATION_LABORATORY: &str = "igloo/bottom";
/// `GENERATION_HEIGHT`.
pub const GENERATION_HEIGHT: i32 = 90;

fn id(path: &str) -> Identifier {
    Identifier::with_default_namespace(path).unwrap_or_else(|_| Identifier::minecraft(path))
}

/// `PIVOTS` — rotation pivot по локации шаблона.
fn pivot(location: &Identifier) -> BlockPos {
    let p = location.to_string();
    if p.ends_with("igloo/top") {
        BlockPos::new(3, 5, 5)
    } else if p.ends_with("igloo/middle") {
        BlockPos::new(1, 3, 1)
    } else {
        BlockPos::new(3, 6, 7)
    }
}

/// `OFFSETS` — смещение templatePosition по локации.
fn offset(location: &Identifier) -> BlockPos {
    let p = location.to_string();
    if p.ends_with("igloo/top") {
        BlockPos::ZERO
    } else if p.ends_with("igloo/middle") {
        BlockPos::new(2, -3, 4)
    } else {
        BlockPos::new(0, -3, -2)
    }
}

/// `makeSettings(rotation, location)`.
fn make_settings(rotation: Rotation, location: &Identifier) -> StructurePlaceSettings {
    let mut settings = StructurePlaceSettings::default();
    settings
        .set_rotation(rotation)
        .set_mirror(Mirror::NONE)
        .set_rotation_pivot(pivot(location))
        .add_processor(BlockIgnoreProcessor::structure_block())
        .set_liquid_settings(LiquidSettings::IgnoreWaterlogging);
    settings
}

/// `makePosition(location, pos, down)`.
fn make_position(location: &Identifier, pos: BlockPos, down: i32) -> BlockPos {
    pos.offset(offset(location).x, offset(location).y, offset(location).z)
        .below_n(down)
}

/// `IglooPieces.IglooPiece`.
pub struct IglooPiece {
    /// `TemplateStructurePiece`-часть.
    pub base: TemplateStructurePiece,
}

impl IglooPiece {
    /// `IglooPiece(manager, location, startPos, rotation, down)`.
    pub fn new(
        structure_template_manager: &StructureTemplateManager,
        location: &Identifier,
        start_pos: BlockPos,
        rotation: Rotation,
        down: i32,
    ) -> Self {
        let mut piece = Self {
            base: TemplateStructurePiece::new(
                Identifier::minecraft("igloo"),
                0,
                structure_template_manager,
                location,
                location.to_string(),
                make_settings(rotation, location),
                make_position(location, start_pos, down),
            ),
        };
        // `handleDataMarker("chest")` — air + loot table на pos.below().
        let loot = Identifier::minecraft("chests/igloo_chest");
        piece.base.data_marker_handler = Some(Rc::new(move |name, pos, level, random, _box| {
            if name == "chest" {
                let air = crate::world::level::block::state::block_state::BlockState::air();
                level.set_block_state(pos, &air);
                level.place_loot_container(pos.below(), &loot, random.next_long());
            }
        }));
        piece
    }

    /// `IglooPiece(manager, tag)` — загрузка из NBT.
    pub fn from_nbt(
        tag: &CompoundTag,
        structure_template_manager: &StructureTemplateManager,
    ) -> Self {
        let rot = tag.get_int("Rot").unwrap_or(0);
        Self {
            base: TemplateStructurePiece::from_nbt(
                Identifier::minecraft("igloo"),
                tag,
                structure_template_manager,
                move |location| {
                    make_settings(Rotation::by_index(rot), location)
                },
            ),
        }
    }
}

impl StructurePiece for IglooPiece {
    fn bounding_box(&self) -> BoundingBox {
        self.base.bounding_box()
    }

    fn gen_depth(&self) -> i32 {
        self.base.gen_depth()
    }

    fn set_gen_depth(&mut self, gen_depth: i32) {
        self.base.set_gen_depth(gen_depth);
    }

    fn orientation(&self) -> Option<Direction> {
        self.base.orientation()
    }

    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.base.set_orientation(orientation);
    }

    fn rotation(&self) -> Rotation {
        self.base.rotation()
    }

    fn mirror(&self) -> Mirror {
        self.base.mirror()
    }

    fn piece_type_id(&self) -> Identifier {
        self.base.piece_type_id()
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        self.base.add_additional_save_data(tag);
        // `tag.store("Rot", Rotation.LEGACY_CODEC, rotation)` — int.
        tag.put("Rot", Tag::Int(self.base.place_settings.get_rotation().get_index() as i32));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.base.load_additional_save_data(tag);
    }

    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
    ) {
        self.base.add_children(piece, pieces, random);
    }

    #[allow(clippy::too_many_arguments)]
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
        let identifier = TemplateStructurePiece::make_template_location(&self.base.template_name);
        let settings = make_settings(self.base.place_settings.get_rotation(), &identifier);
        let off = offset(&identifier);
        let rel = StructureTemplate::calculate_relative_position(
            &settings,
            BlockPos::new(3 - off.x, 0, -off.z),
        );
        let block_pos1 = self.base.template_position.offset(rel.x, rel.y, rel.z);
        let height = level.get_height(HeightmapType::WorldSurfaceWg, block_pos1.x, block_pos1.z);
        let saved_pos = self.base.template_position;
        self.base.template_position = self
            .base
            .template_position
            .offset(0, height - GENERATION_HEIGHT - 1, 0);
        self.base.post_process_template(level, structure_manager, generator, random, box_, chunk_pos, pos);
        if identifier.to_string().ends_with(STRUCTURE_LOCATION_IGLOO) {
            let rel2 = StructureTemplate::calculate_relative_position(
                &settings,
                BlockPos::new(3, 0, 5),
            );
            let block_pos3 = self
                .base
                .template_position
                .offset(rel2.x, rel2.y, rel2.z);
            let below = block_pos3.below();
            let state = level.get_block_state(below).unwrap_or_else(|| {
                crate::world::level::block::state::block_state::BlockState::air()
            });
            if !state.is_air() && state.block.id.path != "ladder" {
                let snow = super::block_state("snow_block");
                level.set_block_state(block_pos3, &snow);
            }
        }
        self.base.template_position = saved_pos;
    }
}

/// `IglooPieces.addPieces` — лаборатория (50% шанс, i×3 вниз),
/// i-1 средних секций, верх всегда.
fn add_pieces(
    structure_template_manager: &StructureTemplateManager,
    start_pos: BlockPos,
    rotation: Rotation,
    pieces: &mut Vec<Box<dyn StructurePiece>>,
    random: &mut dyn RandomSource,
) {
    if random.next_double() < 0.5 {
        let i = random.next_int_bound(8) + 4;
        pieces.push(Box::new(IglooPiece::new(
            structure_template_manager,
            &id(STRUCTURE_LOCATION_LABORATORY),
            start_pos,
            rotation,
            i * 3,
        )));
        for i1 in 0..(i - 1) {
            pieces.push(Box::new(IglooPiece::new(
                structure_template_manager,
                &id(STRUCTURE_LOCATION_LADDER),
                start_pos,
                rotation,
                i1 * 3,
            )));
        }
    }
    pieces.push(Box::new(IglooPiece::new(
        structure_template_manager,
        &id(STRUCTURE_LOCATION_IGLOO),
        start_pos,
        rotation,
        0,
    )));
}

/// `IglooStructure` — `findGenerationPoint` =
/// `onTopOfChunkCenter(WORLD_SURFACE_WG)` + `addPieces` от
/// `(minBlockX, 90, minBlockZ)` со случайным rotation.
pub fn igloo_structure(
    id_: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
) -> Structure {
    let mut structure = Structure::new(id_, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(|context: &mut GenerationContext| -> Option<GenerationStub> {
            let chunk_pos = context.chunk_pos;
            let min_x = chunk_pos.get_min_block_x();
            let min_z = chunk_pos.get_min_block_z();
            let stub = on_top_of_chunk_center(
                context,
                HeightmapType::WorldSurfaceWg,
                move |pieces, context| {
                    // Java: generatePieces использует context.random()
                    // и context.structureTemplateManager() напрямую.
                    let Some(template_manager) = context.template_manager.clone() else {
                        return;
                    };
                    let rotation = Rotation::get_random(&mut context.random);
                    let start_pos = BlockPos::new(min_x, GENERATION_HEIGHT, min_z);
                    add_pieces(
                        &template_manager,
                        start_pos,
                        rotation,
                        pieces,
                        &mut context.random,
                    );
                },
            );
            Some(stub)
        }));
    structure
}
