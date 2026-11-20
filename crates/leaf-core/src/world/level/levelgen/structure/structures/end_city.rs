//! `EndCityStructure` + `EndCityPieces` — город Края:
//! рекурсивная генерация башен/мостов/кораблей из шаблонов
//! `end_city/*` (house tower, tower, tower bridge, fat tower).

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    get_lowest_y_in_5by5_box_offset_7_blocks, GenerationContext, GenerationStub, Structure,
    StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{StructurePiece, StructurePieceAccessor};
use super::super::structure_type::StructureType;
use super::super::template_structure_piece::{DataMarkerHandler, TemplateStructurePiece};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::core::section_pos::SectionPos;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use crate::world::level::levelgen::structure::templatesystem::processors::{
    BlockIgnoreProcessor, StructureProcessor,
};
use crate::world::level::levelgen::structure::templatesystem::structure_place_settings::StructurePlaceSettings;
use crate::world::level::levelgen::structure::templatesystem::structure_template::StructureTemplate;
use crate::world::level::levelgen::structure::templatesystem::structure_template_manager::StructureTemplateManager;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const MAX_GEN_DEPTH: i32 = 8;
const PIECE_TYPE_ID: &str = "end_city_piece";

type EcPiece = Rc<RefCell<TemplateStructurePiece>>;

/// `EndCityPiece` — обёртка `Rc<RefCell<TemplateStructurePiece>>`
/// под `Box<dyn StructurePiece>` (как `RcPoolPiece`).
pub struct EndCityPieceHandle(pub EcPiece);

impl StructurePiece for EndCityPieceHandle {
    fn bounding_box(&self) -> BoundingBox {
        self.0.borrow().bounding_box()
    }
    fn move_by(&mut self, x: i32, y: i32, z: i32) {
        self.0.borrow_mut().move_by(x, y, z);
    }
    fn gen_depth(&self) -> i32 {
        self.0.borrow().gen_depth
    }
    fn set_gen_depth(&mut self, gen_depth: i32) {
        self.0.borrow_mut().gen_depth = gen_depth;
    }
    fn orientation(&self) -> Option<Direction> {
        self.0.borrow().orientation
    }
    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.0.borrow_mut().set_orientation(orientation);
    }
    fn rotation(&self) -> Rotation {
        self.0.borrow().rotation()
    }
    fn mirror(&self) -> crate::world::level::block::mirror::Mirror {
        self.0.borrow().mirror()
    }
    fn piece_type_id(&self) -> Identifier {
        self.0.borrow().piece_type_id()
    }
    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        self.0.borrow().add_additional_save_data(tag);
        let b = self.0.borrow();
        tag.put("Rot", Tag::String(b.place_settings.get_rotation().get_serialized_name().to_string()));
        let overwrite = matches!(
            b.place_settings
                .processors
                .first()
                .and_then(|p| p.as_block_ignore())
                .map(|bi| bi.to_ignore.len()),
            Some(1)
        );
        tag.put("OW", Tag::Byte(if overwrite { 1 } else { 0 }));
    }
    fn load_additional_save_data(&mut self, _tag: &CompoundTag) {}
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
        chunk_pos: crate::world::level::chunk_pos::ChunkPos,
        pos: BlockPos,
    ) {
        self.0.borrow_mut().post_process_template(
            level, structure_manager, generator, random, box_, chunk_pos, pos,
        );
    }
}

/// `ReferenceBounds` (Leaf fix MC-152094) — границы генерации
/// корабля: 8 чанков вокруг стартовой позиции.
#[derive(Clone, Copy)]
struct ReferenceBounds {
    min_x: i32,
    max_x: i32,
    min_z: i32,
    max_z: i32,
}

impl ReferenceBounds {
    /// `around(startPos)`.
    fn around(start_pos: BlockPos) -> Self {
        const RADIUS: i32 = 8; // ChunkStatus#MAX_STRUCTURE_DISTANCE
        let start_chunk_x = SectionPos::block_to_section_coord(start_pos.x);
        let start_chunk_z = SectionPos::block_to_section_coord(start_pos.z);
        Self {
            min_x: SectionPos::section_to_block_coord(start_chunk_x - RADIUS),
            max_x: SectionPos::section_to_block_coord_offset(start_chunk_x + RADIUS, 15),
            min_z: SectionPos::section_to_block_coord(start_chunk_z - RADIUS),
            max_z: SectionPos::section_to_block_coord_offset(start_chunk_z + RADIUS, 15),
        }
    }

    /// `contains(box)`.
    fn contains(&self, box_: &BoundingBox) -> bool {
        box_.min_x >= self.min_x
            && box_.max_x <= self.max_x
            && box_.min_z >= self.min_z
            && box_.max_z <= self.max_z
    }
}

/// `makeSettings(overwrite, rotation)` — настройки куска.
fn make_settings(overwrite: bool, rotation: Rotation) -> StructurePlaceSettings {
    let mut settings = StructurePlaceSettings::default();
    settings.set_ignore_entities(true);
    let processor: Rc<dyn StructureProcessor> = if overwrite {
        BlockIgnoreProcessor::structure_block()
    } else {
        BlockIgnoreProcessor::structure_and_air()
    };
    settings.add_processor(processor);
    settings.set_rotation(rotation);
    settings
}

/// `makeIdentifier(name)` — `end_city/<name>`.
fn make_identifier(name: &str) -> Identifier {
    Identifier::minecraft(&format!("end_city/{}", name))
}

/// `handleDataMarker` — Chest→лут, Sentry→shulker, Elytra→рамка.
fn data_marker_handler(rotation: Rotation) -> DataMarkerHandler {
    Rc::new(move |name: &str, pos: BlockPos, level: &dyn LevelReader, random: &mut dyn RandomSource, box_: &BoundingBox| {
        if name.starts_with("Chest") {
            let below = pos.below();
            if box_.is_inside_pos(below) {
                level.place_loot_container(
                    below,
                    &Identifier::minecraft("chests/end_city_treasure"),
                    random.next_long(),
                );
            }
        } else if box_.is_inside_pos(pos) && level.is_in_spawnable_bounds(pos) {
            if name.starts_with("Sentry") {
                // `Shulker.create(level, STRUCTURE)` + setPos + addFreshEntity.
                let _ = level.spawn_entity_of_type(&Identifier::minecraft("shulker"), pos);
            } else if name.starts_with("Elytra") {
                // `ItemFrame(level, pos, rotation.rotate(SOUTH))` + elytra.
                let facing = rotation.rotate(Direction::SOUTH);
                let mut nbt = CompoundTag::new();
                nbt.put("id", Tag::String("minecraft:item_frame".to_string()));
                nbt.put("Facing", Tag::String(facing.get_name().to_string()));
                let mut item = CompoundTag::new();
                item.put("id", Tag::String("minecraft:elytra".to_string()));
                item.put("count", Tag::Byte(1));
                nbt.put("Item", Tag::Compound(item));
                let _ = level.spawn_entity_from_nbt(
                    &nbt,
                    crate::world::phys::vec3::Vec3::new(pos.x as f64 + 0.5, pos.y as f64, pos.z as f64 + 0.5),
                );
            }
        }
    })
}

/// `new EndCityPiece(manager, name, startPos, rotation, overwrite)`.
fn end_city_piece(
    manager: &StructureTemplateManager,
    name: &str,
    start_pos: BlockPos,
    rotation: Rotation,
    overwrite: bool,
) -> EcPiece {
    let location = make_identifier(name);
    let place_settings = make_settings(overwrite, rotation);
    let piece = Rc::new(RefCell::new(TemplateStructurePiece::new(
        Identifier::minecraft(PIECE_TYPE_ID),
        0,
        manager,
        &location,
        name.to_string(),
        place_settings,
        start_pos,
    )));
    piece.borrow_mut().data_marker_handler = Some(data_marker_handler(rotation));
    piece
}

/// `addHelper(pieces, piece)`.
fn add_helper(pieces: &mut Vec<EcPiece>, piece: EcPiece) -> EcPiece {
    pieces.push(piece.clone());
    piece
}

/// `addPiece(manager, piece, startPos, name, rotation, overwrite)`.
fn add_piece(
    manager: &StructureTemplateManager,
    piece: &EcPiece,
    start_pos: BlockPos,
    name: &str,
    rotation: Rotation,
    overwrite: bool,
) -> EcPiece {
    let parent = piece.borrow();
    let mut child = end_city_piece(manager, name, parent.template_position, rotation, overwrite);
    let connected = StructureTemplate::calculate_connected_position(
        &parent.place_settings,
        start_pos,
        &child.borrow().place_settings,
        BlockPos::ZERO,
    );
    drop(parent);
    child.borrow_mut().move_to(connected.x, connected.y, connected.z);
    child
}

/// `SectionGenerator` — генератор секции (4 ванильных).
#[derive(Clone, Copy, PartialEq)]
enum SectionGenerator {
    HouseTower,
    Tower,
    TowerBridge,
    FatTower,
}

/// Состояние генерации (shipCreated у TOWER_BRIDGE_GENERATOR —
/// единый инстанс на всю генерацию, сбрасывается в init()).
#[derive(Default)]
struct EndCityGenState {
    ship_created: bool,
}

fn init_generators(_state: &mut EndCityGenState) {
    _state.ship_created = false;
}

/// `recursiveChildren(manager, generator, counter, piece, startPos,
/// pieces, random)`.
#[allow(clippy::too_many_arguments)]
fn recursive_children(
    manager: &StructureTemplateManager,
    generator: SectionGenerator,
    counter: i32,
    piece: &EcPiece,
    start_pos: Option<BlockPos>,
    pieces: &mut Vec<EcPiece>,
    random: &mut dyn RandomSource,
    state: &mut EndCityGenState,
) -> bool {
    if counter > MAX_GEN_DEPTH {
        return false;
    }
    let mut list: Vec<EcPiece> = Vec::new();
    if section_generate(manager, generator, counter, piece, start_pos, &mut list, random, state) {
        let random_int = random.next_int();
        let mut flag = false;
        for sp in &list {
            sp.borrow_mut().gen_depth = random_int;
            let sp_bb = sp.borrow().bounding_box();
            let collision = pieces.iter().find(|p| p.borrow().bounding_box().intersects(&sp_bb));
            if let Some(existing) = collision {
                if existing.borrow().gen_depth != piece.borrow().gen_depth {
                    flag = true;
                    break;
                }
            }
        }
        if !flag {
            pieces.extend(list);
            return true;
        }
    }
    false
}

/// `sectionGenerator.generate(...)`.
#[allow(clippy::too_many_arguments)]
fn section_generate(
    manager: &StructureTemplateManager,
    generator: SectionGenerator,
    counter: i32,
    piece: &EcPiece,
    start_pos: Option<BlockPos>,
    pieces: &mut Vec<EcPiece>,
    random: &mut dyn RandomSource,
    state: &mut EndCityGenState,
) -> bool {
    match generator {
        SectionGenerator::HouseTower => {
            house_tower_generate(manager, counter, piece, start_pos, pieces, random)
        }
        SectionGenerator::Tower => tower_generate(manager, counter, piece, pieces, random, state),
        SectionGenerator::TowerBridge => tower_bridge_generate(manager, counter, piece, pieces, random, state),
        SectionGenerator::FatTower => fat_tower_generate(manager, counter, piece, pieces, random, state),
    }
}
fn house_tower_generate(
    manager: &StructureTemplateManager,
    counter: i32,
    piece: &EcPiece,
    start_pos: Option<BlockPos>,
    pieces: &mut Vec<EcPiece>,
    random: &mut dyn RandomSource,
) -> bool {
    if counter > MAX_GEN_DEPTH {
        return false;
    }
    let rotation = piece.borrow().place_settings.get_rotation();
    // Java: `addPiece(piece, startPos, "base_floor", ...)` —
    // startPos приходит из recursiveChildren (house на мосту).
    let base_offset = start_pos.unwrap_or(BlockPos::ZERO);
    let mut end_city_piece = add_helper(
        pieces,
        add_piece(manager, piece, base_offset, "base_floor", rotation, true),
    );
    let random_int = random.next_int_bound(3);
    match random_int {
        0 => {
            end_city_piece = add_helper(
                pieces,
                add_piece(manager, &end_city_piece, BlockPos::new(-1, 4, -1), "base_roof", rotation, true),
            );
        }
        1 => {
            end_city_piece = add_helper(
                pieces,
                add_piece(manager, &end_city_piece, BlockPos::new(-1, 0, -1), "second_floor_2", rotation, false),
            );
            end_city_piece = add_helper(
                pieces,
                add_piece(manager, &end_city_piece, BlockPos::new(-1, 8, -1), "second_roof", rotation, false),
            );
            // recursiveChildren(TOWER_GENERATOR, counter+1, endCityPiece, null, pieces, random)
            let mut state = EndCityGenState::default();
            let _ = recursive_children(
                manager,
                SectionGenerator::Tower,
                counter + 1,
                &end_city_piece,
                None,
                pieces,
                random,
                &mut state,
            );
        }
        _ => {
            end_city_piece = add_helper(
                pieces,
                add_piece(manager, &end_city_piece, BlockPos::new(-1, 0, -1), "second_floor_2", rotation, false),
            );
            end_city_piece = add_helper(
                pieces,
                add_piece(manager, &end_city_piece, BlockPos::new(-1, 4, -1), "third_floor_2", rotation, false),
            );
            end_city_piece = add_helper(
                pieces,
                add_piece(manager, &end_city_piece, BlockPos::new(-1, 8, -1), "third_roof", rotation, true),
            );
            let mut state = EndCityGenState::default();
            let _ = recursive_children(
                manager,
                SectionGenerator::Tower,
                counter + 1,
                &end_city_piece,
                None,
                pieces,
                random,
                &mut state,
            );
        }
    }
    true
}

const TOWER_BRIDGES: [(Rotation, BlockPos); 4] = [
    (Rotation::NONE, BlockPos::new(1, -1, 0)),
    (Rotation::CLOCKWISE_90, BlockPos::new(6, -1, 1)),
    (Rotation::COUNTERCLOCKWISE_90, BlockPos::new(0, -1, 5)),
    (Rotation::CLOCKWISE_180, BlockPos::new(5, -1, 6)),
];

fn tower_generate(
    manager: &StructureTemplateManager,
    counter: i32,
    piece: &EcPiece,
    pieces: &mut Vec<EcPiece>,
    random: &mut dyn RandomSource,
    state: &mut EndCityGenState,
) -> bool {
    let rotation = piece.borrow().place_settings.get_rotation();
    let mut end_city_piece = add_helper(
        pieces,
        add_piece(
            manager,
            piece,
            BlockPos::new(3 + random.next_int_bound(2), -3, 3 + random.next_int_bound(2)),
            "tower_base",
            rotation,
            true,
        ),
    );
    end_city_piece = add_helper(
        pieces,
        add_piece(manager, &end_city_piece, BlockPos::new(0, 7, 0), "tower_piece", rotation, true),
    );
    let mut end_city_piece1: Option<EcPiece> = if random.next_int_bound(3) == 0 {
        Some(end_city_piece.clone())
    } else {
        None
    };
    let i = 1 + random.next_int_bound(3);
    for i1 in 0..i {
        end_city_piece = add_helper(
            pieces,
            add_piece(manager, &end_city_piece, BlockPos::new(0, 4, 0), "tower_piece", rotation, true),
        );
        if i1 < i - 1 && random.next_boolean() {
            end_city_piece1 = Some(end_city_piece.clone());
        }
    }
    if let Some(piece1) = end_city_piece1 {
        for (rot, offset) in TOWER_BRIDGES {
            if random.next_boolean() {
                let end_city_piece2 = add_helper(
                    pieces,
                    add_piece(
                        manager,
                        &piece1,
                        offset,
                        "bridge_end",
                        rotation.get_rotated(rot),
                        true,
                    ),
                );
                recursive_children(
                    manager,
                    SectionGenerator::TowerBridge,
                    counter + 1,
                    &end_city_piece2,
                    None,
                    pieces,
                    random,
                    state,
                );
            }
        }
        end_city_piece = add_helper(
            pieces,
            add_piece(manager, &end_city_piece, BlockPos::new(-1, 4, -1), "tower_top", rotation, true),
        );
    } else {
        if counter != 7 {
            return recursive_children(
                manager,
                SectionGenerator::FatTower,
                counter + 1,
                &end_city_piece,
                None,
                pieces,
                random,
                state,
            );
        }
        end_city_piece = add_helper(
            pieces,
            add_piece(manager, &end_city_piece, BlockPos::new(-1, 4, -1), "tower_top", rotation, true),
        );
    }
    let _ = end_city_piece;
    true
}

fn tower_bridge_generate(
    manager: &StructureTemplateManager,
    counter: i32,
    piece: &EcPiece,
    pieces: &mut Vec<EcPiece>,
    random: &mut dyn RandomSource,
    state: &mut EndCityGenState,
) -> bool {
    let rotation = piece.borrow().place_settings.get_rotation();
    let i = random.next_int_bound(4) + 1;
    let mut end_city_piece = add_helper(
        pieces,
        add_piece(manager, piece, BlockPos::new(0, 0, -4), "bridge_piece", rotation, true),
    );
    end_city_piece.borrow_mut().gen_depth = -1;
    let mut i1 = 0;
    for _i2 in 0..i {
        if random.next_boolean() {
            end_city_piece = add_helper(
                pieces,
                add_piece(manager, &end_city_piece, BlockPos::new(0, i1, -4), "bridge_piece", rotation, true),
            );
            i1 = 0;
        } else {
            if random.next_boolean() {
                end_city_piece = add_helper(
                    pieces,
                    add_piece(manager, &end_city_piece, BlockPos::new(0, i1, -4), "bridge_steep_stairs", rotation, true),
                );
            } else {
                end_city_piece = add_helper(
                    pieces,
                    add_piece(manager, &end_city_piece, BlockPos::new(0, i1, -8), "bridge_gentle_stairs", rotation, true),
                );
            }
            i1 = 4;
        }
    }
    if !state.ship_created && random.next_int_bound(10 - counter) == 0 {
        // Leaf fix MC-152094: корабль внутри ReferenceBounds.
        let tower_bounds = ReferenceBounds::around(end_city_piece.borrow().template_position);
        let ship = add_ship_within_reference_bounds(
            manager,
            &end_city_piece,
            -8 + random.next_int_bound(8),
            i1,
            -70 + random.next_int_bound(10),
            rotation,
            pieces,
            tower_bounds,
            random,
        );
        add_helper(pieces, ship);
        state.ship_created = true;
    } else if !recursive_children(
        manager,
        SectionGenerator::HouseTower,
        counter + 1,
        &end_city_piece,
        Some(BlockPos::new(-3, i1 + 1, -11)),
        pieces,
        random,
        state,
    ) {
        return false;
    }
    let end_city_piece = add_helper(
        pieces,
        add_piece(
            manager,
            &end_city_piece,
            BlockPos::new(4, i1, 0),
            "bridge_end",
            rotation.get_rotated(Rotation::CLOCKWISE_180),
            true,
        ),
    );
    end_city_piece.borrow_mut().gen_depth = -1;
    true
}

/// `addShipWithinReferenceBounds` (Leaf MC-152094).
#[allow(clippy::too_many_arguments)]
fn add_ship_within_reference_bounds(
    manager: &StructureTemplateManager,
    parent: &EcPiece,
    local_x: i32,
    local_y: i32,
    requested_local_z: i32,
    rotation: Rotation,
    pieces: &[EcPiece],
    reference_bounds: ReferenceBounds,
    _random: &mut dyn RandomSource,
) -> EcPiece {
    let mut fallback = None;
    for local_z in (requested_local_z..=0).rev() {
        let ship = add_piece(
            manager,
            parent,
            BlockPos::new(local_x, local_y, local_z),
            "ship",
            rotation,
            true,
        );
        if local_z == requested_local_z {
            fallback = Some(ship.clone());
        }
        let bb = ship.borrow().bounding_box();
        let collision = pieces
            .iter()
            .any(|p| p.borrow().bounding_box().intersects(&bb));
        if reference_bounds.contains(&bb) && !collision {
            return ship;
        }
    }
    // `LOGGER.warn("Unable to find a valid ship position...")`.
    fallback.expect("at least one iteration")
}

const FAT_TOWER_BRIDGES: [(Rotation, BlockPos); 4] = [
    (Rotation::NONE, BlockPos::new(4, -1, 0)),
    (Rotation::CLOCKWISE_90, BlockPos::new(12, -1, 4)),
    (Rotation::COUNTERCLOCKWISE_90, BlockPos::new(0, -1, 8)),
    (Rotation::CLOCKWISE_180, BlockPos::new(8, -1, 12)),
];

fn fat_tower_generate(
    manager: &StructureTemplateManager,
    _counter: i32,
    piece: &EcPiece,
    pieces: &mut Vec<EcPiece>,
    random: &mut dyn RandomSource,
    state: &mut EndCityGenState,
) -> bool {
    let rotation = piece.borrow().place_settings.get_rotation();
    let mut end_city_piece = add_helper(
        pieces,
        add_piece(manager, piece, BlockPos::new(-3, 4, -3), "fat_tower_base", rotation, true),
    );
    end_city_piece = add_helper(
        pieces,
        add_piece(manager, &end_city_piece, BlockPos::new(0, 4, 0), "fat_tower_middle", rotation, true),
    );
    let mut i = 0;
    while i < 2 && random.next_int_bound(3) != 0 {
        end_city_piece = add_helper(
            pieces,
            add_piece(manager, &end_city_piece, BlockPos::new(0, 8, 0), "fat_tower_middle", rotation, true),
        );
        for (rot, offset) in FAT_TOWER_BRIDGES {
            if random.next_boolean() {
                let end_city_piece1 = add_helper(
                    pieces,
                    add_piece(
                        manager,
                        &end_city_piece,
                        offset,
                        "bridge_end",
                        rotation.get_rotated(rot),
                        true,
                    ),
                );
                recursive_children(
                    manager,
                    SectionGenerator::TowerBridge,
                    1,
                    &end_city_piece1,
                    None,
                    pieces,
                    random,
                    state,
                );
            }
        }
        i += 1;
    }
    let _ = add_helper(
        pieces,
        add_piece(manager, &end_city_piece, BlockPos::new(-2, 8, -2), "fat_tower_top", rotation, true),
    );
    true
}

/// `startHouseTower(manager, startPos, rotation, pieces, random)`.
fn start_house_tower(
    manager: &StructureTemplateManager,
    start_pos: BlockPos,
    rotation: Rotation,
    pieces: &mut Vec<EcPiece>,
    random: &mut dyn RandomSource,
) {
    init_generators(&mut EndCityGenState::default());
    let mut state = EndCityGenState::default();
    let mut end_city_piece = add_helper(
        pieces,
        end_city_piece(manager, "base_floor", start_pos, rotation, true),
    );
    end_city_piece = add_helper(
        pieces,
        add_piece(manager, &end_city_piece, BlockPos::new(-1, 0, -1), "second_floor_1", rotation, false),
    );
    end_city_piece = add_helper(
        pieces,
        add_piece(manager, &end_city_piece, BlockPos::new(-1, 4, -1), "third_floor_1", rotation, false),
    );
    end_city_piece = add_helper(
        pieces,
        add_piece(manager, &end_city_piece, BlockPos::new(-1, 8, -1), "third_roof", rotation, true),
    );
    let tower_bounds = ReferenceBounds::around(start_pos);
    let _ = tower_bounds;
    recursive_children(
        manager,
        SectionGenerator::Tower,
        1,
        &end_city_piece,
        None,
        pieces,
        random,
        &mut state,
    );
}

/// `EndCityStructure` — фабрика.
pub fn end_city_structure(
    id: Identifier,
    settings: StructureSettings,
    structure_type: Arc<StructureType>,
) -> Structure {
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(|context: &mut GenerationContext| -> Option<GenerationStub> {
            let rotation = Rotation::get_random(&mut context.random);
            let lowest = get_lowest_y_in_5by5_box_offset_7_blocks(context, rotation);
            if lowest.y < 60 {
                return None;
            }
            let manager = context.template_manager.clone()?;
            let mut pieces: Vec<EcPiece> = Vec::new();
            start_house_tower(manager.as_ref(), lowest, rotation, &mut pieces, &mut context.random);
            let boxed: Vec<Box<dyn StructurePiece>> = pieces
                .into_iter()
                .map(|p| Box::new(EndCityPieceHandle(p)) as Box<dyn StructurePiece>)
                .collect();
            Some(GenerationStub::new(lowest, move |out: &mut Vec<Box<dyn StructurePiece>>, _context: &mut GenerationContext| {
                for piece in boxed {
                    out.push(piece);
                }
            }))
        }));
    structure
}
