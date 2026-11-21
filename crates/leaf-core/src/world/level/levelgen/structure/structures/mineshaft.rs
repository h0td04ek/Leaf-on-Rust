//! Перенос `structures/MineshaftStructure.java` +
//! `structures/MineshaftPieces.java` — заброшенная шахта
//! (NORMAL/MESA типы, room/corridor/crossing/stairs куски,
//! рекурсивная генерация до глубины 8).

use super::super::bounding_box::BoundingBox;
use super::super::pieces_container::{PieceHeader, PiecesContainer};
use super::super::structure::{
    GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{
    generate_box_pred, generate_maybe_box_pred,
    generate_upper_half_sphere_pred, get_block, is_interior,
    is_replaceable_by_structures, maybe_generate_block_pred,
    move_below_sea_level, offset_pieces_vertically, piece_set_orientation,
    pieces_bounding_box, place_block_pred, StructurePiece,
    StructurePieceAccessor,
};
use super::super::structure_type::{register_structure_type, StructureType};
use super::{block_state, with_string_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::util::mth::random_between_inclusive;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const MAX_DEPTH: i32 = 8;
const MAGIC_START_Y: i32 = 50;
const MAX_PILLAR_HEIGHT: i32 = 20;
const MAX_CHAIN_HEIGHT: i32 = 50;

/// `MineshaftStructure.Type` — NORMAL/MESA с состояниями
/// wood/planks/fence.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MineshaftType {
    Normal,
    Mesa,
}

impl MineshaftType {
    /// `byId` — ordinal (NBT `MST`, OutOfBounds → ZERO).
    pub fn by_id(id: i32) -> Self {
        if id == 1 { Self::Mesa } else { Self::Normal }
    }

    /// `getWoodState`.
    pub fn wood_state(self) -> BlockState {
        match self {
            Self::Normal => block_state("oak_log"),
            Self::Mesa => block_state("dark_oak_log"),
        }
    }

    /// `getPlanksState`.
    pub fn planks_state(self) -> BlockState {
        match self {
            Self::Normal => block_state("oak_planks"),
            Self::Mesa => block_state("dark_oak_planks"),
        }
    }

    /// `getFenceState`.
    pub fn fence_state(self) -> BlockState {
        match self {
            Self::Normal => block_state("oak_fence"),
            Self::Mesa => block_state("dark_oak_fence"),
        }
    }
}

/// `type.ordinal()` для NBT `MST`.
fn type_ordinal(ty: MineshaftType) -> i32 {
    match ty {
        MineshaftType::Normal => 0,
        MineshaftType::Mesa => 1,
    }
}

/// `block.is(other.getBlock())` — сравнение по id блока.
fn same_block(a: &BlockState, b: &BlockState) -> bool {
    a.block.id == b.block.id
}

/// `canBeReplaced` из `MineShaftPiece` — нельзя заменять свои
/// planks/wood/fence/iron_chain.
fn mine_can_be_replaced(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    ty: MineshaftType,
    x: i32,
    y: i32,
    z: i32,
    box_: &BoundingBox,
) -> bool {
    let state = get_block(level, bb, orientation, x, y, z, box_);
    let planks = ty.planks_state();
    let wood = ty.wood_state();
    let fence = ty.fence_state();
    !(same_block(&state, &planks)
        || same_block(&state, &wood)
        || same_block(&state, &fence)
        || state.block.id.path == "iron_chain")
}

/// Замыкание-предикат для `_pred` хелперов.
fn can_replace<'a>(
    bb: &'a BoundingBox,
    orientation: Option<Direction>,
    ty: MineshaftType,
) -> impl Fn(&dyn LevelReader, i32, i32, i32, &BoundingBox) -> bool + 'a {
    move |level, x, y, z, box_| mine_can_be_replaced(level, bb, orientation, ty, x, y, z, box_)
}

/// `isSupportingBox` — все блоки y+1 над колонной непустые.
fn is_supporting_box(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    x_start: i32,
    x_end: i32,
    y: i32,
    z: i32,
    box_: &BoundingBox,
) -> bool {
    for i in x_start..=x_end {
        if get_block(level, bb, orientation, i, y + 1, z, box_).is_air() {
            return false;
        }
    }
    true
}

/// `isInInvalidLocation` — биом MINESHAFT_BLOCKING или жидкость
/// на периметре куска.
fn is_in_invalid_location(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    bounding_box: &BoundingBox,
) -> bool {
    let max = bb.min_x - 1.max(bounding_box.min_x);
    let max1 = bb.min_y - 1.max(bounding_box.min_y);
    let max2 = bb.min_z - 1.max(bounding_box.min_z);
    let min = (bb.max_x + 1).min(bounding_box.max_x);
    let min1 = (bb.max_y + 1).min(bounding_box.max_y);
    let min2 = (bb.max_z + 1).min(bounding_box.max_z);
    let center = BlockPos::new((max + min) / 2, (max1 + min1) / 2, (max2 + min2) / 2);
    if level.is_mineshaft_blocking_biome(center) {
        return true;
    }
    let mut pos = center;
    let liquid = |level: &dyn LevelReader, pos: BlockPos| {
        level
            .get_block_state(pos)
            .map(|s| !s.get_fluid_state().is_empty())
            .unwrap_or(false)
    };
    for i in max..=min {
        for i1 in max2..=min2 {
            pos = BlockPos::new(i, max1, i1);
            if liquid(level, pos) {
                return true;
            }
            pos = BlockPos::new(i, min1, i1);
            if liquid(level, pos) {
                return true;
            }
        }
    }
    for i in max..=min {
        for i1 in max1..=min1 {
            pos = BlockPos::new(i, i1, max2);
            if liquid(level, pos) {
                return true;
            }
            pos = BlockPos::new(i, i1, min2);
            if liquid(level, pos) {
                return true;
            }
        }
    }
    for i in max2..=min2 {
        for i1 in max1..=min1 {
            pos = BlockPos::new(max, i1, i);
            if liquid(level, pos) {
                return true;
            }
            pos = BlockPos::new(min, i1, i);
            if liquid(level, pos) {
                return true;
            }
        }
    }
    false
}

/// `setPlanksBlock` — пол под ногами, если верхняя грань не
/// sturdy.
fn set_planks_block(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    ty: MineshaftType,
    x: i32,
    y: i32,
    z: i32,
    box_: &BoundingBox,
) {
    if is_interior(level, bb, orientation, x, y, z, box_) {
        let t = super::super::structure_piece::PieceTransform { bb, orientation };
        let world_pos = t.world_pos(x, y, z);
        if !level.is_face_sturdy(world_pos, Direction::UP) {
            level.set_block_state(world_pos, &ty.planks_state());
        }
    }
}

/// `hasSturdyNeighbours` — >= required sturdy-соседей.
fn has_sturdy_neighbours(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    x: i32,
    y: i32,
    z: i32,
    required: i32,
    box_: &BoundingBox,
) -> bool {
    let t = super::super::structure_piece::PieceTransform { bb, orientation };
    let world_pos = t.world_pos(x, y, z);
    let mut count = 0;
    for direction in Direction::VALUES {
        let neighbour = world_pos.relative(direction);
        if box_.is_inside_pos(neighbour)
            && level.is_face_sturdy(neighbour, direction.get_opposite())
        {
            count += 1;
            if count >= required {
                return true;
            }
        }
    }
    false
}

/// `FallingBlock` — песок/гравий/бетонные порошки/наковальни и
/// пр. (проверка по id; полный список — как в ванили).
fn is_falling_block(state: &BlockState) -> bool {
    matches!(
        state.block.id.path.as_str(),
        "sand" | "red_sand" | "gravel" | "anvil" | "chipped_anvil"
            | "damaged_anvil" | "white_concrete_powder" | "orange_concrete_powder"
            | "magenta_concrete_powder" | "light_blue_concrete_powder"
            | "yellow_concrete_powder" | "lime_concrete_powder"
            | "pink_concrete_powder" | "gray_concrete_powder"
            | "light_gray_concrete_powder" | "cyan_concrete_powder"
            | "purple_concrete_powder" | "blue_concrete_powder"
            | "brown_concrete_powder" | "green_concrete_powder"
            | "red_concrete_powder" | "black_concrete_powder"
            | "dragon_egg" | "suspicious_sand" | "suspicious_gravel"
            | "suspicious_sandstone"
    )
}

/// `fillColumnBetween` — вертикальная заливка [minY, maxY).
fn fill_column_between(
    level: &dyn LevelReader,
    state: &BlockState,
    x: i32,
    min_y: i32,
    max_y: i32,
    z: i32,
) {
    for y in min_y..max_y {
        level.set_block_state(BlockPos::new(x, y, z), state);
    }
}

/// `canPlaceColumnOnTopOf` — верхняя грань блока sturdy.
fn can_place_column_on_top_of(level: &dyn LevelReader, pos: BlockPos, _state: &BlockState) -> bool {
    level.is_face_sturdy(pos, Direction::UP)
}

/// `canHangChainBelow` — `Block.canSupportCenter(DOWN)` и не
/// FallingBlock.
fn can_hang_chain_below(level: &dyn LevelReader, pos: BlockPos, state: &BlockState) -> bool {
    level.is_face_sturdy(pos, Direction::DOWN) && !is_falling_block(state)
}

// ------------------------------------------------------------------
// Генерация дочерних кусков (addChildren)
// ------------------------------------------------------------------

/// `generateAndAddPiece` — возвращает bounding box созданного
/// куска (для `childEntranceBoxes` у комнаты).
fn generate_and_add_piece(
    parent_bb: &BoundingBox,
    ty: MineshaftType,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    direction: Direction,
    gen_depth: i32,
) -> Option<BoundingBox> {
    if gen_depth > MAX_DEPTH {
        return None;
    }
    if (x - parent_bb.min_x).abs() > 80 || (z - parent_bb.min_z).abs() > 80 {
        return None;
    }
    let mut piece = create_random_shaft_piece(pieces, random, x, y, z, direction, gen_depth + 1, ty)?;
    let child_bb = piece.bounding_box();
    let child_gen = piece_child_gen(&*piece);
    pieces.add_piece(piece);
    if let Some(gen) = child_gen {
        expand_children(&gen, parent_bb, ty, pieces, random);
    }
    Some(child_bb)
}

/// Данные куска, нужные для его `addChildren` (аналог полей
/// `this` в Java — ориентация/флаги/направление).
enum ChildGen {
    Corridor {
        bb: BoundingBox,
        gen_depth: i32,
        orientation: Option<Direction>,
        ty: MineshaftType,
    },
    Crossing {
        bb: BoundingBox,
        gen_depth: i32,
        direction: Direction,
        is_two_floored: bool,
        ty: MineshaftType,
    },
    Stairs {
        bb: BoundingBox,
        gen_depth: i32,
        orientation: Option<Direction>,
        ty: MineshaftType,
    },
}

/// Извлекает `ChildGen` из куска (downcast по piece_type_id).
fn piece_child_gen(piece: &dyn StructurePiece) -> Option<ChildGen> {
    let bb = piece.bounding_box();
    let depth = piece.gen_depth();
    let id = piece.piece_type_id();
    match id.path.as_str() {
        "MSCorridor" => {
            let p = piece.as_any()?.downcast_ref::<MineShaftCorridorPiece>()?;
            Some(ChildGen::Corridor {
                bb,
                gen_depth: depth,
                orientation: piece.orientation(),
                ty: p.ty,
            })
        }
        "MSCrossing" => {
            let p = piece.as_any()?.downcast_ref::<MineShaftCrossingPiece>()?;
            Some(ChildGen::Crossing {
                bb,
                gen_depth: depth,
                direction: p.direction,
                is_two_floored: p.is_two_floored,
                ty: p.ty,
            })
        }
        "MSStairs" => {
            let p = piece.as_any()?.downcast_ref::<MineShaftStairsPiece>()?;
            Some(ChildGen::Stairs {
                bb,
                gen_depth: depth,
                orientation: piece.orientation(),
                ty: p.ty,
            })
        }
        _ => None,
    }
}

/// `createRandomShaftPiece` — взвешенный выбор типа куска.
fn create_random_shaft_piece(
    pieces: &dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    orientation: Direction,
    gen_depth: i32,
    ty: MineshaftType,
) -> Option<Box<dyn StructurePiece>> {
    let random_int = random.next_int_bound(100);
    if random_int >= 80 {
        if let Some(bb) = MineShaftCrossingPiece::find_crossing(pieces, random, x, y, z, orientation) {
            return Some(Box::new(MineShaftCrossingPiece::new(gen_depth, bb, Some(orientation), ty)));
        }
    } else if random_int >= 70 {
        if let Some(bb) = MineShaftStairsPiece::find_stairs(pieces, random, x, y, z, orientation) {
            return Some(Box::new(MineShaftStairsPiece::new(gen_depth, bb, Some(orientation), ty)));
        }
    } else if let Some(bb) = MineShaftCorridorPiece::find_corridor_size(pieces, random, x, y, z, orientation) {
        return Some(Box::new(MineShaftCorridorPiece::new(gen_depth, random, bb, orientation, ty)));
    }
    None
}

/// `MineShaftCorridor.addChildren`.
fn corridor_add_children(
    bb: &BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    ty: MineshaftType,
    parent_bb: &BoundingBox,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
) {
    let random_int = random.next_int_bound(4);
    if let Some(orientation) = orientation {
        match orientation {
            Direction::NORTH => {
                if random_int <= 1 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x, gy, bb.min_z - 1, orientation, gen_depth); }
                } else if random_int == 2 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, gy, bb.min_z, Direction::WEST, gen_depth); }
                } else {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, gy, bb.min_z, Direction::EAST, gen_depth); }
                }
            }
            Direction::SOUTH => {
                if random_int <= 1 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x, gy, bb.max_z + 1, orientation, gen_depth); }
                } else if random_int == 2 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, gy, bb.max_z - 3, Direction::WEST, gen_depth); }
                } else {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, gy, bb.max_z - 3, Direction::EAST, gen_depth); }
                }
            }
            Direction::WEST => {
                if random_int <= 1 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, gy, bb.min_z, orientation, gen_depth); }
                } else if random_int == 2 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x, gy, bb.min_z - 1, Direction::NORTH, gen_depth); }
                } else {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x, gy, bb.max_z + 1, Direction::SOUTH, gen_depth); }
                }
            }
            Direction::EAST => {
                if random_int <= 1 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, gy, bb.min_z, orientation, gen_depth); }
                } else if random_int == 2 {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x - 3, gy, bb.min_z - 1, Direction::NORTH, gen_depth); }
                } else {
                    { let gy = bb.min_y - 1 + random.next_int_bound(3); generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x - 3, gy, bb.max_z + 1, Direction::SOUTH, gen_depth); }
                }
            }
            _ => {}
        }
    }

    if gen_depth < MAX_DEPTH {
        if orientation != Some(Direction::NORTH) && orientation != Some(Direction::SOUTH) {
            let mut i = bb.min_x + 3;
            while i + 3 <= bb.max_x {
                let random_int1 = random.next_int_bound(5);
                if random_int1 == 0 {
                    generate_and_add_piece(parent_bb, ty, pieces, random, i, bb.min_y, bb.min_z - 1, Direction::NORTH, gen_depth + 1);
                } else if random_int1 == 1 {
                    generate_and_add_piece(parent_bb, ty, pieces, random, i, bb.min_y, bb.max_z + 1, Direction::SOUTH, gen_depth + 1);
                }
                i += 5;
            }
        } else {
            let mut ix = bb.min_z + 3;
            while ix + 3 <= bb.max_z {
                let random_int1 = random.next_int_bound(5);
                if random_int1 == 0 {
                    generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, bb.min_y, ix, Direction::WEST, gen_depth + 1);
                } else if random_int1 == 1 {
                    generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, bb.min_y, ix, Direction::EAST, gen_depth + 1);
                }
                ix += 5;
            }
        }
    }
}

/// `MineShaftCrossing.addChildren`.
fn crossing_add_children(
    bb: &BoundingBox,
    gen_depth: i32,
    direction: Direction,
    is_two_floored: bool,
    ty: MineshaftType,
    parent_bb: &BoundingBox,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
) {
    match direction {
        Direction::NORTH => {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y, bb.min_z - 1, Direction::NORTH, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, bb.min_y, bb.min_z + 1, Direction::WEST, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, bb.min_y, bb.min_z + 1, Direction::EAST, gen_depth);
        }
        Direction::SOUTH => {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y, bb.max_z + 1, Direction::SOUTH, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, bb.min_y, bb.min_z + 1, Direction::WEST, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, bb.min_y, bb.min_z + 1, Direction::EAST, gen_depth);
        }
        Direction::WEST => {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y, bb.min_z - 1, Direction::NORTH, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y, bb.max_z + 1, Direction::SOUTH, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, bb.min_y, bb.min_z + 1, Direction::WEST, gen_depth);
        }
        Direction::EAST => {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y, bb.min_z - 1, Direction::NORTH, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y, bb.max_z + 1, Direction::SOUTH, gen_depth);
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, bb.min_y, bb.min_z + 1, Direction::EAST, gen_depth);
        }
        _ => {}
    }

    if is_two_floored {
        if random.next_boolean() {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y + 3 + 1, bb.min_z - 1, Direction::NORTH, gen_depth);
        }
        if random.next_boolean() {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, bb.min_y + 3 + 1, bb.min_z + 1, Direction::WEST, gen_depth);
        }
        if random.next_boolean() {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, bb.min_y + 3 + 1, bb.min_z + 1, Direction::EAST, gen_depth);
        }
        if random.next_boolean() {
            generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x + 1, bb.min_y + 3 + 1, bb.max_z + 1, Direction::SOUTH, gen_depth);
        }
    }
}

/// `MineShaftStairs.addChildren`.
fn stairs_add_children(
    bb: &BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    ty: MineshaftType,
    parent_bb: &BoundingBox,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
) {
    if let Some(orientation) = orientation {
        match orientation {
            Direction::NORTH => {
                generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x, bb.min_y, bb.min_z - 1, Direction::NORTH, gen_depth);
            }
            Direction::SOUTH => {
                generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x, bb.min_y, bb.max_z + 1, Direction::SOUTH, gen_depth);
            }
            Direction::WEST => {
                generate_and_add_piece(parent_bb, ty, pieces, random, bb.min_x - 1, bb.min_y, bb.min_z, Direction::WEST, gen_depth);
            }
            Direction::EAST => {
                generate_and_add_piece(parent_bb, ty, pieces, random, bb.max_x + 1, bb.min_y, bb.min_z, Direction::EAST, gen_depth);
            }
            _ => {}
        }
    }
}

/// Диспетчер `expand_children` — вызывает нужный `addChildren`
/// по типу `ChildGen`.
fn expand_children(
    gen: &ChildGen,
    parent_bb: &BoundingBox,
    ty: MineshaftType,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
) {
    match gen {
        ChildGen::Corridor { bb, gen_depth, orientation, ty: _ } => {
            corridor_add_children(bb, *gen_depth, *orientation, ty, parent_bb, pieces, random);
        }
        ChildGen::Crossing { bb, gen_depth, direction, is_two_floored, ty: _ } => {
            crossing_add_children(bb, *gen_depth, *direction, *is_two_floored, ty, parent_bb, pieces, random);
        }
        ChildGen::Stairs { bb, gen_depth, orientation, ty: _ } => {
            stairs_add_children(bb, *gen_depth, *orientation, ty, parent_bb, pieces, random);
        }
    }
}

// ------------------------------------------------------------------
// Общий trait-импл для 4 кусков
// ------------------------------------------------------------------

macro_rules! impl_piece_common {
    ($name:ident) => {
        impl $name {
            fn world_pos(&self, x: i32, y: i32, z: i32) -> BlockPos {
                super::super::structure_piece::PieceTransform {
                    bb: &self.bounding_box,
                    orientation: self.orientation,
                }
                .world_pos(x, y, z)
            }
        }
    };
}

fn piece_id(id: &str) -> Identifier {
    Identifier::minecraft(id)
}

// ------------------------------------------------------------------
// MineShaftCorridor
// ------------------------------------------------------------------

pub struct MineShaftCorridorPiece {
    pub bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    pub ty: MineshaftType,
    has_rails: bool,
    spider_corridor: bool,
    has_placed_spider: bool,
    num_sections: i32,
}

impl MineShaftCorridorPiece {
    pub fn from_nbt(tag: &CompoundTag) -> Option<Self> {
        let header = PieceHeader::read(tag)?;
        let ty = MineshaftType::by_id(tag.get_int_or("MST", 0));
        let mut piece = Self {
            bounding_box: header.bounds,
            gen_depth: header.depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            ty,
            has_rails: false,
            spider_corridor: false,
            has_placed_spider: false,
            num_sections: 0,
        };
        piece.set_orientation(header.orientation);
        piece.load_additional_save_data(tag);
        Some(piece)
    }

    pub fn new(gen_depth: i32, random: &mut dyn RandomSource, bounding_box: BoundingBox, orientation: Direction, ty: MineshaftType) -> Self {
        let mut piece = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            ty,
            has_rails: false,
            spider_corridor: false,
            has_placed_spider: false,
            num_sections: 0,
        };
        piece_set_orientation(Some(orientation), &mut piece.mirror, &mut piece.rotation);
        piece.orientation = Some(orientation);
        piece.has_rails = random.next_int_bound(3) == 0;
        piece.spider_corridor = !piece.has_rails && random.next_int_bound(23) == 0;
        piece.num_sections = if piece.orientation.map(|o| o.get_axis()) == Some(crate::core::direction::Axis::Z) {
            piece.bounding_box.get_z_span() / 5
        } else {
            piece.bounding_box.get_x_span() / 5
        };
        piece
    }

    /// `findCorridorSize`.
    pub fn find_corridor_size(
        pieces: &dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
        x: i32,
        y: i32,
        z: i32,
        direction: Direction,
    ) -> Option<BoundingBox> {
        for i in (1..=random.next_int_bound(3) + 2).rev() {
            let i1 = i * 5;
            let mut bounding_box = match direction {
                Direction::SOUTH => BoundingBox::new(0, 0, 0, 2, 2, i1 - 1),
                Direction::WEST => BoundingBox::new(-(i1 - 1), 0, 0, 0, 2, 2),
                Direction::EAST => BoundingBox::new(0, 0, 0, i1 - 1, 2, 2),
                _ => BoundingBox::new(0, 0, -(i1 - 1), 2, 2, 0),
            };
            bounding_box.move_(x, y, z);
            if pieces.find_collision_piece(&bounding_box).is_none() {
                return Some(bounding_box);
            }
        }
        None
    }

    /// `placeSupport`.
    fn place_support(
        &self,
        level: &dyn LevelReader,
        box_: &BoundingBox,
        min_x: i32,
        min_y: i32,
        z: i32,
        max_y: i32,
        max_x: i32,
        random: &mut dyn RandomSource,
    ) {
        if is_supporting_box(level, &self.bounding_box, self.orientation, min_x, max_x, max_y, z, box_) {
            let planks = self.ty.planks_state();
            let fence = self.ty.fence_state();
            let mut fence_west = fence.clone();
            fence_west.values.set_erased(
                "west",
                crate::world::level::block::state::property::StateValue::Bool(true),
            );
            let mut fence_east = fence;
            fence_east.values.set_erased(
                "east",
                crate::world::level::block::state::property::StateValue::Bool(true),
            );
            generate_box_pred(
                level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_,
                min_x, min_y, z, min_x, max_y - 1, z, &fence_west, &BlockState::air(), false,
                &can_replace(&self.bounding_box, self.orientation, self.ty),
            );
            generate_box_pred(
                level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_,
                max_x, min_y, z, max_x, max_y - 1, z, &fence_east, &BlockState::air(), false,
                &can_replace(&self.bounding_box, self.orientation, self.ty),
            );
            if random.next_int_bound(4) == 0 {
                generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x, max_y, z, min_x, max_y, z, &planks, &BlockState::air(), false, &can_replace(&self.bounding_box, self.orientation, self.ty));
                generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, max_x, max_y, z, max_x, max_y, z, &planks, &BlockState::air(), false, &can_replace(&self.bounding_box, self.orientation, self.ty));
            } else {
                generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x, max_y, z, max_x, max_y, z, &planks, &BlockState::air(), false, &can_replace(&self.bounding_box, self.orientation, self.ty));
                let torch_s = with_string_property(block_state("wall_torch"), "facing", "south");
                let torch_n = with_string_property(block_state("wall_torch"), "facing", "north");
                maybe_generate_block_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, random, 0.05, min_x + 1, max_y, z - 1, &torch_s, &can_replace(&self.bounding_box, self.orientation, self.ty));
                maybe_generate_block_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, random, 0.05, min_x + 1, max_y, z + 1, &torch_n, &can_replace(&self.bounding_box, self.orientation, self.ty));
            }
        }
    }

    /// `maybePlaceCobWeb`.
    fn maybe_place_cob_web(
        &self,
        level: &dyn LevelReader,
        box_: &BoundingBox,
        random: &mut dyn RandomSource,
        chance: f32,
        x: i32,
        y: i32,
        z: i32,
    ) {
        if is_interior(level, &self.bounding_box, self.orientation, x, y, z, box_)
            && random.next_float() < chance
            && has_sturdy_neighbours(level, &self.bounding_box, self.orientation, x, y, z, 2, box_)
        {
            place_block_pred(
                level, &self.bounding_box, self.orientation, self.mirror, self.rotation,
                block_state("cobweb"), x, y, z, box_,
                &can_replace(&self.bounding_box, self.orientation, self.ty),
            );
        }
    }

    /// `placeDoubleLowerOrUpperSupport`.
    fn place_double_lower_or_upper_support(&self, level: &dyn LevelReader, box_: &BoundingBox, x: i32, y: i32, z: i32) {
        let wood = self.ty.wood_state();
        let planks = self.ty.planks_state();
        let check = |px: i32| {
            get_block(level, &self.bounding_box, self.orientation, px, y, z, box_)
                .block
                .id
                == planks.block.id
        };
        if check(x) {
            self.fill_pillar_down_or_chain_up(level, &wood, x, y, z, box_);
        }
        if check(x + 2) {
            self.fill_pillar_down_or_chain_up(level, &wood, x + 2, y, z, box_);
        }
    }

    /// `fillPillarDownOrChainUp`.
    fn fill_pillar_down_or_chain_up(&self, level: &dyn LevelReader, state: &BlockState, x: i32, y: i32, z: i32, box_: &BoundingBox) {
        let t = super::super::structure_piece::PieceTransform { bb: &self.bounding_box, orientation: self.orientation };
        let base = t.world_pos(x, y, z);
        if !box_.is_inside_pos(base) {
            return;
        }
        let y1 = base.y;
        let mut i = 1;
        let mut flag = true;
        let mut flag1 = true;
        let min_y = level.min_build_height() + 1;
        let max_y = level.min_build_height() + level.height();
        while flag || flag1 {
            if flag {
                let world_pos = BlockPos::new(base.x, y1 - i, base.z);
                let bs = level.get_block_state(world_pos).unwrap_or_else(BlockState::air);
                let flag2 = is_replaceable_by_structures(&bs)
                    && bs.block.id.path != "lava";
                if !flag2 && can_place_column_on_top_of(level, world_pos, &bs) {
                    fill_column_between(level, state, base.x, y1 - i + 1, y1, base.z);
                    return;
                }
                flag = i <= MAX_PILLAR_HEIGHT && flag2 && world_pos.y > min_y;
            }
            if flag1 {
                let world_pos = BlockPos::new(base.x, y1 + i, base.z);
                let bs = level.get_block_state(world_pos).unwrap_or_else(BlockState::air);
                let flag2 = is_replaceable_by_structures(&bs);
                if !flag2 && can_hang_chain_below(level, world_pos, &bs) {
                    level.set_block_state(BlockPos::new(base.x, y1 + 1, base.z), &self.ty.fence_state());
                    fill_column_between(level, &block_state("iron_chain"), base.x, y1 + 2, y1 + i, base.z);
                    return;
                }
                flag1 = i <= MAX_CHAIN_HEIGHT && flag2 && world_pos.y < max_y;
            }
            i += 1;
        }
    }
}

impl_piece_common!(MineShaftCorridorPiece);

impl StructurePiece for MineShaftCorridorPiece {
    fn bounding_box(&self) -> BoundingBox { self.bounding_box.clone() }
    fn bounding_box_mut(&mut self) -> Option<&mut BoundingBox> { Some(&mut self.bounding_box) }
    fn gen_depth(&self) -> i32 { self.gen_depth }
    fn set_gen_depth(&mut self, gen_depth: i32) { self.gen_depth = gen_depth; }
    fn orientation(&self) -> Option<Direction> { self.orientation }
    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.orientation = orientation;
        piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
    }
    fn rotation(&self) -> Rotation { self.rotation }
    fn mirror(&self) -> Mirror { self.mirror }
    fn piece_type_id(&self) -> Identifier { piece_id("mscorridor") }
    fn as_any(&self) -> Option<&dyn std::any::Any> { Some(self) }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.ty = MineshaftType::by_id(tag.get_int("MST").unwrap_or(0));
        self.has_rails = tag.get_bool("hr").unwrap_or(false);
        self.spider_corridor = tag.get_bool("sc").unwrap_or(false);
        self.has_placed_spider = tag.get_bool("hps").unwrap_or(false);
        self.num_sections = tag.get_int("Num").unwrap_or(0);
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("MST", Tag::Int(type_ordinal(self.ty)));
        tag.put("hr", Tag::Byte(if self.has_rails { 1 } else { 0 }));
        tag.put("sc", Tag::Byte(if self.spider_corridor { 1 } else { 0 }));
        tag.put("hps", Tag::Byte(if self.has_placed_spider { 1 } else { 0 }));
        tag.put("Num", Tag::Int(self.num_sections));
    }

    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
    ) {
        let parent_bb = piece.bounding_box();
        corridor_add_children(
            &self.bounding_box, self.gen_depth, self.orientation, self.ty,
            &parent_bb, pieces, random,
        );
    }

    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        _structure_manager: &dyn StructureManagerAccess,
        _generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        if is_in_invalid_location(level, &self.bounding_box, box_) {
            return;
        }
        let cave_air = BlockState::air();
        let i4 = self.num_sections * 5 - 1;
        let can = can_replace(&self.bounding_box, self.orientation, self.ty);
        generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, 0, 0, 0, 2, 1, i4, &cave_air, &cave_air, false, &can);
        generate_maybe_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, random, 0.8, 0, 2, 0, 2, 2, i4, &cave_air, &cave_air, false, false, &can);
        if self.spider_corridor {
            generate_maybe_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, random, 0.6, 0, 0, 0, 2, 1, i4, &block_state("cobweb"), &cave_air, false, true, &can);
        }

        for i5 in 0..self.num_sections {
            let i6 = 2 + i5 * 5;
            self.place_support(level, box_, 0, 0, i6, 2, 2, random);
            self.maybe_place_cob_web(level, box_, random, 0.1, 0, 2, i6 - 1);
            self.maybe_place_cob_web(level, box_, random, 0.1, 2, 2, i6 - 1);
            self.maybe_place_cob_web(level, box_, random, 0.1, 0, 2, i6 + 1);
            self.maybe_place_cob_web(level, box_, random, 0.1, 2, 2, i6 + 1);
            self.maybe_place_cob_web(level, box_, random, 0.05, 0, 2, i6 - 2);
            self.maybe_place_cob_web(level, box_, random, 0.05, 2, 2, i6 - 2);
            self.maybe_place_cob_web(level, box_, random, 0.05, 0, 2, i6 + 2);
            self.maybe_place_cob_web(level, box_, random, 0.05, 2, 2, i6 + 2);
            if random.next_int_bound(100) == 0 {
                self.create_minecart_chest(level, box_, random, 2, 0, i6 - 1);
            }
            if random.next_int_bound(100) == 0 {
                self.create_minecart_chest(level, box_, random, 0, 0, i6 + 1);
            }
            if self.spider_corridor && !self.has_placed_spider {
                let i8 = i6 - 1 + random.next_int_bound(3);
                let world_pos = self.world_pos(1, 0, i8);
                if box_.is_inside_pos(world_pos)
                    && is_interior(level, &self.bounding_box, self.orientation, 1, 0, i8, box_)
                {
                    self.has_placed_spider = true;
                    // placeCraftSpawner(CAVE_SPIDER)
                    level.set_block_state(world_pos, &block_state("spawner"));
                    level.place_structure_spawner(world_pos, "cave_spider");
                }
            }
        }

        for i5 in 0..=2 {
            for i6 in 0..=i4 {
                set_planks_block(level, &self.bounding_box, self.orientation, self.ty, i5, -1, i6, box_);
            }
        }

        self.place_double_lower_or_upper_support(level, box_, 0, -1, 2);
        if self.num_sections > 1 {
            self.place_double_lower_or_upper_support(level, box_, 0, -1, i4 - 2);
        }

        if self.has_rails {
            let rail = with_string_property(block_state("rail"), "shape", "north_south");
            for i7 in 0..=i4 {
                let block = get_block(level, &self.bounding_box, self.orientation, 1, -1, i7, box_);
                if !block.is_air() && block.is_solid_render(level, self.world_pos(1, -1, i7)) {
                    let f = if is_interior(level, &self.bounding_box, self.orientation, 1, 0, i7, box_) { 0.7 } else { 0.9 };
                    maybe_generate_block_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, random, f, 1, 0, i7, &rail, &can);
                }
            }
        }
    }
}

impl MineShaftCorridorPiece {
    /// `createChest` override — minecart с сундуком вместо блока.
    fn create_minecart_chest(&self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource, x: i32, y: i32, z: i32) {
        let world_pos = self.world_pos(x, y, z);
        if box_.is_inside_pos(world_pos)
            && level.get_block_state(world_pos).map(|s| s.is_air()).unwrap_or(false)
            && level.get_block_state(world_pos.below()).map(|s| !s.is_air()).unwrap_or(false)
        {
            let rail = with_string_property(block_state("rail"), "shape", if random.next_boolean() { "north_south" } else { "east_west" });
            place_block_pred(
                level, &self.bounding_box, self.orientation, self.mirror, self.rotation,
                rail, x, y, z, box_, &can_replace(&self.bounding_box, self.orientation, self.ty),
            );
            level.spawn_minecart_chest(
                crate::world::phys::vec3::Vec3::new(
                    world_pos.x as f64 + 0.5,
                    world_pos.y as f64 + 0.5,
                    world_pos.z as f64 + 0.5,
                ),
                &Identifier::minecraft("chests/abandoned_mineshaft"),
                random.next_long(),
            );
        }
    }
}

// ------------------------------------------------------------------
// MineShaftCrossing
// ------------------------------------------------------------------

pub struct MineShaftCrossingPiece {
    pub bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    pub ty: MineshaftType,
    direction: Direction,
    is_two_floored: bool,
}

impl MineShaftCrossingPiece {
    pub fn from_nbt(tag: &CompoundTag) -> Option<Self> {
        let header = PieceHeader::read(tag)?;
        let ty = MineshaftType::by_id(tag.get_int_or("MST", 0));
        let mut piece = Self::new(header.depth, header.bounds, None, ty);
        piece.set_orientation(header.orientation);
        piece.load_additional_save_data(tag);
        Some(piece)
    }

    pub fn new(gen_depth: i32, bounding_box: BoundingBox, direction: Option<Direction>, ty: MineshaftType) -> Self {
        let is_two_floored = bounding_box.get_y_span() > 3;
        Self {
            bounding_box,
            gen_depth,
            orientation: direction,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            ty,
            direction: direction.unwrap_or(Direction::SOUTH),
            is_two_floored,
        }
    }

    /// `findCrossing`.
    pub fn find_crossing(
        pieces: &dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
        x: i32,
        y: i32,
        z: i32,
        direction: Direction,
    ) -> Option<BoundingBox> {
        let i = if random.next_int_bound(4) == 0 { 6 } else { 2 };
        let mut bounding_box = match direction {
            Direction::SOUTH => BoundingBox::new(-1, 0, 0, 3, i, 4),
            Direction::WEST => BoundingBox::new(-4, 0, -1, 0, i, 3),
            Direction::EAST => BoundingBox::new(0, 0, -1, 4, i, 3),
            _ => BoundingBox::new(-1, 0, -4, 3, i, 0),
        };
        bounding_box.move_(x, y, z);
        if pieces.find_collision_piece(&bounding_box).is_some() {
            None
        } else {
            Some(bounding_box)
        }
    }

    /// `placeSupportPillar`.
    fn place_support_pillar(&self, level: &dyn LevelReader, box_: &BoundingBox, x: i32, y: i32, z: i32, max_y: i32) {
        if !get_block(level, &self.bounding_box, self.orientation, x, max_y + 1, z, box_).is_air() {
            generate_box_pred(
                level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_,
                x, y, z, x, max_y, z, &self.ty.planks_state(), &BlockState::air(), false,
                &can_replace(&self.bounding_box, self.orientation, self.ty),
            );
        }
    }
}

impl_piece_common!(MineShaftCrossingPiece);

impl StructurePiece for MineShaftCrossingPiece {
    fn bounding_box(&self) -> BoundingBox { self.bounding_box.clone() }
    fn bounding_box_mut(&mut self) -> Option<&mut BoundingBox> { Some(&mut self.bounding_box) }
    fn gen_depth(&self) -> i32 { self.gen_depth }
    fn set_gen_depth(&mut self, gen_depth: i32) { self.gen_depth = gen_depth; }
    fn orientation(&self) -> Option<Direction> { self.orientation }
    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.orientation = orientation;
        piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
    }
    fn rotation(&self) -> Rotation { self.rotation }
    fn mirror(&self) -> Mirror { self.mirror }
    fn piece_type_id(&self) -> Identifier { piece_id("mscrossing") }
    fn as_any(&self) -> Option<&dyn std::any::Any> { Some(self) }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.ty = MineshaftType::by_id(tag.get_int("MST").unwrap_or(0));
        self.is_two_floored = tag.get_bool("tf").unwrap_or(false);
        if let Some(v) = tag.get_int("D") {
            self.direction = Direction::from_2d_data_value(v);
        }
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("MST", Tag::Int(type_ordinal(self.ty)));
        tag.put("tf", Tag::Byte(if self.is_two_floored { 1 } else { 0 }));
        tag.put("D", Tag::Int(self.direction.get_2d_data_value()));
    }

    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
    ) {
        let parent_bb = piece.bounding_box();
        crossing_add_children(
            &self.bounding_box, self.gen_depth, self.direction, self.is_two_floored, self.ty,
            &parent_bb, pieces, random,
        );
    }

    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        _structure_manager: &dyn StructureManagerAccess,
        _generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        let _ = random;
        if is_in_invalid_location(level, &self.bounding_box, box_) {
            return;
        }
        let can = can_replace(&self.bounding_box, self.orientation, self.ty);
        let (min_x, min_y, min_z) = (self.bounding_box.min_x, self.bounding_box.min_y, self.bounding_box.min_z);
        let (max_x, max_y, max_z) = (self.bounding_box.max_x, self.bounding_box.max_y, self.bounding_box.max_z);
        if self.is_two_floored {
            generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x + 1, min_y, min_z, max_x - 1, min_y + 3 - 1, max_z, &BlockState::air(), &BlockState::air(), false, &can);
            generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x, min_y, min_z + 1, max_x, min_y + 3 - 1, max_z - 1, &BlockState::air(), &BlockState::air(), false, &can);
            generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x + 1, max_y - 2, min_z, max_x - 1, max_y, max_z, &BlockState::air(), &BlockState::air(), false, &can);
            generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x, max_y - 2, min_z + 1, max_x, max_y, max_z - 1, &BlockState::air(), &BlockState::air(), false, &can);
            generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x + 1, min_y + 3, min_z + 1, max_x - 1, min_y + 3, max_z - 1, &BlockState::air(), &BlockState::air(), false, &can);
        } else {
            generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x + 1, min_y, min_z, max_x - 1, max_y, max_z, &BlockState::air(), &BlockState::air(), false, &can);
            generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, min_x, min_y, min_z + 1, max_x, max_y, max_z - 1, &BlockState::air(), &BlockState::air(), false, &can);
        }
        self.place_support_pillar(level, box_, min_x + 1, min_y, min_z + 1, max_y);
        self.place_support_pillar(level, box_, min_x + 1, min_y, max_z - 1, max_y);
        self.place_support_pillar(level, box_, max_x - 1, min_y, min_z + 1, max_y);
        self.place_support_pillar(level, box_, max_x - 1, min_y, max_z - 1, max_y);
        let i = min_y - 1;
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                set_planks_block(level, &self.bounding_box, self.orientation, self.ty, x, i, z, box_);
            }
        }
    }
}

// ------------------------------------------------------------------
// MineShaftRoom
// ------------------------------------------------------------------

pub struct MineShaftRoomPiece {
    pub bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    pub ty: MineshaftType,
    /// `childEntranceBoxes` — shared с генератором (комната
    /// добавляется в pieces до вызова addChildren, а boxes
    /// нужны её postProcess — Rc<RefCell> позволяет заполнить
    /// через общий handle).
    pub child_entrance_boxes: Rc<RefCell<Vec<BoundingBox>>>,
}

impl MineShaftRoomPiece {
    pub fn from_nbt(tag: &CompoundTag) -> Option<Self> {
        let header = PieceHeader::read(tag)?;
        let ty = MineshaftType::by_id(tag.get_int_or("MST", 0));
        let mut piece = Self {
            bounding_box: header.bounds,
            gen_depth: header.depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            ty,
            child_entrance_boxes: Rc::new(RefCell::new(Vec::new())),
        };
        piece.set_orientation(header.orientation);
        piece.load_additional_save_data(tag);
        Some(piece)
    }

    /// `new(genDepth, random, x, z, type)` — комната 7-13 × 4-9 × 7-13
    /// на y=50.
    pub fn new(gen_depth: i32, random: &mut dyn RandomSource, x: i32, z: i32, ty: MineshaftType) -> Self {
        Self {
            bounding_box: BoundingBox::new(
                x, MAGIC_START_Y, z,
                x + 7 + random.next_int_bound(6),
                54 + random.next_int_bound(6),
                z + 7 + random.next_int_bound(6),
            ),
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            ty,
            child_entrance_boxes: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl_piece_common!(MineShaftRoomPiece);

impl StructurePiece for MineShaftRoomPiece {
    fn bounding_box(&self) -> BoundingBox { self.bounding_box.clone() }
    fn bounding_box_mut(&mut self) -> Option<&mut BoundingBox> { Some(&mut self.bounding_box) }
    fn gen_depth(&self) -> i32 { self.gen_depth }
    fn set_gen_depth(&mut self, gen_depth: i32) { self.gen_depth = gen_depth; }
    fn orientation(&self) -> Option<Direction> { self.orientation }
    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.orientation = orientation;
        piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
    }
    fn rotation(&self) -> Rotation { self.rotation }
    fn mirror(&self) -> Mirror { self.mirror }
    fn piece_type_id(&self) -> Identifier { piece_id("msroom") }
    fn as_any(&self) -> Option<&dyn std::any::Any> { Some(self) }

    fn move_by(&mut self, x: i32, y: i32, z: i32) {
        self.bounding_box.move_(x, y, z);
        for b in self.child_entrance_boxes.borrow_mut().iter_mut() {
            b.move_(x, y, z);
        }
    }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.ty = MineshaftType::by_id(tag.get_int("MST").unwrap_or(0));
        let mut boxes = self.child_entrance_boxes.borrow_mut();
        boxes.clear();
        for entry in tag.get_list_or_empty("Entrances") {
            if let Some(&[x0, y0, z0, x1, y1, z1]) = entry.as_int_array() {
                boxes.push(BoundingBox::new(x0, y0, z0, x1, y1, z1));
            }
        }
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("MST", Tag::Int(type_ordinal(self.ty)));
        let list: Vec<Tag> = self
            .child_entrance_boxes
            .borrow()
            .iter()
            .map(|b| {
                Tag::IntArray(vec![
                    b.min_x, b.min_y, b.min_z, b.max_x, b.max_y, b.max_z,
                ])
            })
            .collect();
        tag.put("Entrances", Tag::List(list));
    }

    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
    ) {
        let parent_bb = piece.bounding_box();
        let bb = self.bounding_box.clone();
        let ty = self.ty;
        let depth = self.gen_depth;
        let mut entrances = self.child_entrance_boxes.borrow_mut();
        room_expand(&bb, depth, ty, &parent_bb, pieces, random, &mut entrances);
    }

    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        _structure_manager: &dyn StructureManagerAccess,
        _generator: Option<&dyn PoolGeneratorAccess>,
        _random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        if is_in_invalid_location(level, &self.bounding_box, box_) {
            return;
        }
        let cave_air = BlockState::air();
        generate_box_pred(
            level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_,
            self.bounding_box.min_x, self.bounding_box.min_y + 1, self.bounding_box.min_z,
            self.bounding_box.max_x,
            (self.bounding_box.min_y + 3).min(self.bounding_box.max_y),
            self.bounding_box.max_z,
            &cave_air, &cave_air, false,
            &can_replace(&self.bounding_box, self.orientation, self.ty),
        );
        let entrances = self.child_entrance_boxes.borrow().clone();
        for b in &entrances {
            generate_box_pred(
                level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_,
                b.min_x, b.max_y - 2, b.min_z, b.max_x, b.max_y, b.max_z,
                &cave_air, &cave_air, false,
                &can_replace(&self.bounding_box, self.orientation, self.ty),
            );
        }
        generate_upper_half_sphere_pred(
            level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_,
            self.bounding_box.min_x, self.bounding_box.min_y + 4, self.bounding_box.min_z,
            self.bounding_box.max_x, self.bounding_box.max_y, self.bounding_box.max_z,
            &cave_air, false,
            &can_replace(&self.bounding_box, self.orientation, self.ty),
        );
    }
}

// ------------------------------------------------------------------
// MineShaftStairs
// ------------------------------------------------------------------

pub struct MineShaftStairsPiece {
    pub bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    pub ty: MineshaftType,
}

impl MineShaftStairsPiece {
    pub fn from_nbt(tag: &CompoundTag) -> Option<Self> {
        let header = PieceHeader::read(tag)?;
        let ty = MineshaftType::by_id(tag.get_int_or("MST", 0));
        Some(Self::new(header.depth, header.bounds, header.orientation, ty))
    }

    pub fn new(gen_depth: i32, bounding_box: BoundingBox, orientation: Option<Direction>, ty: MineshaftType) -> Self {
        let mut piece = Self {
            bounding_box,
            gen_depth,
            orientation,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            ty,
        };
        piece_set_orientation(orientation, &mut piece.mirror, &mut piece.rotation);
        piece
    }

    /// `findStairs`.
    pub fn find_stairs(
        pieces: &dyn StructurePieceAccessor,
        _random: &mut dyn RandomSource,
        x: i32,
        y: i32,
        z: i32,
        direction: Direction,
    ) -> Option<BoundingBox> {
        let mut bounding_box = match direction {
            Direction::SOUTH => BoundingBox::new(0, -5, 0, 2, 2, 8),
            Direction::WEST => BoundingBox::new(-8, -5, 0, 0, 2, 2),
            Direction::EAST => BoundingBox::new(0, -5, 0, 8, 2, 2),
            _ => BoundingBox::new(0, -5, -8, 2, 2, 0),
        };
        bounding_box.move_(x, y, z);
        if pieces.find_collision_piece(&bounding_box).is_some() {
            None
        } else {
            Some(bounding_box)
        }
    }
}

impl_piece_common!(MineShaftStairsPiece);

impl StructurePiece for MineShaftStairsPiece {
    fn bounding_box(&self) -> BoundingBox { self.bounding_box.clone() }
    fn bounding_box_mut(&mut self) -> Option<&mut BoundingBox> { Some(&mut self.bounding_box) }
    fn gen_depth(&self) -> i32 { self.gen_depth }
    fn set_gen_depth(&mut self, gen_depth: i32) { self.gen_depth = gen_depth; }
    fn orientation(&self) -> Option<Direction> { self.orientation }
    fn set_orientation(&mut self, orientation: Option<Direction>) {
        self.orientation = orientation;
        piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
    }
    fn rotation(&self) -> Rotation { self.rotation }
    fn mirror(&self) -> Mirror { self.mirror }
    fn piece_type_id(&self) -> Identifier { piece_id("msstairs") }
    fn as_any(&self) -> Option<&dyn std::any::Any> { Some(self) }

    fn load_additional_save_data(&mut self, tag: &CompoundTag) {
        self.ty = MineshaftType::by_id(tag.get_int("MST").unwrap_or(0));
    }

    fn add_additional_save_data(&self, tag: &mut CompoundTag) {
        tag.put("MST", Tag::Int(type_ordinal(self.ty)));
    }

    fn add_children(
        &mut self,
        piece: &mut dyn StructurePiece,
        pieces: &mut dyn StructurePieceAccessor,
        random: &mut dyn RandomSource,
    ) {
        let parent_bb = piece.bounding_box();
        stairs_add_children(
            &self.bounding_box, self.gen_depth, self.orientation, self.ty,
            &parent_bb, pieces, random,
        );
    }

    fn post_process(
        &mut self,
        level: &dyn LevelReader,
        _structure_manager: &dyn StructureManagerAccess,
        _generator: Option<&dyn PoolGeneratorAccess>,
        _random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        if is_in_invalid_location(level, &self.bounding_box, box_) {
            return;
        }
        let can = can_replace(&self.bounding_box, self.orientation, self.ty);
        let air = BlockState::air();
        generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, 0, 5, 0, 2, 7, 1, &air, &air, false, &can);
        generate_box_pred(level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_, 0, 0, 7, 2, 2, 8, &air, &air, false, &can);
        for i in 0..5 {
            generate_box_pred(
                level, &self.bounding_box, self.orientation, self.mirror, self.rotation, box_,
                0, 5 - i - if i < 4 { 1 } else { 0 }, 2 + i,
                2, 7 - i, 2 + i,
                &air, &air, false, &can,
            );
        }
    }
}

// ------------------------------------------------------------------
// MineShaftRoom — особый случай: entrance_boxes собираются при
// генерации и читаются в postProcess.
// ------------------------------------------------------------------

/// `MineShaftRoom.addChildren` — собирает entrance boxes.
fn room_expand(
    bb: &BoundingBox,
    gen_depth: i32,
    ty: MineshaftType,
    parent_bb: &BoundingBox,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    entrance_boxes: &mut Vec<BoundingBox>,
) {
    let mut i = bb.get_y_span() - 3 - 1;
    if i <= 0 {
        i = 1;
    }

    let mut i1 = 0;
    while i1 < bb.get_x_span() {
        i1 += random.next_int_bound(bb.get_x_span());
        if i1 + 3 > bb.get_x_span() {
            break;
        }
        let gy = bb.min_y + random.next_int_bound(i) + 1;
        if let Some(child) = generate_and_add_piece(
            parent_bb, ty, pieces, random,
            bb.min_x + i1,
            gy,
            bb.min_z - 1,
            Direction::NORTH,
            gen_depth,
        ) {
            entrance_boxes.push(BoundingBox::new(child.min_x, child.min_y, bb.min_z, child.max_x, child.max_y, bb.min_z + 1));
        }
        i1 += 4;
    }

    let mut i1 = 0;
    while i1 < bb.get_x_span() {
        i1 += random.next_int_bound(bb.get_x_span());
        if i1 + 3 > bb.get_x_span() {
            break;
        }
        let gy = bb.min_y + random.next_int_bound(i) + 1;
        if let Some(child) = generate_and_add_piece(
            parent_bb, ty, pieces, random,
            bb.min_x + i1,
            gy,
            bb.max_z + 1,
            Direction::SOUTH,
            gen_depth,
        ) {
            entrance_boxes.push(BoundingBox::new(child.min_x, child.min_y, bb.max_z - 1, child.max_x, child.max_y, bb.max_z));
        }
        i1 += 4;
    }

    let mut i1 = 0;
    while i1 < bb.get_z_span() {
        i1 += random.next_int_bound(bb.get_z_span());
        if i1 + 3 > bb.get_z_span() {
            break;
        }
        let gy = bb.min_y + random.next_int_bound(i) + 1;
        if let Some(child) = generate_and_add_piece(
            parent_bb, ty, pieces, random,
            bb.min_x - 1,
            gy,
            bb.min_z + i1,
            Direction::WEST,
            gen_depth,
        ) {
            entrance_boxes.push(BoundingBox::new(bb.min_x, child.min_y, child.min_z, bb.min_x + 1, child.max_y, child.max_z));
        }
        i1 += 4;
    }

    let mut i1 = 0;
    while i1 < bb.get_z_span() {
        i1 += random.next_int_bound(bb.get_z_span());
        if i1 + 3 > bb.get_z_span() {
            break;
        }
        let gy = bb.min_y + random.next_int_bound(i) + 1;
        if let Some(child) = generate_and_add_piece(
            parent_bb, ty, pieces, random,
            bb.max_x + 1,
            gy,
            bb.min_z + i1,
            Direction::EAST,
            gen_depth,
        ) {
            entrance_boxes.push(BoundingBox::new(bb.max_x - 1, child.min_y, child.min_z, bb.max_x, child.max_y, child.max_z));
        }
        i1 += 4;
    }
}

// ------------------------------------------------------------------
// MineshaftStructure
// ------------------------------------------------------------------

/// `mineshaft_structure(id, settings, type)` — фабрика
/// `MineshaftStructure`.
pub fn mineshaft_structure(
    id: Identifier,
    settings: StructureSettings,
    ty: MineshaftType,
) -> Structure {
    let structure_type = register_structure_type(Identifier::minecraft("mineshaft"));
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point = Some(Arc::new(move |context: &mut GenerationContext| {
        // `context.random().nextDouble()` — потребляем как в Java.
        let _ = context.random.next_double();
        let chunk_pos = context.chunk_pos;
        let block_pos = BlockPos::new(
            chunk_pos.get_middle_block_x(),
            MAGIC_START_Y,
            chunk_pos.get_min_block_z(),
        );

        let mut pieces: Vec<Box<dyn StructurePiece>> = Vec::new();
        // `chunkPos.getBlockX(2)` = minBlockX + 2*16.
        let room_x = chunk_pos.get_min_block_x() + 32;
        let room_z = chunk_pos.get_min_block_z() + 32;
        let room = MineShaftRoomPiece::new(0, &mut context.random, room_x, room_z, ty);
        let entrances = room.child_entrance_boxes.clone();
        let room_bb = room.bounding_box();
        pieces.push(Box::new(room));
        // `mineShaftRoom.addChildren(mineShaftRoom, builder, random)`
        {
            let mut entrances_mut = entrances.borrow_mut();
            room_expand(
                &room_bb,
                0,
                ty,
                &room_bb,
                &mut pieces,
                &mut context.random,
                &mut entrances_mut,
            );
        }

        let sea_level = context.level.sea_level();
        let min_y = context.level.min_build_height();
        let offset = if ty == MineshaftType::Mesa {
            // MESA: baseHeight(WORLD_SURFACE_WG) в центре →
            // offsetPiecesVertically.
            let center = pieces_bounding_box(&pieces).get_center();
            let base_height = context.level.get_base_height(
                HeightmapType::WorldSurfaceWg,
                center.x,
                center.z,
            );
            let i = if base_height <= sea_level {
                sea_level
            } else {
                random_between_inclusive(&mut context.random, sea_level, base_height)
            };
            let i1 = i - center.y;
            offset_pieces_vertically(&mut pieces, i1);
            i1
        } else {
            move_below_sea_level(&mut pieces, sea_level, min_y, &mut context.random, 10)
        };

        let pos = block_pos.offset(0, offset, 0);
        Some(GenerationStub::new(pos, move |out, _ctx| {
            out.append(&mut pieces);
        }))
    }));
    structure
}

