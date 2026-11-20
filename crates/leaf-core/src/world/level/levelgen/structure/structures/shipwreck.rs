//! Перенос `structures/ShipwreckStructure.java` +
//! `structures/ShipwreckPieces.java` — кораблекрушения
//! (beached/ocean варианты, маркеры лута map/treasure/supply,
//! подстройка высоты по heightmap).

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    get_lowest_y, get_mean_first_occupied_height, on_top_of_chunk_center,
    GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{StructurePiece, StructurePieceAccessor};
use super::super::structure_type::StructureType;
use super::super::template_structure_piece::TemplateStructurePiece;
use super::super::templatesystem::processors::block_ignore_processor::BlockIgnoreProcessor;
use super::super::templatesystem::structure_place_settings::StructurePlaceSettings;
use super::super::templatesystem::structure_template_manager::StructureTemplateManager;
use super::super::pools::structure_pool_element::PoolGeneratorAccess;
use super::block_state;
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
use std::sync::Arc;

const PIVOT: BlockPos = BlockPos { x: 4, y: 0, z: 15 };

const STRUCTURE_LOCATION_BEACHED: [&str; 11] = [
    "shipwreck/with_mast",
    "shipwreck/sideways_full",
    "shipwreck/sideways_fronthalf",
    "shipwreck/sideways_backhalf",
    "shipwreck/rightsideup_full",
    "shipwreck/rightsideup_fronthalf",
    "shipwreck/rightsideup_backhalf",
    "shipwreck/with_mast_degraded",
    "shipwreck/rightsideup_full_degraded",
    "shipwreck/rightsideup_fronthalf_degraded",
    "shipwreck/rightsideup_backhalf_degraded",
];

const STRUCTURE_LOCATION_OCEAN: [&str; 19] = [
    "shipwreck/with_mast",
    "shipwreck/upsidedown_full",
    "shipwreck/upsidedown_fronthalf",
    "shipwreck/upsidedown_backhalf",
    "shipwreck/sideways_full",
    "shipwreck/sideways_fronthalf",
    "shipwreck/sideways_backhalf",
    "shipwreck/rightsideup_full",
    "shipwreck/rightsideup_fronthalf",
    "shipwreck/rightsideup_backhalf",
    "shipwreck/with_mast_degraded",
    "shipwreck/upsidedown_full_degraded",
    "shipwreck/upsidedown_fronthalf_degraded",
    "shipwreck/upsidedown_backhalf_degraded",
    "shipwreck/sideways_full_degraded",
    "shipwreck/sideways_fronthalf_degraded",
    "shipwreck/sideways_backhalf_degraded",
    "shipwreck/rightsideup_full_degraded",
    "shipwreck/rightsideup_fronthalf_degraded",
    "shipwreck/rightsideup_backhalf_degraded",
];

fn template_id(path: &str) -> Identifier {
    Identifier::with_default_namespace(path).unwrap_or_else(|_| Identifier::minecraft(path))
}

/// `MARKERS_TO_LOOT` — data-marker → loot table.
fn marker_loot(name: &str) -> Option<Identifier> {
    match name {
        "map_chest" => Some(Identifier::minecraft("chests/shipwreck_map")),
        "treasure_chest" => Some(Identifier::minecraft("chests/shipwreck_treasure")),
        "supply_chest" => Some(Identifier::minecraft("chests/shipwreck_supply")),
        _ => None,
    }
}

/// `makeSettings(rotation)`.
fn make_settings(rotation: Rotation) -> StructurePlaceSettings {
    let mut settings = StructurePlaceSettings::default();
    settings
        .set_rotation(rotation)
        .set_mirror(Mirror::NONE)
        .set_rotation_pivot(PIVOT)
        .add_processor(BlockIgnoreProcessor::structure_and_air());
    settings
}

/// `ShipwreckPieces.ShipwreckPiece`.
pub struct ShipwreckPiece {
    /// `TemplateStructurePiece`-часть.
    pub base: TemplateStructurePiece,
    /// `isBeached`.
    pub is_beached: bool,
}

impl ShipwreckPiece {
    /// `ShipwreckPiece(manager, location, pos, rotation, isBeached)`.
    pub fn new(
        structure_template_manager: &StructureTemplateManager,
        location: &Identifier,
        pos: BlockPos,
        rotation: Rotation,
        is_beached: bool,
    ) -> Self {
        let mut piece = Self {
            base: TemplateStructurePiece::new(
                Identifier::minecraft("shipwreck"),
                0,
                structure_template_manager,
                location,
                location.to_string(),
                make_settings(rotation),
                pos,
            ),
            is_beached,
        };
        piece.base.data_marker_handler =
            Some(std::rc::Rc::new(|name, pos, level, random, _box| {
                if let Some(loot) = marker_loot(name) {
                    level.place_loot_container(pos.below(), &loot, random.next_long());
                }
            }));
        piece
    }

    /// `ShipwreckPiece(manager, tag)` — загрузка из NBT.
    pub fn from_nbt(
        tag: &CompoundTag,
        structure_template_manager: &StructureTemplateManager,
    ) -> Self {
        let rot = tag.get_int("Rot").unwrap_or(0);
        let is_beached = tag.get_bool("isBeached").unwrap_or(false);
        Self {
            base: TemplateStructurePiece::from_nbt(
                Identifier::minecraft("shipwreck"),
                tag,
                structure_template_manager,
                move |_location| make_settings(Rotation::by_index(rot)),
            ),
            is_beached,
        }
    }

    /// `isTooBigToFitInWorldGenRegion` — X или Y > 32.
    pub fn is_too_big_to_fit_in_world_gen_region(&self) -> bool {
        let size = self.base.template.get_size();
        size.x > 32 || size.y > 32
    }

    /// `calculateBeachedPosition(maxHeight, random)`.
    pub fn calculate_beached_position(&self, max_height: i32, random: &mut dyn RandomSource) -> i32 {
        max_height - self.base.template.get_size().y / 2 - random.next_int_bound(3)
    }

    /// `adjustPositionHeight(height)`.
    pub fn adjust_position_height(&mut self, height: i32) {
        self.base.template_position = BlockPos::new(
            self.base.template_position.x,
            height,
            self.base.template_position.z,
        );
    }
}

impl StructurePiece for ShipwreckPiece {
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
        tag.put("isBeached", Tag::Byte(if self.is_beached { 1 } else { 0 }));
        tag.put("Rot", Tag::Int(self.base.place_settings.get_rotation().get_index() as i32));
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.base.load_additional_save_data(tag);
        self.is_beached = tag.get_bool("isBeached").unwrap_or(false);
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
        if self.is_too_big_to_fit_in_world_gen_region() {
            self.base.post_process_template(level, structure_manager, generator, random, box_, chunk_pos, pos);
            return;
        }
        let heightmap = if self.is_beached {
            HeightmapType::WorldSurfaceWg
        } else {
            HeightmapType::OceanFloorWg
        };
        let size = self.base.template.get_size();
        // Java: `i = level.getMaxY() + 1` = minBuildHeight + height.
        let mut min_height = level.min_build_height() + level.height();
        let mut total = 0i64;
        let count = (size.x * size.z) as i64;
        if count == 0 {
            total = level
                .get_height(heightmap, self.base.template_position.x, self.base.template_position.z)
                as i64;
        } else {
            let end = self.base.template_position.offset(size.x - 1, 0, size.z - 1);
            for p in BlockPos::between_closed(&self.base.template_position, &end) {
                let h = level.get_height(heightmap, p.x, p.z);
                total += h as i64;
                min_height = min_height.min(h);
            }
            total /= count;
        }
        let height = if self.is_beached {
            self.calculate_beached_position(min_height, random)
        } else {
            total as i32
        };
        self.adjust_position_height(height);
        self.base.post_process_template(level, structure_manager, generator, random, box_, chunk_pos, pos);
    }
}

/// `ShipwreckPieces.addRandomPiece` — выбор шаблона + добавление.
/// Возвращает созданный кусок (нужен для adjustPositionHeight).
pub fn add_random_piece(
    structure_template_manager: &StructureTemplateManager,
    pos: BlockPos,
    rotation: Rotation,
    is_beached: bool,
    random: &mut dyn RandomSource,
) -> ShipwreckPiece {
    let locations = if is_beached {
        &STRUCTURE_LOCATION_BEACHED[..]
    } else {
        &STRUCTURE_LOCATION_OCEAN[..]
    };
    let index = random.next_int_bound(locations.len() as i32) as usize;
    let location = template_id(locations[index]);
    ShipwreckPiece::new(structure_template_manager, &location, pos, rotation, is_beached)
}

/// `ShipwreckStructure` — `isBeached` флаг + `findGenerationPoint`.
pub fn shipwreck_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
    is_beached: bool,
) -> Structure {
    let mut structure = Structure::new(id, settings, structure_type);
    let heightmap = if is_beached {
        HeightmapType::WorldSurfaceWg
    } else {
        HeightmapType::OceanFloorWg
    };
    structure.find_generation_point =
        Some(Arc::new(move |context: &mut GenerationContext| -> Option<GenerationStub> {
            let chunk_pos = context.chunk_pos;
            let min_x = chunk_pos.get_min_block_x();
            let min_z = chunk_pos.get_min_block_z();
            let stub = on_top_of_chunk_center(context, heightmap, move |pieces, context| {
                // Java `generatePieces`: rotation + addRandomPiece на
                // context.random(), затем isTooBigToFitInWorldGenRegion
                // → adjustPositionHeight через getLowestY /
                // getMeanFirstOccupiedHeight.
                let Some(template_manager) = context.template_manager.clone() else {
                    return;
                };
                let rotation = Rotation::get_random(&mut context.random);
                let start_pos = BlockPos::new(min_x, 90, min_z);
                let mut piece = add_random_piece(
                    &template_manager,
                    start_pos,
                    rotation,
                    is_beached,
                    &mut context.random,
                );
                if piece.is_too_big_to_fit_in_world_gen_region() {
                    let bb = piece.bounding_box();
                    let i = if is_beached {
                        let lowest_y = get_lowest_y(
                            context,
                            bb.min_x,
                            bb.get_x_span(),
                            bb.min_z,
                            bb.get_z_span(),
                        );
                        piece.calculate_beached_position(lowest_y, &mut context.random)
                    } else {
                        get_mean_first_occupied_height(
                            context,
                            bb.min_x,
                            bb.get_x_span(),
                            bb.min_z,
                            bb.get_z_span(),
                        )
                    };
                    piece.adjust_position_height(i);
                }
                pieces.push(Box::new(piece));
            });
            Some(stub)
        }));
    structure
}
