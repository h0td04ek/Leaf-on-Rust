//! Перенос `structures/StrongholdStructure.java` +
//! `structures/StrongholdPieces.java` — спонсор/крепость:
//! `StartPiece` (StairsDown-источник 5x11x5 @ y=64), weighted-выбор
//! кусков (`PieceWeight` с `doPlace(genDepth)`-переопределениями
//! Library/PortalRoom), `imposedPiece` (FiveCrossing после
//! стартовой лестницы), `pendingChildren`-цикл до появления
//! PortalRoom, `moveBelowSeaLevel(seaLevel, minY, random, 10)` и
//! все 13 типов кусков с точной геометрией (коридоры, библиотека,
//! комната портала с end portal frame + silverfish-спавнером,
//! тюремный зал, FillerCorridor-fallback).

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{
    create_chest, generate_box, generate_box_selector, generate_maybe_box,
    get_random_horizontal_direction, make_bounding_box, maybe_generate_block,
    move_below_sea_level, piece_set_orientation, place_block, BlockSelector,
    PieceTransform, StructurePiece, StructurePieceAccessor,
};
use super::super::structure_type::register_structure_type;
use super::super::pools::structure_pool_element::PoolGeneratorAccess;
use super::{block_state, with_bool_property, with_string_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;

/// `MAX_DEPTH`.
const MAX_DEPTH: i32 = 50;
/// `LOWEST_Y_POSITION` (`isOkBox`).
const LOWEST_Y_POSITION: i32 = 10;
/// `MAGIC_START_Y`.
const MAGIC_START_Y: i32 = 64;
/// `moveBelowSeaLevel` amount.
const BELOW_SEA_AMOUNT: i32 = 10;
/// `BuiltInLootTables.STRONGHOLD_CORRIDOR`.
const STRONGHOLD_CORRIDOR_LOOT: &str = "chests/stronghold_corridor";
/// `BuiltInLootTables.STRONGHOLD_LIBRARY`.
const STRONGHOLD_LIBRARY_LOOT: &str = "chests/stronghold_library";
/// `BuiltInLootTables.STRONGHOLD_CROSSING`.
const STRONGHOLD_CROSSING_LOOT: &str = "chests/stronghold_crossing";

// ======================================================================
// SmallDoorType / PieceKind / PieceWeight / StartPieceData
// ======================================================================

/// `StrongholdPiece.SmallDoorType`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum SmallDoorType {
    Opening,
    WoodDoor,
    Grates,
    IronDoor,
}

impl SmallDoorType {
    fn name(self) -> &'static str {
        match self {
            Self::Opening => "opening",
            Self::WoodDoor => "wood_door",
            Self::Grates => "grates",
            Self::IronDoor => "iron_door",
        }
    }
    fn from_name(s: &str) -> Self {
        match s {
            "wood_door" => Self::WoodDoor,
            "grates" => Self::Grates,
            "iron_door" => Self::IronDoor,
            _ => Self::Opening,
        }
    }
}

/// `randomSmallDoor` — `nextInt(5)`: 0,1,default→OPENING,
/// 2→WOOD_DOOR, 3→GRATES, 4→IRON_DOOR.
fn random_small_door(random: &mut dyn RandomSource) -> SmallDoorType {
    match random.next_int_bound(5) {
        2 => SmallDoorType::WoodDoor,
        3 => SmallDoorType::Grates,
        4 => SmallDoorType::IronDoor,
        _ => SmallDoorType::Opening,
    }
}

/// `PieceWeight.pieceClass` как enum (11 weighted-типов;
/// `FillerCorridor` создаётся напрямую в fallback).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum PieceKind {
    Straight,
    PrisonHall,
    LeftTurn,
    RightTurn,
    RoomCrossing,
    StraightStairsDown,
    StairsDown,
    FiveCrossing,
    ChestCorridor,
    Library,
    PortalRoom,
}

/// `PieceWeight` — `placeCount` через `Cell` (shared по `Rc`,
/// как Java-мутация `currentPieces` внутри `StartPiece`).
/// `min_gen_depth` — переопределение `doPlace(genDepth)`:
/// Library требует `genDepth > 4`, PortalRoom `genDepth > 5`;
/// -1 — без проверки глубины.
struct PieceWeight {
    kind: PieceKind,
    weight: i32,
    place_count: Cell<i32>,
    max_place_count: i32,
    min_gen_depth: i32,
}

type Weight = Rc<PieceWeight>;

fn weight(kind: PieceKind, weight: i32, max_place_count: i32, min_gen_depth: i32) -> Weight {
    Rc::new(PieceWeight {
        kind,
        weight,
        place_count: Cell::new(0),
        max_place_count,
        min_gen_depth,
    })
}

impl PieceWeight {
    /// `doPlace(genDepth)`.
    fn do_place(&self, gen_depth: i32) -> bool {
        (self.max_place_count == 0 || self.place_count.get() < self.max_place_count)
            && gen_depth > self.min_gen_depth
    }
    /// `isValid`.
    fn is_valid(&self) -> bool {
        self.max_place_count == 0 || self.place_count.get() < self.max_place_count
    }
}

/// Флаги куска, нужные `addChildren` у родителя (Java читает
/// поля экземпляра: FiveCrossing leftLow/…, Straight leftChild/…).
/// Пробрасываются через `PendingChild` при добавлении.
#[derive(Clone, Copy, Default)]
struct ChildFlags {
    f: [bool; 4],
}

/// `StartPiece.pendingChildren` — метаданные куска
/// (bb/orientation/genDepth/type/flags), т.к. `Vec<Box<dyn>>`
/// не позволяет вызвать `addChildren` на элементах по ссылке.
struct PendingChild {
    ty: Identifier,
    bb: BoundingBox,
    orientation: Option<Direction>,
    gen_depth: i32,
    flags: ChildFlags,
}

/// Данные `StartPiece` (`currentPieces`, `previousPiece`,
/// `imposedPiece`, `pendingChildren`, `portalRoomPiece`).
/// `RefCell` — доступ через `&StartPiece` в `add_children` и
/// из драйвера. `resetPieces()` Java = новый `StartPieceData`
/// на каждую попытку генерации.
struct StartPieceData {
    current_pieces: Vec<Weight>,
    previous_piece: Option<Weight>,
    imposed_piece: Option<PieceKind>,
    pending_children: Vec<PendingChild>,
    start_bb: BoundingBox,
    portal_room_placed: bool,
}

impl StartPieceData {
    fn new() -> Self {
        Self {
            current_pieces: vec![
                weight(PieceKind::Straight, 40, 0, -1),
                weight(PieceKind::PrisonHall, 5, 5, -1),
                weight(PieceKind::LeftTurn, 20, 0, -1),
                weight(PieceKind::RightTurn, 20, 0, -1),
                weight(PieceKind::RoomCrossing, 10, 6, -1),
                weight(PieceKind::StraightStairsDown, 5, 5, -1),
                weight(PieceKind::StairsDown, 5, 5, -1),
                weight(PieceKind::FiveCrossing, 5, 4, -1),
                weight(PieceKind::ChestCorridor, 5, 4, -1),
                weight(PieceKind::Library, 10, 2, 4),
                weight(PieceKind::PortalRoom, 20, 1, 5),
            ],
            previous_piece: None,
            imposed_piece: None,
            pending_children: Vec::new(),
            start_bb: BoundingBox::new(0, 0, 0, -1, -1, -1),
            portal_room_placed: false,
        }
    }
}

// ======================================================================
// Дочерние куски: generateSmallDoorChildForward/Left/Right,
// generateAndAddPiece, generatePieceFromSmallDoor
// ======================================================================

/// `generateSmallDoorChildForward` — позиция ребёнка перед куском.
/// Java-аргументы `(offsetX, offsetY)`.
#[allow(clippy::too_many_arguments)]
fn child_forward(
    bb: &BoundingBox,
    orientation: Option<Direction>,
    gen_depth: i32,
    data: &Rc<RefCell<StartPieceData>>,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    offset_x: i32,
    offset_y: i32,
) {
    let Some(o) = orientation else { return };
    let (x, y, z) = match o {
        Direction::NORTH => (bb.min_x + offset_x, bb.min_y + offset_y, bb.min_z - 1),
        Direction::SOUTH => (bb.min_x + offset_x, bb.min_y + offset_y, bb.max_z + 1),
        Direction::WEST => (bb.min_x - 1, bb.min_y + offset_y, bb.min_z + offset_x),
        Direction::EAST => (bb.max_x + 1, bb.min_y + offset_y, bb.min_z + offset_x),
        _ => return,
    };
    generate_and_add_piece(data, pieces, random, x, y, z, o, gen_depth);
}

/// `generateSmallDoorChildLeft` — позиция ребёнка слева.
/// Java-аргументы `(offsetY, offsetX)` — offsetY идёт в minY,
/// offsetX в minZ/minX.
#[allow(clippy::too_many_arguments)]
fn child_left(
    bb: &BoundingBox,
    orientation: Option<Direction>,
    gen_depth: i32,
    data: &Rc<RefCell<StartPieceData>>,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    offset_y: i32,
    offset_x: i32,
) {
    let Some(o) = orientation else { return };
    let (x, y, z, child_o) = match o {
        Direction::NORTH | Direction::SOUTH => {
            (bb.min_x - 1, bb.min_y + offset_y, bb.min_z + offset_x, Direction::WEST)
        }
        Direction::WEST | Direction::EAST => {
            (bb.min_x + offset_x, bb.min_y + offset_y, bb.min_z - 1, Direction::NORTH)
        }
        _ => return,
    };
    generate_and_add_piece(data, pieces, random, x, y, z, child_o, gen_depth);
}

/// `generateSmallDoorChildRight` — позиция ребёнка справа.
/// Java-аргументы `(offsetY, offsetX)`.
#[allow(clippy::too_many_arguments)]
fn child_right(
    bb: &BoundingBox,
    orientation: Option<Direction>,
    gen_depth: i32,
    data: &Rc<RefCell<StartPieceData>>,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    offset_y: i32,
    offset_x: i32,
) {
    let Some(o) = orientation else { return };
    let (x, y, z, child_o) = match o {
        Direction::NORTH | Direction::SOUTH => {
            (bb.max_x + 1, bb.min_y + offset_y, bb.min_z + offset_x, Direction::EAST)
        }
        Direction::WEST | Direction::EAST => {
            (bb.min_x + offset_x, bb.min_y + offset_y, bb.max_z + 1, Direction::SOUTH)
        }
        _ => return,
    };
    generate_and_add_piece(data, pieces, random, x, y, z, child_o, gen_depth);
}

/// `generateAndAddPiece` — `genDepth > 50` → null; 112-блоковая
/// проверка от start-bb; `generatePieceFromSmallDoor(genDepth+1)`;
/// addPiece + pendingChildren. Вне диапазона — null (без
/// fallback-куска, в отличие от крепости).
#[allow(clippy::too_many_arguments)]
fn generate_and_add_piece(
    data: &Rc<RefCell<StartPieceData>>,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    orientation: Direction,
    gen_depth: i32,
) {
    if gen_depth > MAX_DEPTH {
        return;
    }
    let start_bb = data.borrow().start_bb;
    if (x - start_bb.min_x).abs() <= 112 && (z - start_bb.min_z).abs() <= 112 {
        if let Some(piece) =
            generate_piece_from_small_door(data, pieces, random, x, y, z, orientation, gen_depth + 1)
        {
            let pending = PendingChild {
                ty: piece.piece_type_id(),
                bb: piece.bounding_box(),
                orientation: piece.orientation(),
                gen_depth: piece.gen_depth(),
                flags: child_flags_of(piece.as_ref()),
            };
            pieces.add_piece(piece);
            data.borrow_mut().pending_children.push(pending);
        }
    }
}

/// `updatePieceWeight` — `flag` если есть placeable weight с
/// `maxPlaceCount > 0`; `totalWeight` — сумма весов ВСЕХ
/// текущих кусков. `None` ↔ Java `false`.
fn update_piece_weight(weights: &[Weight]) -> Option<i32> {
    let mut flag = false;
    let mut total = 0;
    for w in weights {
        if w.max_place_count > 0 && w.place_count.get() < w.max_place_count {
            flag = true;
        }
        total += w.weight;
    }
    if flag {
        Some(total)
    } else {
        None
    }
}

/// `generatePieceFromSmallDoor` — `imposedPiece` в первую
/// очередь, затем weighted-цикл ×5 (`pieceWeight ==
/// previousPiece` → break, без `allowInRow`), fallback
/// `FillerCorridor.findPieceBox`.
#[allow(clippy::too_many_arguments)]
fn generate_piece_from_small_door(
    data: &Rc<RefCell<StartPieceData>>,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    orientation: Direction,
    gen_depth: i32,
) -> Option<Box<dyn StructurePiece>> {
    let total = {
        let d = data.borrow();
        update_piece_weight(&d.current_pieces)?
    };

    let imposed = data.borrow_mut().imposed_piece.take();
    if let Some(kind) = imposed {
        if let Some(piece) = create_piece_kind(kind, pieces, random, x, y, z, orientation, gen_depth) {
            return Some(piece);
        }
    }

    let weights: Vec<Weight> = data.borrow().current_pieces.clone();
    let mut i = 0;
    while i < 5 {
        i += 1;
        let mut random_int = random.next_int_bound(total);
        for w in &weights {
            random_int -= w.weight;
            if random_int < 0 {
                let is_prev = data
                    .borrow()
                    .previous_piece
                    .as_ref()
                    .map(|p| Rc::ptr_eq(p, w))
                    .unwrap_or(false);
                if !w.do_place(gen_depth) || is_prev {
                    break;
                }
                if let Some(piece) = create_piece_kind(w.kind, pieces, random, x, y, z, orientation, gen_depth)
                {
                    w.place_count.set(w.place_count.get() + 1);
                    data.borrow_mut().previous_piece = Some(w.clone());
                    if !w.is_valid() {
                        let mut d = data.borrow_mut();
                        if let Some(pos) = d.current_pieces.iter().position(|x| Rc::ptr_eq(x, w)) {
                            d.current_pieces.remove(pos);
                        }
                    }
                    return Some(piece);
                }
            }
        }
    }

    let bb = filler_find_piece_box(pieces, x, y, z, orientation)?;
    if bb.min_y > 1 {
        Some(Box::new(FillerCorridor::new(gen_depth, bb, orientation)) as Box<dyn StructurePiece>)
    } else {
        None
    }
}

/// `FillerCorridor.findPieceBox` — подбирает глубину 1-3 под
/// коллизию куска на minY.
fn filler_find_piece_box(
    pieces: &dyn StructurePieceAccessor,
    x: i32,
    y: i32,
    z: i32,
    orientation: Direction,
) -> Option<BoundingBox> {
    let bb = BoundingBox::orient_box_offsets(x, y, z, -1, -1, 0, 5, 5, 4, orientation);
    let piece = pieces.find_collision_piece(&bb)?;
    if piece.bounding_box().min_y == bb.min_y {
        for i1 in [2, 1] {
            let bb2 = BoundingBox::orient_box_offsets(x, y, z, -1, -1, 0, 5, 5, i1, orientation);
            if !piece.bounding_box().intersects(&bb2) {
                return Some(BoundingBox::orient_box_offsets(x, y, z, -1, -1, 0, 5, 5, i1 + 1, orientation));
            }
        }
    }
    None
}

/// `isOkBox` — `box.minY() > 10`.
fn is_ok_box(bb: &BoundingBox) -> bool {
    bb.min_y > LOWEST_Y_POSITION
}

// ======================================================================
// createPiece-фабрики (`findAndCreatePieceFactory`)
// ======================================================================

#[allow(clippy::too_many_arguments)]
fn create_piece_kind(
    kind: PieceKind,
    pieces: &dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    orientation: Direction,
    gen_depth: i32,
) -> Option<Box<dyn StructurePiece>> {
    let bb = |x0: i32, y0: i32, z0: i32, x1: i32, y1: i32, z1: i32| {
        BoundingBox::orient_box_offsets(x, y, z, x0, y0, z0, x1, y1, z1, orientation)
    };
    let ok = |b: &BoundingBox| is_ok_box(b) && pieces.find_collision_piece(b).is_none();
    match kind {
        PieceKind::Straight => {
            let b = bb(-1, -1, 0, 5, 5, 7);
            ok(&b).then(|| Box::new(Straight::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::PrisonHall => {
            let b = bb(-1, -1, 0, 9, 5, 11);
            ok(&b).then(|| Box::new(PrisonHall::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::LeftTurn => {
            let b = bb(-1, -1, 0, 5, 5, 5);
            ok(&b).then(|| Box::new(LeftTurn::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::RightTurn => {
            let b = bb(-1, -1, 0, 5, 5, 5);
            ok(&b).then(|| Box::new(RightTurn::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::RoomCrossing => {
            let b = bb(-4, -1, 0, 11, 7, 11);
            ok(&b).then(|| Box::new(RoomCrossing::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::StraightStairsDown => {
            let b = bb(-1, -7, 0, 5, 11, 8);
            ok(&b).then(|| {
                Box::new(StraightStairsDown::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>
            })
        }
        PieceKind::StairsDown => {
            let b = bb(-1, -7, 0, 5, 11, 5);
            ok(&b).then(|| Box::new(StairsDown::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::FiveCrossing => {
            let b = bb(-4, -3, 0, 10, 9, 11);
            ok(&b).then(|| Box::new(FiveCrossing::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::ChestCorridor => {
            let b = bb(-1, -1, 0, 5, 5, 7);
            ok(&b).then(|| {
                Box::new(ChestCorridor::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>
            })
        }
        PieceKind::Library => {
            // Java: сначала tall (ySpan 11), при коллизии — short (6).
            let mut b = bb(-4, -1, 0, 14, 11, 15);
            if !(is_ok_box(&b) && pieces.find_collision_piece(&b).is_none()) {
                b = bb(-4, -1, 0, 14, 6, 15);
                if !(is_ok_box(&b) && pieces.find_collision_piece(&b).is_none()) {
                    return None;
                }
            }
            Some(Box::new(Library::new(gen_depth, b, orientation, random)) as Box<dyn StructurePiece>)
        }
        PieceKind::PortalRoom => {
            let b = bb(-4, -1, 0, 11, 8, 16);
            ok(&b).then(|| Box::new(PortalRoom::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
    }
}

// ======================================================================
// addChildren-dispatch
// ======================================================================

/// `addChildren`-dispatch по типу куска (Java: каждый класс
/// вызывает generateSmallDoorChild* в своём порядке).
#[allow(clippy::too_many_arguments)]
fn stronghold_add_children(
    ty: &str,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    gen_depth: i32,
    flags: ChildFlags,
    data: &Rc<RefCell<StartPieceData>>,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
) {
    use Direction::*;
    match ty {
        // StartPiece (isSource) → imposedPiece = FiveCrossing.
        "stronghold_start" => {
            data.borrow_mut().imposed_piece = Some(PieceKind::FiveCrossing);
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 1);
        }
        "stronghold_stairs_down" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 1);
        }
        "stronghold_straight_stairs_down" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 1);
        }
        "stronghold_chest_corridor" | "stronghold_prison_hall" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 1);
        }
        "stronghold_filler_corridor" | "stronghold_library" => {}
        "stronghold_portal_room" => {
            data.borrow_mut().portal_room_placed = true;
        }
        "stronghold_five_crossing" => {
            let (mut i, mut i1) = (3, 5);
            if orientation == Some(WEST) || orientation == Some(NORTH) {
                i = 8 - i;
                i1 = 8 - i1;
            }
            child_forward(bb, orientation, gen_depth, data, pieces, random, 5, 1);
            if flags.f[0] {
                child_left(bb, orientation, gen_depth, data, pieces, random, i, 1);
            }
            if flags.f[1] {
                child_left(bb, orientation, gen_depth, data, pieces, random, i1, 7);
            }
            if flags.f[2] {
                child_right(bb, orientation, gen_depth, data, pieces, random, i, 1);
            }
            if flags.f[3] {
                child_right(bb, orientation, gen_depth, data, pieces, random, i1, 7);
            }
        }
        "stronghold_left_turn" => {
            if orientation != Some(NORTH) && orientation != Some(EAST) {
                child_right(bb, orientation, gen_depth, data, pieces, random, 1, 1);
            } else {
                child_left(bb, orientation, gen_depth, data, pieces, random, 1, 1);
            }
        }
        "stronghold_right_turn" => {
            if orientation != Some(NORTH) && orientation != Some(EAST) {
                child_left(bb, orientation, gen_depth, data, pieces, random, 1, 1);
            } else {
                child_right(bb, orientation, gen_depth, data, pieces, random, 1, 1);
            }
        }
        "stronghold_room_crossing" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 4, 1);
            child_left(bb, orientation, gen_depth, data, pieces, random, 1, 4);
            child_right(bb, orientation, gen_depth, data, pieces, random, 1, 4);
        }
        "stronghold_straight" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 1);
            if flags.f[0] {
                child_left(bb, orientation, gen_depth, data, pieces, random, 1, 2);
            }
            if flags.f[1] {
                child_right(bb, orientation, gen_depth, data, pieces, random, 1, 2);
            }
        }
        _ => {}
    }
}

/// Флаги куска для `PendingChild` (Java читает поля экземпляра
/// в `addChildren`).
fn child_flags_of(piece: &dyn StructurePiece) -> ChildFlags {
    let Some(a) = piece.as_any() else { return ChildFlags::default() };
    if let Some(p) = a.downcast_ref::<FiveCrossing>() {
        return ChildFlags {
            f: [p.left_low, p.left_high, p.right_low, p.right_high],
        };
    }
    if let Some(p) = a.downcast_ref::<Straight>() {
        return ChildFlags {
            f: [p.left_child, p.right_child, false, false],
        };
    }
    ChildFlags::default()
}

// ======================================================================
// Блок-состояния + SmoothStoneSelector + макросы
// ======================================================================

fn stone_bricks() -> BlockState {
    block_state("stone_bricks")
}
fn cave_air() -> BlockState {
    block_state("cave_air")
}
fn smooth_stone_slab() -> BlockState {
    block_state("smooth_stone_slab")
}
fn double_smooth_stone_slab() -> BlockState {
    with_string_property(smooth_stone_slab(), "type", "double")
}
fn stone_brick_slab() -> BlockState {
    block_state("stone_brick_slab")
}
fn oak_planks() -> BlockState {
    block_state("oak_planks")
}
fn bookshelf() -> BlockState {
    block_state("bookshelf")
}
fn cobweb() -> BlockState {
    block_state("cobweb")
}
fn water() -> BlockState {
    block_state("water")
}
fn cobblestone() -> BlockState {
    block_state("cobblestone")
}
fn lava() -> BlockState {
    block_state("lava")
}
fn torch() -> BlockState {
    block_state("torch")
}
fn wall_torch(facing: &str) -> BlockState {
    with_string_property(block_state("wall_torch"), "facing", facing)
}
fn plain_wall_torch() -> BlockState {
    block_state("wall_torch")
}
fn ladder(facing: &str) -> BlockState {
    with_string_property(block_state("ladder"), "facing", facing)
}
fn oak_fence(n: bool, s: bool, e: bool, w: bool) -> BlockState {
    let st = block_state("oak_fence");
    let st = with_bool_property(st, "north", n);
    let st = with_bool_property(st, "south", s);
    let st = with_bool_property(st, "east", e);
    with_bool_property(st, "west", w)
}
fn iron_bars(n: bool, s: bool, e: bool, w: bool) -> BlockState {
    let st = block_state("iron_bars");
    let st = with_bool_property(st, "north", n);
    let st = with_bool_property(st, "south", s);
    let st = with_bool_property(st, "east", e);
    with_bool_property(st, "west", w)
}
fn iron_door(upper: bool) -> BlockState {
    let st = block_state("iron_door");
    if upper {
        with_string_property(st, "half", "upper")
    } else {
        st
    }
}
fn oak_door(upper: bool) -> BlockState {
    let st = block_state("oak_door");
    if upper {
        with_string_property(st, "half", "upper")
    } else {
        st
    }
}
fn stone_button(facing: &str) -> BlockState {
    with_string_property(block_state("stone_button"), "facing", facing)
}
fn end_portal_frame(facing: &str, has_eye: bool) -> BlockState {
    let st = with_string_property(block_state("end_portal_frame"), "facing", facing);
    with_bool_property(st, "has_eye", has_eye)
}
fn end_portal() -> BlockState {
    block_state("end_portal")
}
fn stone_brick_stairs(facing: &str) -> BlockState {
    with_string_property(block_state("stone_brick_stairs"), "facing", facing)
}
fn cobblestone_stairs(facing: &str) -> BlockState {
    with_string_property(block_state("cobblestone_stairs"), "facing", facing)
}

/// `SmoothStoneSelector` — стены: 20% cracked, 50% mossy,
/// 55% infested, иначе stone bricks; не-стены — cave_air.
struct SmoothStoneSelector {
    next_state: BlockState,
}

impl SmoothStoneSelector {
    fn new() -> Self {
        Self {
            next_state: BlockState::air(),
        }
    }
}

impl BlockSelector for SmoothStoneSelector {
    fn next(&mut self, random: &mut dyn RandomSource, _x: i32, _y: i32, _z: i32, wall: bool) {
        if wall {
            let f = random.next_float();
            self.next_state = if f < 0.2 {
                block_state("cracked_stone_bricks")
            } else if f < 0.5 {
                block_state("mossy_stone_bricks")
            } else if f < 0.55 {
                block_state("infested_stone_bricks")
            } else {
                stone_bricks()
            };
        } else {
            self.next_state = cave_air();
        }
    }
    fn get_next(&self) -> &BlockState {
        &self.next_state
    }
}

/// `generateBox` selector-вариант с `SmoothStoneSelector`.
macro_rules! gbs {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $always:expr, $r:expr, $x1:expr, $y1:expr, $z1:expr, $x2:expr, $y2:expr, $z2:expr) => {{
        let mut sel = SmoothStoneSelector::new();
        generate_box_selector(
            $l, $t.bb, $t.orientation, $mir, $rot, $b,
            $x1, $y1, $z1, $x2, $y2, $z2, $always, $r, &mut sel,
        );
    }};
}

/// `generateBox` (boundary/inside/existingOnly).
macro_rules! gb {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $x1:expr, $y1:expr, $z1:expr, $x2:expr, $y2:expr, $z2:expr, $s1:expr, $s2:expr, $e:expr) => {
        generate_box(
            $l, $t.bb, $t.orientation, $mir, $rot, $b,
            $x1, $y1, $z1, $x2, $y2, $z2, &$s1, &$s2, $e,
        )
    };
}

/// `placeBlock(level, state, x, y, z, box)`.
macro_rules! pb {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $s:expr, $x:expr, $y:expr, $z:expr) => {
        place_block($l, $t.bb, $t.orientation, $mir, $rot, $s, $x, $y, $z, $b)
    };
}

/// `generateSmallDoor`.
macro_rules! sd {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $door:expr, $x:expr, $y:expr, $z:expr) => {
        generate_small_door($l, $t.bb, $t.orientation, $mir, $rot, $b, $door, $x, $y, $z)
    };
}

/// `maybeGenerateBlock`.
macro_rules! mgb {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $r:expr, $c:expr, $x:expr, $y:expr, $z:expr, $s:expr) => {
        maybe_generate_block($l, $t.bb, $t.orientation, $mir, $rot, $b, $r, $c, $x, $y, $z, &$s)
    };
}

/// `generateMaybeBox`.
#[allow(clippy::too_many_arguments)]
macro_rules! gmb {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $r:expr, $c:expr, $x1:expr, $y1:expr, $z1:expr, $x2:expr, $y2:expr, $z2:expr, $s1:expr, $s2:expr, $rna:expr, $rs:expr) => {
        generate_maybe_box(
            $l, $t.bb, $t.orientation, $mir, $rot, $b, $r, $c,
            $x1, $y1, $z1, $x2, $y2, $z2, &$s1, &$s2, $rna, $rs,
        )
    };
}

/// `generateSmallDoor` — 4 типа дверей 3x3.
#[allow(clippy::too_many_arguments)]
fn generate_small_door(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    door_type: SmallDoorType,
    x: i32,
    y: i32,
    z: i32,
) {
    let t = PieceTransform { bb, orientation };
    match door_type {
        SmallDoorType::Opening => {
            generate_box(
                level, bb, orientation, mirror, rotation, box_,
                x, y, z, x + 2, y + 2, z, &cave_air(), &cave_air(), false,
            );
        }
        SmallDoorType::WoodDoor => {
            let bricks = stone_bricks();
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x, y, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x, y + 1, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x, y + 2, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 1, y + 2, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 2, y + 2, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 2, y + 1, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 2, y, z);
            pb!(level, t, box_, mirror, rotation, oak_door(false), x + 1, y, z);
            pb!(level, t, box_, mirror, rotation, oak_door(true), x + 1, y + 1, z);
        }
        SmallDoorType::Grates => {
            pb!(level, t, box_, mirror, rotation, cave_air(), x + 1, y, z);
            pb!(level, t, box_, mirror, rotation, cave_air(), x + 1, y + 1, z);
            pb!(level, t, box_, mirror, rotation, iron_bars(false, false, false, true), x, y, z);
            pb!(level, t, box_, mirror, rotation, iron_bars(false, false, false, true), x, y + 1, z);
            pb!(level, t, box_, mirror, rotation, iron_bars(false, false, true, true), x, y + 2, z);
            pb!(level, t, box_, mirror, rotation, iron_bars(false, false, true, true), x + 1, y + 2, z);
            pb!(level, t, box_, mirror, rotation, iron_bars(false, false, true, true), x + 2, y + 2, z);
            pb!(level, t, box_, mirror, rotation, iron_bars(false, false, true, false), x + 2, y + 1, z);
            pb!(level, t, box_, mirror, rotation, iron_bars(false, false, true, false), x + 2, y, z);
        }
        SmallDoorType::IronDoor => {
            let bricks = stone_bricks();
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x, y, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x, y + 1, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x, y + 2, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 1, y + 2, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 2, y + 2, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 2, y + 1, z);
            pb!(level, t, box_, mirror, rotation, bricks.clone(), x + 2, y, z);
            pb!(level, t, box_, mirror, rotation, iron_door(false), x + 1, y, z);
            pb!(level, t, box_, mirror, rotation, iron_door(true), x + 1, y + 1, z);
            pb!(level, t, box_, mirror, rotation, stone_button("north"), x + 2, y + 1, z + 1);
            pb!(level, t, box_, mirror, rotation, stone_button("south"), x + 2, y + 1, z - 1);
        }
    }
}

// ======================================================================
// Куски
// ======================================================================

/// Общий `StructurePiece`-trait impl для кусков спонсора.
macro_rules! impl_strong_piece {
    ($name:ident, $ty:expr, $post:ident) => {
        impl StructurePiece for $name {
            fn bounding_box(&self) -> BoundingBox {
                self.bounding_box.clone()
            }
            fn bounding_box_mut(&mut self) -> Option<&mut BoundingBox> {
                Some(&mut self.bounding_box)
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
                Identifier::minecraft($ty)
            }
            fn add_additional_save_data(&self, tag: &mut CompoundTag) {
                tag.put(
                    "EntryDoor",
                    Tag::String(self.entry_door.name().to_string()),
                );
                StrongExtra::save_extra(self, tag);
            }
            fn load_additional_save_data(&mut self, tag: &CompoundTag) {
                self.entry_door =
                    SmallDoorType::from_name(tag.get_string("EntryDoor").unwrap_or("opening"));
                StrongExtra::load_extra(self, tag);
            }
            fn add_children(
                &mut self,
                piece: &mut dyn StructurePiece,
                pieces: &mut dyn StructurePieceAccessor,
                random: &mut dyn RandomSource,
            ) {
                if let Some(data) = piece
                    .as_any()
                    .and_then(|a| a.downcast_ref::<StartPiece>())
                    .map(|s| s.data.clone())
                {
                    stronghold_add_children(
                        $ty,
                        &self.bounding_box,
                        self.orientation,
                        self.gen_depth,
                        StrongExtra::child_flags(self),
                        &data,
                        pieces,
                        random,
                    );
                }
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
                self.$post(level, box_, random);
            }
            fn as_any(&self) -> Option<&dyn std::any::Any> {
                Some(self)
            }
        }
    };
}

/// Дополнительные NBT-поля кусков (`Chest`/`Steps`/`Tall`/`Mob`/
/// `Type`/`Source`/`Left`/`Right`/FiveCrossing-флаги) — default
/// как в Java `addAdditionalSaveData`.
trait StrongExtra {
    fn child_flags(&self) -> ChildFlags {
        ChildFlags::default()
    }
    fn save_extra(&self, _tag: &mut CompoundTag) {}
    fn load_extra(&mut self, _tag: &CompoundTag) {}
}

/// Общий конструктор: `setOrientation` + `entryDoor =
/// randomSmallDoor(random)` (Java-порядок: orientation, затем
/// дверь; RNG-порядок сохраняем вызовом после set_orientation).
macro_rules! strong_new {
    ($name:ident { $($field:ident : $val:expr),* $(,)? }) => {
        impl $name {
            fn new(
                gen_depth: i32,
                bounding_box: BoundingBox,
                orientation: Direction,
                random: &mut dyn RandomSource,
            ) -> Self {
                let mut s = Self {
                    bounding_box,
                    gen_depth,
                    orientation: None,
                    mirror: Mirror::NONE,
                    rotation: Rotation::NONE,
                    entry_door: SmallDoorType::Opening,
                    $($field: $val,)*
                };
                s.set_orientation(Some(orientation));
                s.entry_door = random_small_door(random);
                s
            }
        }
    };
}

// ---------- ChestCorridor ----------

pub struct ChestCorridor {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    has_placed_chest: bool,
}

strong_new!(ChestCorridor { has_placed_chest: false });

impl ChestCorridor {
    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 4, 4, 6);
        sd!(level, t, box_, mir, rot, self.entry_door, 1, 1, 0);
        sd!(level, t, box_, mir, rot, SmallDoorType::Opening, 1, 1, 6);
        gb!(level, t, box_, mir, rot, 3, 1, 2, 3, 1, 4, stone_bricks(), stone_bricks(), false);
        let slab = stone_brick_slab();
        pb!(level, t, box_, mir, rot, slab.clone(), 3, 1, 1);
        pb!(level, t, box_, mir, rot, slab.clone(), 3, 1, 5);
        pb!(level, t, box_, mir, rot, slab.clone(), 3, 2, 2);
        pb!(level, t, box_, mir, rot, slab.clone(), 3, 2, 4);
        for i in 2..=4 {
            pb!(level, t, box_, mir, rot, slab.clone(), 2, 1, i);
        }
        if !self.has_placed_chest && box_.is_inside_pos(t.world_pos(3, 2, 3)) {
            self.has_placed_chest = true;
            create_chest(
                level,
                t.bb,
                t.orientation,
                mir,
                rot,
                box_,
                random,
                3,
                2,
                3,
                &Identifier::minecraft(STRONGHOLD_CORRIDOR_LOOT),
            );
        }
    }
}

impl StrongExtra for ChestCorridor {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Chest", Tag::Byte(if self.has_placed_chest { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.has_placed_chest = tag.get_bool("Chest").unwrap_or(false);
    }
}

impl_strong_piece!(ChestCorridor, "stronghold_chest_corridor", post);

// ---------- FillerCorridor ----------

pub struct FillerCorridor {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    steps: i32,
}

impl FillerCorridor {
    fn new(gen_depth: i32, bounding_box: BoundingBox, orientation: Direction) -> Self {
        let steps = if orientation != Direction::NORTH && orientation != Direction::SOUTH {
            bounding_box.get_x_span()
        } else {
            bounding_box.get_z_span()
        };
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            steps,
        };
        s.set_orientation(Some(orientation));
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let bricks = stone_bricks();
        let air = cave_air();
        for i in 0..self.steps {
            for x in 0..=4 {
                pb!(level, t, box_, mir, rot, bricks.clone(), x, 0, i);
            }
            for y in 1..=3 {
                pb!(level, t, box_, mir, rot, bricks.clone(), 0, y, i);
                pb!(level, t, box_, mir, rot, air.clone(), 1, y, i);
                pb!(level, t, box_, mir, rot, air.clone(), 2, y, i);
                pb!(level, t, box_, mir, rot, air.clone(), 3, y, i);
                pb!(level, t, box_, mir, rot, bricks.clone(), 4, y, i);
            }
            for x in 0..=4 {
                pb!(level, t, box_, mir, rot, bricks.clone(), x, 4, i);
            }
        }
    }
}

impl StrongExtra for FillerCorridor {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Steps", Tag::Int(self.steps));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.steps = tag.get_int("Steps").unwrap_or(0);
    }
}

impl_strong_piece!(FillerCorridor, "stronghold_filler_corridor", post);

// ---------- FiveCrossing ----------

pub struct FiveCrossing {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    left_low: bool,
    left_high: bool,
    right_low: bool,
    right_high: bool,
}

impl FiveCrossing {
    fn new(
        gen_depth: i32,
        bounding_box: BoundingBox,
        orientation: Direction,
        random: &mut dyn RandomSource,
    ) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            left_low: false,
            left_high: false,
            right_low: false,
            right_high: false,
        };
        s.set_orientation(Some(orientation));
        s.entry_door = random_small_door(random);
        s.left_low = random.next_boolean();
        s.left_high = random.next_boolean();
        s.right_low = random.next_boolean();
        s.right_high = random.next_int_bound(3) > 0;
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let air = cave_air();
        let slab = smooth_stone_slab();
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 9, 8, 10);
        sd!(level, t, box_, mir, rot, self.entry_door, 4, 3, 0);
        if self.left_low {
            gb!(level, t, box_, mir, rot, 0, 3, 1, 0, 5, 3, air.clone(), air.clone(), false);
        }
        if self.right_low {
            gb!(level, t, box_, mir, rot, 9, 3, 1, 9, 5, 3, air.clone(), air.clone(), false);
        }
        if self.left_high {
            gb!(level, t, box_, mir, rot, 0, 5, 7, 0, 7, 9, air.clone(), air.clone(), false);
        }
        if self.right_high {
            gb!(level, t, box_, mir, rot, 9, 5, 7, 9, 7, 9, air.clone(), air.clone(), false);
        }
        gb!(level, t, box_, mir, rot, 5, 1, 10, 7, 3, 10, air.clone(), air.clone(), false);
        gbs!(level, t, box_, mir, rot, false, random, 1, 2, 1, 8, 2, 6);
        gbs!(level, t, box_, mir, rot, false, random, 4, 1, 5, 4, 4, 9);
        gbs!(level, t, box_, mir, rot, false, random, 8, 1, 5, 8, 4, 9);
        gbs!(level, t, box_, mir, rot, false, random, 1, 4, 7, 3, 4, 9);
        gbs!(level, t, box_, mir, rot, false, random, 1, 3, 5, 3, 3, 6);
        gb!(level, t, box_, mir, rot, 1, 3, 4, 3, 3, 4, slab.clone(), slab.clone(), false);
        gb!(level, t, box_, mir, rot, 1, 4, 6, 3, 4, 6, slab.clone(), slab.clone(), false);
        gbs!(level, t, box_, mir, rot, false, random, 5, 1, 7, 7, 1, 8);
        gb!(level, t, box_, mir, rot, 5, 1, 9, 7, 1, 9, slab.clone(), slab.clone(), false);
        gb!(level, t, box_, mir, rot, 5, 2, 7, 7, 2, 7, slab.clone(), slab.clone(), false);
        gb!(level, t, box_, mir, rot, 4, 5, 7, 4, 5, 9, slab.clone(), slab.clone(), false);
        gb!(level, t, box_, mir, rot, 8, 5, 7, 8, 5, 9, slab.clone(), slab.clone(), false);
        gb!(level, t, box_, mir, rot, 5, 5, 7, 7, 5, 9, double_smooth_stone_slab(), double_smooth_stone_slab(), false);
        pb!(level, t, box_, mir, rot, wall_torch("south"), 6, 5, 6);
    }
}

impl StrongExtra for FiveCrossing {
    fn child_flags(&self) -> ChildFlags {
        ChildFlags {
            f: [self.left_low, self.left_high, self.right_low, self.right_high],
        }
    }
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("leftLow", Tag::Byte(if self.left_low { 1 } else { 0 }));
        tag.put("leftHigh", Tag::Byte(if self.left_high { 1 } else { 0 }));
        tag.put("rightLow", Tag::Byte(if self.right_low { 1 } else { 0 }));
        tag.put("rightHigh", Tag::Byte(if self.right_high { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.left_low = tag.get_bool("leftLow").unwrap_or(false);
        self.left_high = tag.get_bool("leftHigh").unwrap_or(false);
        self.right_low = tag.get_bool("rightLow").unwrap_or(false);
        self.right_high = tag.get_bool("rightHigh").unwrap_or(false);
    }
}

impl_strong_piece!(FiveCrossing, "stronghold_five_crossing", post);

// ---------- LeftTurn / RightTurn (Turn) ----------

pub struct LeftTurn {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
}

strong_new!(LeftTurn {});

impl LeftTurn {
    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let air = cave_air();
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 4, 4, 4);
        sd!(level, t, box_, mir, rot, self.entry_door, 1, 1, 0);
        if self.orientation != Some(Direction::NORTH) && self.orientation != Some(Direction::EAST) {
            gb!(level, t, box_, mir, rot, 4, 1, 1, 4, 3, 3, air.clone(), air.clone(), false);
        } else {
            gb!(level, t, box_, mir, rot, 0, 1, 1, 0, 3, 3, air.clone(), air.clone(), false);
        }
    }
}

impl StrongExtra for LeftTurn {}

impl_strong_piece!(LeftTurn, "stronghold_left_turn", post);

pub struct RightTurn {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
}

strong_new!(RightTurn {});

impl RightTurn {
    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let air = cave_air();
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 4, 4, 4);
        sd!(level, t, box_, mir, rot, self.entry_door, 1, 1, 0);
        if self.orientation != Some(Direction::NORTH) && self.orientation != Some(Direction::EAST) {
            gb!(level, t, box_, mir, rot, 0, 1, 1, 0, 3, 3, air.clone(), air.clone(), false);
        } else {
            gb!(level, t, box_, mir, rot, 4, 1, 1, 4, 3, 3, air.clone(), air.clone(), false);
        }
    }
}

impl StrongExtra for RightTurn {}

impl_strong_piece!(RightTurn, "stronghold_right_turn", post);

// ---------- Library ----------

pub struct Library {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    is_tall: bool,
}

impl Library {
    fn new(
        gen_depth: i32,
        bounding_box: BoundingBox,
        orientation: Direction,
        random: &mut dyn RandomSource,
    ) -> Self {
        let is_tall = bounding_box.get_y_span() > 6;
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            is_tall,
        };
        s.set_orientation(Some(orientation));
        s.entry_door = random_small_door(random);
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let i = if self.is_tall { 11 } else { 6 };
        let planks = oak_planks();
        let shelf = bookshelf();
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 13, i - 1, 14);
        sd!(level, t, box_, mir, rot, self.entry_door, 4, 1, 0);
        gmb!(level, t, box_, mir, rot, random, 0.07, 2, 1, 1, 11, 4, 13, cobweb(), cobweb(), false, false);
        for z in 1..=13 {
            if (z - 1) % 4 == 0 {
                gb!(level, t, box_, mir, rot, 1, 1, z, 1, 4, z, planks.clone(), planks.clone(), false);
                gb!(level, t, box_, mir, rot, 12, 1, z, 12, 4, z, planks.clone(), planks.clone(), false);
                pb!(level, t, box_, mir, rot, wall_torch("east"), 2, 3, z);
                pb!(level, t, box_, mir, rot, wall_torch("west"), 11, 3, z);
                if self.is_tall {
                    gb!(level, t, box_, mir, rot, 1, 6, z, 1, 9, z, planks.clone(), planks.clone(), false);
                    gb!(level, t, box_, mir, rot, 12, 6, z, 12, 9, z, planks.clone(), planks.clone(), false);
                }
            } else {
                gb!(level, t, box_, mir, rot, 1, 1, z, 1, 4, z, shelf.clone(), shelf.clone(), false);
                gb!(level, t, box_, mir, rot, 12, 1, z, 12, 4, z, shelf.clone(), shelf.clone(), false);
                if self.is_tall {
                    gb!(level, t, box_, mir, rot, 1, 6, z, 1, 9, z, shelf.clone(), shelf.clone(), false);
                    gb!(level, t, box_, mir, rot, 12, 6, z, 12, 9, z, shelf.clone(), shelf.clone(), false);
                }
            }
        }
        for z in (3..12).step_by(2) {
            gb!(level, t, box_, mir, rot, 3, 1, z, 4, 3, z, shelf.clone(), shelf.clone(), false);
            gb!(level, t, box_, mir, rot, 6, 1, z, 7, 3, z, shelf.clone(), shelf.clone(), false);
            gb!(level, t, box_, mir, rot, 9, 1, z, 10, 3, z, shelf.clone(), shelf.clone(), false);
        }
        if self.is_tall {
            gb!(level, t, box_, mir, rot, 1, 5, 1, 3, 5, 13, planks.clone(), planks.clone(), false);
            gb!(level, t, box_, mir, rot, 10, 5, 1, 12, 5, 13, planks.clone(), planks.clone(), false);
            gb!(level, t, box_, mir, rot, 4, 5, 1, 9, 5, 2, planks.clone(), planks.clone(), false);
            gb!(level, t, box_, mir, rot, 4, 5, 12, 9, 5, 13, planks.clone(), planks.clone(), false);
            pb!(level, t, box_, mir, rot, planks.clone(), 9, 5, 11);
            pb!(level, t, box_, mir, rot, planks.clone(), 8, 5, 11);
            pb!(level, t, box_, mir, rot, planks.clone(), 9, 5, 10);
            let bs_we = oak_fence(false, false, true, true);
            let bs_ns = oak_fence(true, true, false, false);
            gb!(level, t, box_, mir, rot, 3, 6, 3, 3, 6, 11, bs_ns.clone(), bs_ns.clone(), false);
            gb!(level, t, box_, mir, rot, 10, 6, 3, 10, 6, 9, bs_ns.clone(), bs_ns.clone(), false);
            gb!(level, t, box_, mir, rot, 4, 6, 2, 9, 6, 2, bs_we.clone(), bs_we.clone(), false);
            gb!(level, t, box_, mir, rot, 4, 6, 12, 7, 6, 12, bs_we.clone(), bs_we.clone(), false);
            pb!(level, t, box_, mir, rot, oak_fence(true, false, true, false), 3, 6, 2);
            pb!(level, t, box_, mir, rot, oak_fence(false, true, true, false), 3, 6, 12);
            pb!(level, t, box_, mir, rot, oak_fence(true, false, false, true), 10, 6, 2);
            for i4 in 0..=2 {
                pb!(level, t, box_, mir, rot, oak_fence(false, true, false, true), 8 + i4, 6, 12 - i4);
                if i4 != 2 {
                    pb!(level, t, box_, mir, rot, oak_fence(true, false, true, false), 8 + i4, 6, 11 - i4);
                }
            }
            let lad = ladder("south");
            for y in 1..=7 {
                pb!(level, t, box_, mir, rot, lad.clone(), 10, y, 13);
            }
            let bs_e = oak_fence(false, false, true, false);
            let bs_w = oak_fence(false, false, false, true);
            pb!(level, t, box_, mir, rot, bs_e.clone(), 6, 9, 7);
            pb!(level, t, box_, mir, rot, bs_w.clone(), 7, 9, 7);
            pb!(level, t, box_, mir, rot, bs_e.clone(), 6, 8, 7);
            pb!(level, t, box_, mir, rot, bs_w.clone(), 7, 8, 7);
            let bs_all = oak_fence(true, true, true, true);
            pb!(level, t, box_, mir, rot, bs_all.clone(), 6, 7, 7);
            pb!(level, t, box_, mir, rot, bs_all.clone(), 7, 7, 7);
            pb!(level, t, box_, mir, rot, bs_e.clone(), 5, 7, 7);
            pb!(level, t, box_, mir, rot, bs_w.clone(), 8, 7, 7);
            pb!(level, t, box_, mir, rot, oak_fence(true, false, true, false), 6, 7, 6);
            pb!(level, t, box_, mir, rot, oak_fence(false, true, true, false), 6, 7, 8);
            pb!(level, t, box_, mir, rot, oak_fence(true, false, false, true), 7, 7, 6);
            pb!(level, t, box_, mir, rot, oak_fence(false, true, false, true), 7, 7, 8);
            let tor = torch();
            pb!(level, t, box_, mir, rot, tor.clone(), 5, 8, 7);
            pb!(level, t, box_, mir, rot, tor.clone(), 8, 8, 7);
            pb!(level, t, box_, mir, rot, tor.clone(), 6, 8, 6);
            pb!(level, t, box_, mir, rot, tor.clone(), 6, 8, 8);
            pb!(level, t, box_, mir, rot, tor.clone(), 7, 8, 6);
            pb!(level, t, box_, mir, rot, tor.clone(), 7, 8, 8);
        }
        create_chest(
            level,
            t.bb,
            t.orientation,
            mir,
            rot,
            box_,
            random,
            3,
            3,
            5,
            &Identifier::minecraft(STRONGHOLD_LIBRARY_LOOT),
        );
        if self.is_tall {
            pb!(level, t, box_, mir, rot, cave_air(), 12, 9, 1);
            create_chest(
                level,
                t.bb,
                t.orientation,
                mir,
                rot,
                box_,
                random,
                12,
                8,
                1,
                &Identifier::minecraft(STRONGHOLD_LIBRARY_LOOT),
            );
        }
    }
}

impl StrongExtra for Library {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Tall", Tag::Byte(if self.is_tall { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.is_tall = tag.get_bool("Tall").unwrap_or(false);
    }
}

impl_strong_piece!(Library, "stronghold_library", post);

// ---------- PortalRoom ----------

pub struct PortalRoom {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    has_placed_spawner: bool,
}

impl PortalRoom {
    fn new(gen_depth: i32, bounding_box: BoundingBox, orientation: Direction) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            has_placed_spawner: false,
        };
        s.set_orientation(Some(orientation));
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let lv = lava();
        let bars_ns = iron_bars(true, true, false, false);
        let bars_we = iron_bars(false, false, true, true);
        gbs!(level, t, box_, mir, rot, false, random, 0, 0, 0, 10, 7, 15);
        sd!(level, t, box_, mir, rot, SmallDoorType::Grates, 4, 1, 0);
        gbs!(level, t, box_, mir, rot, false, random, 1, 6, 1, 1, 6, 14);
        gbs!(level, t, box_, mir, rot, false, random, 9, 6, 1, 9, 6, 14);
        gbs!(level, t, box_, mir, rot, false, random, 2, 6, 1, 8, 6, 2);
        gbs!(level, t, box_, mir, rot, false, random, 2, 6, 14, 8, 6, 14);
        gbs!(level, t, box_, mir, rot, false, random, 1, 1, 1, 2, 1, 4);
        gbs!(level, t, box_, mir, rot, false, random, 8, 1, 1, 9, 1, 4);
        gb!(level, t, box_, mir, rot, 1, 1, 1, 1, 1, 3, lv.clone(), lv.clone(), false);
        gb!(level, t, box_, mir, rot, 9, 1, 1, 9, 1, 3, lv.clone(), lv.clone(), false);
        gbs!(level, t, box_, mir, rot, false, random, 3, 1, 8, 7, 1, 12);
        gb!(level, t, box_, mir, rot, 4, 1, 9, 6, 1, 11, lv.clone(), lv.clone(), false);
        for i1 in (3..14).step_by(2) {
            gb!(level, t, box_, mir, rot, 0, 3, i1, 0, 4, i1, bars_ns.clone(), bars_ns.clone(), false);
            gb!(level, t, box_, mir, rot, 10, 3, i1, 10, 4, i1, bars_ns.clone(), bars_ns.clone(), false);
        }
        for i1 in (2..9).step_by(2) {
            gb!(level, t, box_, mir, rot, i1, 3, 15, i1, 4, 15, bars_we.clone(), bars_we.clone(), false);
        }
        let stairs = stone_brick_stairs("north");
        gbs!(level, t, box_, mir, rot, false, random, 4, 1, 5, 6, 1, 7);
        gbs!(level, t, box_, mir, rot, false, random, 4, 2, 6, 6, 2, 7);
        gbs!(level, t, box_, mir, rot, false, random, 4, 3, 7, 6, 3, 7);
        for i2 in 4..=6 {
            pb!(level, t, box_, mir, rot, stairs.clone(), i2, 1, 4);
            pb!(level, t, box_, mir, rot, stairs.clone(), i2, 2, 5);
            pb!(level, t, box_, mir, rot, stairs.clone(), i2, 3, 6);
        }
        let frame_n = end_portal_frame("north", false);
        let frame_s = end_portal_frame("south", false);
        let frame_e = end_portal_frame("east", false);
        let frame_w = end_portal_frame("west", false);
        let mut flags = [false; 12];
        let mut all_eyes = true;
        for i3 in 0..flags.len() {
            flags[i3] = random.next_float() > 0.9;
            all_eyes &= flags[i3];
        }
        pb!(level, t, box_, mir, rot, with_bool_property(frame_n.clone(), "has_eye", flags[0]), 4, 3, 8);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_n.clone(), "has_eye", flags[1]), 5, 3, 8);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_n.clone(), "has_eye", flags[2]), 6, 3, 8);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_s.clone(), "has_eye", flags[3]), 4, 3, 12);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_s.clone(), "has_eye", flags[4]), 5, 3, 12);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_s.clone(), "has_eye", flags[5]), 6, 3, 12);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_e.clone(), "has_eye", flags[6]), 3, 3, 9);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_e.clone(), "has_eye", flags[7]), 3, 3, 10);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_e.clone(), "has_eye", flags[8]), 3, 3, 11);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_w.clone(), "has_eye", flags[9]), 7, 3, 9);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_w.clone(), "has_eye", flags[10]), 7, 3, 10);
        pb!(level, t, box_, mir, rot, with_bool_property(frame_w.clone(), "has_eye", flags[11]), 7, 3, 11);
        if all_eyes {
            let portal = end_portal();
            for x in 4..=6 {
                for z in 9..=11 {
                    pb!(level, t, box_, mir, rot, portal.clone(), x, 3, z);
                }
            }
        }
        if !self.has_placed_spawner {
            let world_pos = t.world_pos(5, 3, 6);
            if box_.is_inside_pos(world_pos) {
                self.has_placed_spawner = true;
                level.place_structure_spawner(world_pos, "silverfish");
            }
        }
    }
}

impl StrongExtra for PortalRoom {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Mob", Tag::Byte(if self.has_placed_spawner { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.has_placed_spawner = tag.get_bool("Mob").unwrap_or(false);
    }
}

impl_strong_piece!(PortalRoom, "stronghold_portal_room", post);

// ---------- PrisonHall ----------

pub struct PrisonHall {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
}

strong_new!(PrisonHall {});

impl PrisonHall {
    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let air = cave_air();
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 8, 4, 10);
        sd!(level, t, box_, mir, rot, self.entry_door, 1, 1, 0);
        gb!(level, t, box_, mir, rot, 1, 1, 10, 3, 3, 10, air.clone(), air.clone(), false);
        gbs!(level, t, box_, mir, rot, false, random, 4, 1, 1, 4, 3, 1);
        gbs!(level, t, box_, mir, rot, false, random, 4, 1, 3, 4, 3, 3);
        gbs!(level, t, box_, mir, rot, false, random, 4, 1, 7, 4, 3, 7);
        gbs!(level, t, box_, mir, rot, false, random, 4, 1, 9, 4, 3, 9);
        let bars_ns = iron_bars(true, true, false, false);
        let bars_ns_e = iron_bars(true, true, true, false);
        let bars_we = iron_bars(false, false, true, true);
        for i in 1..=3 {
            pb!(level, t, box_, mir, rot, bars_ns.clone(), 4, i, 4);
            pb!(level, t, box_, mir, rot, bars_ns_e.clone(), 4, i, 5);
            pb!(level, t, box_, mir, rot, bars_ns.clone(), 4, i, 6);
            pb!(level, t, box_, mir, rot, bars_we.clone(), 5, i, 5);
            pb!(level, t, box_, mir, rot, bars_we.clone(), 6, i, 5);
            pb!(level, t, box_, mir, rot, bars_we.clone(), 7, i, 5);
        }
        pb!(level, t, box_, mir, rot, bars_ns.clone(), 4, 3, 2);
        pb!(level, t, box_, mir, rot, bars_ns.clone(), 4, 3, 8);
        let door = with_string_property(iron_door(false), "facing", "west");
        let door_u = with_string_property(iron_door(true), "facing", "west");
        pb!(level, t, box_, mir, rot, door.clone(), 4, 1, 2);
        pb!(level, t, box_, mir, rot, door_u.clone(), 4, 2, 2);
        pb!(level, t, box_, mir, rot, door.clone(), 4, 1, 8);
        pb!(level, t, box_, mir, rot, door_u.clone(), 4, 2, 8);
    }
}

impl StrongExtra for PrisonHall {}

impl_strong_piece!(PrisonHall, "stronghold_prison_hall", post);

// ---------- RoomCrossing ----------

pub struct RoomCrossing {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    r#type: i32,
}

impl RoomCrossing {
    fn new(
        gen_depth: i32,
        bounding_box: BoundingBox,
        orientation: Direction,
        random: &mut dyn RandomSource,
    ) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            r#type: 0,
        };
        s.set_orientation(Some(orientation));
        s.entry_door = random_small_door(random);
        s.r#type = random.next_int_bound(5);
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let air = cave_air();
        let bricks = stone_bricks();
        let slab = smooth_stone_slab();
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 10, 6, 10);
        sd!(level, t, box_, mir, rot, self.entry_door, 4, 1, 0);
        gb!(level, t, box_, mir, rot, 4, 1, 10, 6, 3, 10, air.clone(), air.clone(), false);
        gb!(level, t, box_, mir, rot, 0, 1, 4, 0, 3, 6, air.clone(), air.clone(), false);
        gb!(level, t, box_, mir, rot, 10, 1, 4, 10, 3, 6, air.clone(), air.clone(), false);
        match self.r#type {
            0 => {
                pb!(level, t, box_, mir, rot, bricks.clone(), 5, 1, 5);
                pb!(level, t, box_, mir, rot, bricks.clone(), 5, 2, 5);
                pb!(level, t, box_, mir, rot, bricks.clone(), 5, 3, 5);
                pb!(level, t, box_, mir, rot, wall_torch("west"), 4, 3, 5);
                pb!(level, t, box_, mir, rot, wall_torch("east"), 6, 3, 5);
                pb!(level, t, box_, mir, rot, wall_torch("south"), 5, 3, 4);
                pb!(level, t, box_, mir, rot, wall_torch("north"), 5, 3, 6);
                pb!(level, t, box_, mir, rot, slab.clone(), 4, 1, 4);
                pb!(level, t, box_, mir, rot, slab.clone(), 4, 1, 5);
                pb!(level, t, box_, mir, rot, slab.clone(), 4, 1, 6);
                pb!(level, t, box_, mir, rot, slab.clone(), 6, 1, 4);
                pb!(level, t, box_, mir, rot, slab.clone(), 6, 1, 5);
                pb!(level, t, box_, mir, rot, slab.clone(), 6, 1, 6);
                pb!(level, t, box_, mir, rot, slab.clone(), 5, 1, 4);
                pb!(level, t, box_, mir, rot, slab.clone(), 5, 1, 6);
            }
            1 => {
                for i in 0..5 {
                    pb!(level, t, box_, mir, rot, bricks.clone(), 3, 1, 3 + i);
                    pb!(level, t, box_, mir, rot, bricks.clone(), 7, 1, 3 + i);
                    pb!(level, t, box_, mir, rot, bricks.clone(), 3 + i, 1, 3);
                    pb!(level, t, box_, mir, rot, bricks.clone(), 3 + i, 1, 7);
                }
                pb!(level, t, box_, mir, rot, bricks.clone(), 5, 1, 5);
                pb!(level, t, box_, mir, rot, bricks.clone(), 5, 2, 5);
                pb!(level, t, box_, mir, rot, bricks.clone(), 5, 3, 5);
                pb!(level, t, box_, mir, rot, water(), 5, 4, 5);
            }
            2 => {
                for i in 1..=9 {
                    pb!(level, t, box_, mir, rot, cobblestone(), 1, 3, i);
                    pb!(level, t, box_, mir, rot, cobblestone(), 9, 3, i);
                }
                for i in 1..=9 {
                    pb!(level, t, box_, mir, rot, cobblestone(), i, 3, 1);
                    pb!(level, t, box_, mir, rot, cobblestone(), i, 3, 9);
                }
                pb!(level, t, box_, mir, rot, cobblestone(), 5, 1, 4);
                pb!(level, t, box_, mir, rot, cobblestone(), 5, 1, 6);
                pb!(level, t, box_, mir, rot, cobblestone(), 5, 3, 4);
                pb!(level, t, box_, mir, rot, cobblestone(), 5, 3, 6);
                pb!(level, t, box_, mir, rot, cobblestone(), 4, 1, 5);
                pb!(level, t, box_, mir, rot, cobblestone(), 6, 1, 5);
                pb!(level, t, box_, mir, rot, cobblestone(), 4, 3, 5);
                pb!(level, t, box_, mir, rot, cobblestone(), 6, 3, 5);
                for i in 1..=3 {
                    pb!(level, t, box_, mir, rot, cobblestone(), 4, i, 4);
                    pb!(level, t, box_, mir, rot, cobblestone(), 6, i, 4);
                    pb!(level, t, box_, mir, rot, cobblestone(), 4, i, 6);
                    pb!(level, t, box_, mir, rot, cobblestone(), 6, i, 6);
                }
                pb!(level, t, box_, mir, rot, plain_wall_torch(), 5, 3, 5);
                let planks = oak_planks();
                for i in 2..=8 {
                    pb!(level, t, box_, mir, rot, planks.clone(), 2, 3, i);
                    pb!(level, t, box_, mir, rot, planks.clone(), 3, 3, i);
                    if i <= 3 || i >= 7 {
                        pb!(level, t, box_, mir, rot, planks.clone(), 4, 3, i);
                        pb!(level, t, box_, mir, rot, planks.clone(), 5, 3, i);
                        pb!(level, t, box_, mir, rot, planks.clone(), 6, 3, i);
                    }
                    pb!(level, t, box_, mir, rot, planks.clone(), 7, 3, i);
                    pb!(level, t, box_, mir, rot, planks.clone(), 8, 3, i);
                }
                let lad = ladder("west");
                pb!(level, t, box_, mir, rot, lad.clone(), 9, 1, 3);
                pb!(level, t, box_, mir, rot, lad.clone(), 9, 2, 3);
                pb!(level, t, box_, mir, rot, lad.clone(), 9, 3, 3);
                create_chest(
                    level,
                    t.bb,
                    t.orientation,
                    mir,
                    rot,
                    box_,
                    random,
                    3,
                    4,
                    8,
                    &Identifier::minecraft(STRONGHOLD_CROSSING_LOOT),
                );
            }
            _ => {}
        }
    }
}

impl StrongExtra for RoomCrossing {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Type", Tag::Int(self.r#type));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.r#type = tag.get_int("Type").unwrap_or(0);
    }
}

impl_strong_piece!(RoomCrossing, "stronghold_room_crossing", post);

// ---------- StairsDown / StartPiece ----------

/// Общий `postProcess` StairsDown (StartPiece наследует).
#[allow(clippy::too_many_arguments)]
fn stairs_down_post(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    box_: &BoundingBox,
    entry_door: SmallDoorType,
    random: &mut dyn RandomSource,
) {
    let t = PieceTransform { bb, orientation };
    let bricks = stone_bricks();
    let slab = smooth_stone_slab();
    gbs!(level, t, box_, mirror, rotation, true, random, 0, 0, 0, 4, 10, 4);
    sd!(level, t, box_, mirror, rotation, entry_door, 1, 7, 0);
    sd!(level, t, box_, mirror, rotation, SmallDoorType::Opening, 1, 1, 4);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 2, 6, 1);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 1, 5, 1);
    pb!(level, t, box_, mirror, rotation, slab.clone(), 1, 6, 1);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 1, 5, 2);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 1, 4, 3);
    pb!(level, t, box_, mirror, rotation, slab.clone(), 1, 5, 3);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 2, 4, 3);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 3, 3, 3);
    pb!(level, t, box_, mirror, rotation, slab.clone(), 3, 4, 3);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 3, 3, 2);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 3, 2, 1);
    pb!(level, t, box_, mirror, rotation, slab.clone(), 3, 3, 1);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 2, 2, 1);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 1, 1, 1);
    pb!(level, t, box_, mirror, rotation, slab.clone(), 1, 2, 1);
    pb!(level, t, box_, mirror, rotation, bricks.clone(), 1, 1, 2);
    pb!(level, t, box_, mirror, rotation, slab.clone(), 1, 1, 3);
}

pub struct StairsDown {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    is_source: bool,
}

impl StairsDown {
    fn new(
        gen_depth: i32,
        bounding_box: BoundingBox,
        orientation: Direction,
        random: &mut dyn RandomSource,
    ) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            is_source: false,
        };
        s.set_orientation(Some(orientation));
        s.entry_door = random_small_door(random);
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        stairs_down_post(
            level,
            &bb,
            self.orientation,
            self.mirror,
            self.rotation,
            box_,
            self.entry_door,
            random,
        );
    }
}

impl StrongExtra for StairsDown {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Source", Tag::Byte(if self.is_source { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.is_source = tag.get_bool("Source").unwrap_or(false);
    }
}

impl_strong_piece!(StairsDown, "stronghold_stairs_down", post);

/// `StartPiece` — source-StairsDown (`isSource=true`,
/// `entryDoor=OPENING`, genDepth=0) + start-данные.
pub struct StartPiece {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    data: Rc<RefCell<StartPieceData>>,
}

impl StartPiece {
    fn new(random: &mut dyn RandomSource, x: i32, z: i32) -> Self {
        let dir = get_random_horizontal_direction(random);
        let bb = make_bounding_box(x, MAGIC_START_Y, z, dir, 5, 11, 5);
        let mut s = Self {
            bounding_box: bb,
            gen_depth: 0,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            data: Rc::new(RefCell::new(StartPieceData::new())),
        };
        s.set_orientation(Some(dir));
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        stairs_down_post(
            level,
            &bb,
            self.orientation,
            self.mirror,
            self.rotation,
            box_,
            self.entry_door,
            random,
        );
    }
}

impl StrongExtra for StartPiece {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Source", Tag::Byte(1));
    }
}

impl_strong_piece!(StartPiece, "stronghold_start", post);

// ---------- Straight ----------

pub struct Straight {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
    left_child: bool,
    right_child: bool,
}

impl Straight {
    fn new(
        gen_depth: i32,
        bounding_box: BoundingBox,
        orientation: Direction,
        random: &mut dyn RandomSource,
    ) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            entry_door: SmallDoorType::Opening,
            left_child: false,
            right_child: false,
        };
        s.set_orientation(Some(orientation));
        s.entry_door = random_small_door(random);
        s.left_child = random.next_int_bound(2) == 0;
        s.right_child = random.next_int_bound(2) == 0;
        s
    }

    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let air = cave_air();
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 4, 4, 6);
        sd!(level, t, box_, mir, rot, self.entry_door, 1, 1, 0);
        sd!(level, t, box_, mir, rot, SmallDoorType::Opening, 1, 1, 6);
        let torch_e = wall_torch("east");
        let torch_w = wall_torch("west");
        mgb!(level, t, box_, mir, rot, random, 0.1, 1, 2, 1, torch_e);
        mgb!(level, t, box_, mir, rot, random, 0.1, 3, 2, 1, torch_w);
        mgb!(level, t, box_, mir, rot, random, 0.1, 1, 2, 5, torch_e);
        mgb!(level, t, box_, mir, rot, random, 0.1, 3, 2, 5, torch_w);
        if self.left_child {
            gb!(level, t, box_, mir, rot, 0, 1, 2, 0, 3, 4, air.clone(), air.clone(), false);
        }
        if self.right_child {
            gb!(level, t, box_, mir, rot, 4, 1, 2, 4, 3, 4, air.clone(), air.clone(), false);
        }
    }
}

impl StrongExtra for Straight {
    fn child_flags(&self) -> ChildFlags {
        ChildFlags {
            f: [self.left_child, self.right_child, false, false],
        }
    }
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Left", Tag::Byte(if self.left_child { 1 } else { 0 }));
        tag.put("Right", Tag::Byte(if self.right_child { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.left_child = tag.get_bool("Left").unwrap_or(false);
        self.right_child = tag.get_bool("Right").unwrap_or(false);
    }
}

impl_strong_piece!(Straight, "stronghold_straight", post);

// ---------- StraightStairsDown ----------

pub struct StraightStairsDown {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    entry_door: SmallDoorType,
}

strong_new!(StraightStairsDown {});

impl StraightStairsDown {
    fn post(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        let bb = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let bricks = stone_bricks();
        let stairs = cobblestone_stairs("south");
        gbs!(level, t, box_, mir, rot, true, random, 0, 0, 0, 4, 10, 7);
        sd!(level, t, box_, mir, rot, self.entry_door, 1, 7, 0);
        sd!(level, t, box_, mir, rot, SmallDoorType::Opening, 1, 1, 7);
        for i in 0..6 {
            pb!(level, t, box_, mir, rot, stairs.clone(), 1, 6 - i, 1 + i);
            pb!(level, t, box_, mir, rot, stairs.clone(), 2, 6 - i, 1 + i);
            pb!(level, t, box_, mir, rot, stairs.clone(), 3, 6 - i, 1 + i);
            if i < 5 {
                pb!(level, t, box_, mir, rot, bricks.clone(), 1, 5 - i, 1 + i);
                pb!(level, t, box_, mir, rot, bricks.clone(), 2, 5 - i, 1 + i);
                pb!(level, t, box_, mir, rot, bricks.clone(), 3, 5 - i, 1 + i);
            }
        }
    }
}

impl StrongExtra for StraightStairsDown {}

impl_strong_piece!(StraightStairsDown, "stronghold_straight_stairs_down", post);

// ======================================================================
// Драйвер + фабрика структуры
// ======================================================================

/// `generatePieces` — do-while: `setLargeFeatureSeed(seed + i++)`,
/// `resetPieces()` (новый StartPieceData), StartPiece +
/// addChildren + pending-цикл, `moveBelowSeaLevel(..., 10)`;
/// повтор пока `builder.isEmpty() || portalRoomPiece == null`.
fn generate_pieces(out: &mut Vec<Box<dyn StructurePiece>>, context: &mut GenerationContext) {
    let chunk = context.chunk_pos;
    let mut i: i64 = 0;
    loop {
        out.clear();
        context
            .random
            .set_large_feature_seed(context.seed + i, chunk.x, chunk.z);
        i += 1;

        let start = StartPiece::new(
            &mut context.random,
            chunk.get_min_block_x() + 2,
            chunk.get_min_block_z() + 2,
        );
        let data = start.data.clone();
        data.borrow_mut().start_bb = start.bounding_box();
        let (ty, bb, orient) = (
            start.piece_type_id(),
            start.bounding_box(),
            start.orientation(),
        );
        out.push(Box::new(start));
        stronghold_add_children(
            &ty.path,
            &bb,
            orient,
            0,
            ChildFlags::default(),
            &data,
            out,
            &mut context.random,
        );

        loop {
            let next = {
                let mut d = data.borrow_mut();
                if d.pending_children.is_empty() {
                    None
                } else {
                    let idx = context
                        .random
                        .next_int_bound(d.pending_children.len() as i32)
                        as usize;
                    Some(d.pending_children.remove(idx))
                }
            };
            let Some(pc) = next else { break };
            stronghold_add_children(
                &pc.ty.path,
                &pc.bb,
                pc.orientation,
                pc.gen_depth,
                pc.flags,
                &data,
                out,
                &mut context.random,
            );
        }

        move_below_sea_level(
            out,
            context.level.sea_level(),
            context.level.min_build_height(),
            &mut context.random,
            BELOW_SEA_AMOUNT,
        );

        if !out.is_empty() && data.borrow().portal_room_placed {
            break;
        }
    }
}

/// `StrongholdStructure` — `findGenerationPoint` =
/// `chunkPos.getWorldPosition()` + `generatePieces`.
pub fn stronghold_structure(id_: Identifier, settings: StructureSettings) -> Structure {
    let structure_type = register_structure_type(Identifier::minecraft("stronghold"));
    let mut structure = Structure::new(id_, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(|context: &mut GenerationContext| -> Option<GenerationStub> {
            let pos = context.chunk_pos.get_world_position();
            Some(GenerationStub::new(pos, |out, ctx| {
                generate_pieces(out, ctx);
            }))
        }));
    structure
}
