//! Перенос `structures/NetherFortressStructure.java` +
//! `structures/NetherFortressPieces.java` — незер-крепость:
//! `StartPiece` (BridgeCrossing 19x10x19 @ y=64), weighted-выбор
//! bridge/castle кусков (`PieceWeight`), рекурсивное
//! `pendingChildren`-расширение до `MAX_DEPTH=30`,
//! `moveInsideHeights(48, 70)` и все 15 типов кусков с точной
//! геометрией (мосты, замковые коридоры, monster throne,
//! soul sand/nether wart, сундуки, blaze-спавнер, лава).

use super::super::bounding_box::BoundingBox;
use super::super::structure::{
    GenerationContext, GenerationStub, Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{
    create_chest, fill_column_down, generate_box, get_random_horizontal_direction,
    make_bounding_box, move_inside_heights, piece_set_orientation, place_block,
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
use crate::random::xoroshiro::XoroshiroRandomSource;
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
const MAX_DEPTH: i32 = 30;
/// `LOWEST_Y_POSITION` (`isOkBox`).
const LOWEST_Y_POSITION: i32 = 10;
/// `MAGIC_START_Y`.
const MAGIC_START_Y: i32 = 64;
/// `moveInsideHeights` диапазон.
const MIN_HEIGHT: i32 = 48;
const MAX_HEIGHT: i32 = 70;
/// `BuiltInLootTables.NETHER_BRIDGE`.
const NETHER_BRIDGE_LOOT: &str = "chests/nether_bridge";

// ======================================================================
// PieceWeight / FortressKind / StartPieceData
// ======================================================================

/// `PieceWeight.pieceClass` как enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum FortressKind {
    BridgeStraight,
    BridgeCrossing,
    RoomCrossing,
    StairsRoom,
    MonsterThrone,
    CastleEntrance,
    CastleSmallCorridor,
    CastleSmallCorridorCrossing,
    CastleSmallCorridorRightTurn,
    CastleSmallCorridorLeftTurn,
    CastleCorridorStairs,
    CastleCorridorTBalcony,
    CastleStalkRoom,
    BridgeEndFiller,
}

/// `PieceWeight` — `placeCount` через `Cell` (shared по `Rc`,
/// как Java-мутация статического списка в `StartPiece`).
struct PieceWeight {
    kind: FortressKind,
    weight: i32,
    place_count: Cell<i32>,
    max_place_count: i32,
    allow_in_row: bool,
}

type Weight = Rc<PieceWeight>;

fn weight(kind: FortressKind, weight: i32, max_place_count: i32, allow_in_row: bool) -> Weight {
    Rc::new(PieceWeight {
        kind,
        weight,
        place_count: Cell::new(0),
        max_place_count,
        allow_in_row,
    })
}

impl PieceWeight {
    /// `doPlace(genDepth)` — genDepth в Java не используется.
    fn do_place(&self) -> bool {
        self.max_place_count == 0 || self.place_count.get() < self.max_place_count
    }
    /// `isValid`.
    fn is_valid(&self) -> bool {
        self.max_place_count == 0 || self.place_count.get() < self.max_place_count
    }
}

/// `StartPiece.pendingChildren` — метаданные куска
/// (bb/orientation/genDepth/type), т.к. `Vec<Box<dyn>>`
/// не позволяет вызвать `addChildren` на элементах по ссылке.
struct PendingChild {
    ty: Identifier,
    bb: BoundingBox,
    orientation: Option<Direction>,
    gen_depth: i32,
}

/// Данные `StartPiece` (`availableBridgePieces`,
/// `availableCastlePieces`, `previousPiece`, `pendingChildren`,
/// bounding box для 112-проверки). `RefCell` — доступ через
/// `&StartPiece` в `add_children` и из драйвера.
struct StartPieceData {
    available_bridge: Vec<Weight>,
    available_castle: Vec<Weight>,
    previous_piece: Option<Weight>,
    pending_children: Vec<PendingChild>,
    start_bb: BoundingBox,
}

impl StartPieceData {
    fn new() -> Self {
        Self {
            available_bridge: vec![
                weight(FortressKind::BridgeStraight, 30, 0, true),
                weight(FortressKind::BridgeCrossing, 10, 4, false),
                weight(FortressKind::RoomCrossing, 10, 4, false),
                weight(FortressKind::StairsRoom, 10, 3, false),
                weight(FortressKind::MonsterThrone, 5, 2, false),
                weight(FortressKind::CastleEntrance, 5, 1, false),
            ],
            available_castle: vec![
                weight(FortressKind::CastleSmallCorridor, 25, 0, true),
                weight(FortressKind::CastleSmallCorridorCrossing, 15, 5, false),
                weight(FortressKind::CastleSmallCorridorRightTurn, 5, 10, false),
                weight(FortressKind::CastleSmallCorridorLeftTurn, 5, 10, false),
                weight(FortressKind::CastleCorridorStairs, 10, 3, true),
                weight(FortressKind::CastleCorridorTBalcony, 7, 2, false),
                weight(FortressKind::CastleStalkRoom, 5, 2, false),
            ],
            previous_piece: None,
            pending_children: Vec::new(),
            start_bb: BoundingBox::new(0, 0, 0, -1, -1, -1),
        }
    }
}

// ======================================================================
// Дочерние куски: generateChildForward/Left/Right,
// generateAndAddPiece, generatePiece, createPiece-фабрики
// ======================================================================

/// `generateChildForward` — позиция ребёнка перед куском.
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
    castle_piece: bool,
) {
    let Some(o) = orientation else { return };
    let (x, y, z) = match o {
        Direction::NORTH => (bb.min_x + offset_x, bb.min_y + offset_y, bb.min_z - 1),
        Direction::SOUTH => (bb.min_x + offset_x, bb.min_y + offset_y, bb.max_z + 1),
        Direction::WEST => (bb.min_x - 1, bb.min_y + offset_y, bb.min_z + offset_x),
        Direction::EAST => (bb.max_x + 1, bb.min_y + offset_y, bb.min_z + offset_x),
        _ => return,
    };
    generate_and_add_piece(data, pieces, random, x, y, z, o, gen_depth, castle_piece);
}

/// `generateChildLeft` — позиция ребёнка слева.
/// Аргументы Java: `(offsetY, offsetX)` — offsetY идёт в minY,
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
    castle_piece: bool,
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
    generate_and_add_piece(data, pieces, random, x, y, z, child_o, gen_depth, castle_piece);
}

/// `generateChildRight` — позиция ребёнка справа.
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
    castle_piece: bool,
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
    generate_and_add_piece(data, pieces, random, x, y, z, child_o, gen_depth, castle_piece);
}

/// `generateAndAddPiece` — 112-блоковая проверка от start-bb,
/// `generatePiece(..., genDepth + 1)`, addPiece + pendingChildren.
/// Вне диапазона — `BridgeEndFiller.createPiece` БЕЗ добавления
/// (Java возвращает созданный, но отброшенный кусок — RNG в
/// конструкторе (`selfSeed`) при этом потребляется, если бокс
/// валиден).
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
    castle_piece: bool,
) {
    let start_bb = data.borrow().start_bb;
    if (x - start_bb.min_x).abs() <= 112 && (z - start_bb.min_z).abs() <= 112 {
        if let Some(piece) = generate_piece(data, castle_piece, pieces, random, x, y, z, orientation, gen_depth + 1) {
            let pending = PendingChild {
                ty: piece.piece_type_id(),
                bb: piece.bounding_box(),
                orientation: piece.orientation(),
                gen_depth: piece.gen_depth(),
            };
            pieces.add_piece(piece);
            data.borrow_mut().pending_children.push(pending);
        }
    } else {
        // Java: return BridgeEndFiller.createPiece(...) — результат
        // отбрасывается вызывающим, но конструктор ест RNG.
        let _ = create_piece_kind(
            FortressKind::BridgeEndFiller,
            pieces,
            random,
            x,
            y,
            z,
            orientation,
            gen_depth,
        );
    }
}

/// `updatePieceWeight` — суммарный вес; -1 если ничего не placeable.
fn update_piece_weight(weights: &[Weight]) -> i32 {
    let mut flag = false;
    let mut i = 0;
    for w in weights {
        if w.max_place_count > 0 && w.place_count.get() < w.max_place_count {
            flag = true;
        }
        i += w.weight;
    }
    if flag {
        i
    } else {
        -1
    }
}

/// `generatePiece` — weighted-выбор куска с retry до 5 раз и
/// fallback `BridgeEndFiller`.
#[allow(clippy::too_many_arguments)]
fn generate_piece(
    data: &Rc<RefCell<StartPieceData>>,
    castle: bool,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
    x: i32,
    y: i32,
    z: i32,
    orientation: Direction,
    gen_depth: i32,
) -> Option<Box<dyn StructurePiece>> {
    // Java итерирует живой список и делает weights.remove() при
    // невалидности — клонируем Vec<Rc> (веса shared), удаление
    // делаем в исходном списке по Rc::ptr_eq.
    let weights: Vec<Weight> = {
        let d = data.borrow();
        if castle {
            d.available_castle.clone()
        } else {
            d.available_bridge.clone()
        }
    };
    let total = update_piece_weight(&weights);
    let flag = total > 0 && gen_depth <= MAX_DEPTH;
    let mut i1 = 0;
    while i1 < 5 && flag {
        i1 += 1;
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
                if !w.do_place() || (is_prev && !w.allow_in_row) {
                    break;
                }
                if let Some(piece) = create_piece_kind(w.kind, pieces, random, x, y, z, orientation, gen_depth) {
                    w.place_count.set(w.place_count.get() + 1);
                    data.borrow_mut().previous_piece = Some(w.clone());
                    if !w.is_valid() {
                        let mut d = data.borrow_mut();
                        let list = if castle {
                            &mut d.available_castle
                        } else {
                            &mut d.available_bridge
                        };
                        if let Some(pos) = list.iter().position(|x| Rc::ptr_eq(x, w)) {
                            list.remove(pos);
                        }
                    }
                    return Some(piece);
                }
                // createPiece == null → Java продолжает for-цикл
                // (randomInt уже < 0, следующая итерация тоже войдёт).
            }
        }
    }
    create_piece_kind(FortressKind::BridgeEndFiller, pieces, random, x, y, z, orientation, gen_depth)
}

// ======================================================================
// createPiece-фабрики (`findAndCreateBridgePieceFactory`)
// ======================================================================

/// `isOkBox` — `box.minY() > 10`.
fn is_ok_box(bb: &BoundingBox) -> bool {
    bb.min_y > LOWEST_Y_POSITION
}

#[allow(clippy::too_many_arguments)]
fn create_piece_kind(
    kind: FortressKind,
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
        FortressKind::BridgeStraight => {
            let b = bb(-1, -3, 0, 5, 10, 19);
            ok(&b).then(|| Box::new(BridgeStraight::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::BridgeCrossing => {
            let b = bb(-8, -3, 0, 19, 10, 19);
            ok(&b).then(|| Box::new(BridgeCrossing::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::RoomCrossing => {
            let b = bb(-2, 0, 0, 7, 9, 7);
            ok(&b).then(|| Box::new(RoomCrossing::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::StairsRoom => {
            let b = bb(-2, 0, 0, 7, 11, 7);
            ok(&b).then(|| Box::new(StairsRoom::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::MonsterThrone => {
            let b = bb(-2, 0, 0, 7, 8, 9);
            ok(&b).then(|| Box::new(MonsterThrone::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::CastleEntrance => {
            let b = bb(-5, -3, 0, 13, 14, 13);
            ok(&b).then(|| Box::new(CastleEntrance::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::CastleSmallCorridor => {
            let b = bb(-1, 0, 0, 5, 7, 5);
            ok(&b).then(|| Box::new(CastleSmallCorridor::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::CastleSmallCorridorCrossing => {
            let b = bb(-1, 0, 0, 5, 7, 5);
            ok(&b).then(|| Box::new(CastleSmallCorridorCrossing::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::CastleSmallCorridorRightTurn => {
            let b = bb(-1, 0, 0, 5, 7, 5);
            ok(&b).then(|| {
                let is_needing_chest = random.next_int_bound(3) == 0;
                Box::new(CastleSmallCorridorRightTurn::with_chest(gen_depth, b, orientation, is_needing_chest))
                    as Box<dyn StructurePiece>
            })
        }
        FortressKind::CastleSmallCorridorLeftTurn => {
            let b = bb(-1, 0, 0, 5, 7, 5);
            ok(&b).then(|| {
                let is_needing_chest = random.next_int_bound(3) == 0;
                Box::new(CastleSmallCorridorLeftTurn::with_chest(gen_depth, b, orientation, is_needing_chest))
                    as Box<dyn StructurePiece>
            })
        }
        FortressKind::CastleCorridorStairs => {
            let b = bb(-1, -7, 0, 5, 14, 10);
            ok(&b).then(|| Box::new(CastleCorridorStairs::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::CastleCorridorTBalcony => {
            let b = bb(-3, 0, 0, 9, 7, 9);
            ok(&b).then(|| Box::new(CastleCorridorTBalcony::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::CastleStalkRoom => {
            let b = bb(-5, -3, 0, 13, 14, 13);
            ok(&b).then(|| Box::new(CastleStalkRoom::new(gen_depth, b, orientation)) as Box<dyn StructurePiece>)
        }
        FortressKind::BridgeEndFiller => {
            let b = bb(-1, -3, 0, 5, 10, 8);
            ok(&b).then(|| {
                Box::new(BridgeEndFiller::new(gen_depth, b, orientation, random.next_int()))
                    as Box<dyn StructurePiece>
            })
        }
    }
}

// ======================================================================
// Куски
// ======================================================================

/// `setOrientation` с обновлением mirror/rotation.
macro_rules! impl_orientation {
    () => {
        fn orientation(&self) -> Option<Direction> {
            self.orientation
        }
        fn set_orientation(&mut self, orientation: Option<Direction>) {
            self.orientation = orientation;
            piece_set_orientation(orientation, &mut self.mirror, &mut self.rotation);
        }
    };
}

/// Дополнительные NBT-поля кусков (`Seed`/`Chest`/`Mob`) —
/// default-методы как в Java `addAdditionalSaveData`.
trait FortressExtra {
    fn save_extra(&self, _tag: &mut CompoundTag) {}
    fn load_extra(&mut self, _tag: &CompoundTag) {}
}

/// Общий `StructurePiece`-trait impl для кусков крепости.
/// `$init` — дополнительные поля в `new` (напр.
/// `has_placed_spawner: false`).
macro_rules! impl_fortress_piece {
    ($name:ident, $ty:expr, $post:ident) => {
        impl_fortress_piece!($name, $ty, $post,);
    };
    ($name:ident, $ty:expr, $post:ident, $($init:tt)*) => {
        impl $name {
            /// `new(genDepth, box, orientation)`.
            #[allow(dead_code)]
            fn new(gen_depth: i32, bounding_box: BoundingBox, orientation: Direction) -> Self {
                let mut s = Self {
                    bounding_box,
                    gen_depth,
                    orientation: None,
                    mirror: Mirror::NONE,
                    rotation: Rotation::NONE,
                    $($init)*
                };
                s.set_orientation(Some(orientation));
                s
            }

            fn transform(&self) -> PieceTransform<'_> {
                PieceTransform { bb: &self.bounding_box, orientation: self.orientation }
            }

            /// `getWorldPos(x, y, z)` через PieceTransform.
            #[allow(dead_code)]
            fn world_pos(&self, x: i32, y: i32, z: i32) -> BlockPos {
                self.transform().world_pos(x, y, z)
            }
        }

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
            impl_orientation!();
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
                FortressExtra::save_extra(self, tag);
            }
            fn load_additional_save_data(&mut self, tag: &CompoundTag) {
                FortressExtra::load_extra(self, tag);
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
                    fortress_add_children(
                        $ty,
                        &self.bounding_box,
                        self.orientation,
                        self.gen_depth,
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
        }
    };
}

/// `generateBox` короткая форма: boundary == inside == state,
/// existingOnly = false. `mir`/`rot` — mirror/rotation куска.
macro_rules! gb {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $x1:expr, $y1:expr, $z1:expr, $x2:expr, $y2:expr, $z2:expr, $s:expr) => {
        generate_box(
            $l,
            $t.bb,
            $t.orientation,
            $mir,
            $rot,
            $b,
            $x1,
            $y1,
            $z1,
            $x2,
            $y2,
            $z2,
            &$s,
            &$s,
            false,
        )
    };
}

/// `placeBlock(level, state, x, y, z, box)`.
macro_rules! pb {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $s:expr, $x:expr, $y:expr, $z:expr) => {
        place_block($l, $t.bb, $t.orientation, $mir, $rot, $s, $x, $y, $z, $b)
    };
}

/// `fillColumnDown(level, state, x, y, z, box)`.
macro_rules! fc {
    ($l:expr, $t:expr, $b:expr, $mir:expr, $rot:expr, $s:expr, $x:expr, $y:expr, $z:expr) => {
        fill_column_down($l, $t.bb, $t.orientation, $mir, $rot, $b, &$s, $x, $y, $z)
    };
}

fn bricks() -> BlockState {
    block_state("nether_bricks")
}
fn air() -> BlockState {
    block_state("air")
}
fn soul_sand() -> BlockState {
    block_state("soul_sand")
}
fn nether_wart() -> BlockState {
    block_state("nether_wart")
}
fn lava() -> BlockState {
    block_state("lava")
}
/// `Blocks.NETHER_BRICK_FENCE.defaultBlockState()` — все
/// соединения false.
fn plain_fence() -> BlockState {
    block_state("nether_brick_fence")
}
/// `nether_brick_fence` с четырьмя bool-соединениями.
fn fence(n: bool, s: bool, e: bool, w: bool) -> BlockState {
    let st = block_state("nether_brick_fence");
    let st = with_bool_property(st, "north", n);
    let st = with_bool_property(st, "south", s);
    let st = with_bool_property(st, "east", e);
    with_bool_property(st, "west", w)
}
/// `nether_brick_stairs` с `facing`.
fn stairs(facing: &str) -> BlockState {
    with_string_property(block_state("nether_brick_stairs"), "facing", facing)
}

// ---------- BridgeStraight ----------

pub struct BridgeStraight {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl BridgeStraight {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 3, 0, 4, 4, 18, b);
        gb!(level, t, box_, mir, rot, 1, 5, 0, 3, 7, 18, a);
        gb!(level, t, box_, mir, rot, 0, 5, 0, 0, 5, 18, b);
        gb!(level, t, box_, mir, rot, 4, 5, 0, 4, 5, 18, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 4, 2, 5, b);
        gb!(level, t, box_, mir, rot, 0, 2, 13, 4, 2, 18, b);
        gb!(level, t, box_, mir, rot, 0, 0, 0, 4, 1, 3, b);
        gb!(level, t, box_, mir, rot, 0, 0, 15, 4, 1, 18, b);
        for i in 0..=4 {
            for i1 in 0..=2 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
                fc!(level, t, box_, mir, rot, b, i, -1, 18 - i1);
            }
        }
        // Java: blockState = N+S; blockState1 = +E; blockState2 = +W
        let bs1 = fence(true, true, true, false);
        let bs2 = fence(true, true, false, true);
        gb!(level, t, box_, mir, rot, 0, 1, 1, 0, 4, 1, bs1);
        gb!(level, t, box_, mir, rot, 0, 3, 4, 0, 4, 4, bs1);
        gb!(level, t, box_, mir, rot, 0, 3, 14, 0, 4, 14, bs1);
        gb!(level, t, box_, mir, rot, 0, 1, 17, 0, 4, 17, bs1);
        gb!(level, t, box_, mir, rot, 4, 1, 1, 4, 4, 1, bs2);
        gb!(level, t, box_, mir, rot, 4, 3, 4, 4, 4, 4, bs2);
        gb!(level, t, box_, mir, rot, 4, 3, 14, 4, 4, 14, bs2);
        gb!(level, t, box_, mir, rot, 4, 1, 17, 4, 4, 17, bs2);
    }
}

impl_fortress_piece!(BridgeStraight, "nether_fortress_bridge_straight", build);

// ---------- BridgeEndFiller ----------

pub struct BridgeEndFiller {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    self_seed: i32,
}

impl BridgeEndFiller {
    fn new(gen_depth: i32, bounding_box: BoundingBox, orientation: Direction, self_seed: i32) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            self_seed,
        };
        s.set_orientation(Some(orientation));
        s
    }

    fn transform(&self) -> PieceTransform<'_> {
        PieceTransform { bb: &self.bounding_box, orientation: self.orientation }
    }

    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let mut rng = XoroshiroRandomSource::new(self.self_seed as i64);
        let b = bricks();
        for i in 0..=4 {
            for i1 in 3..=4 {
                let r = rng.next_int_bound(8);
                gb!(level, t, box_, mir, rot, i, i1, 0, i, i1, r, b);
            }
        }
        let r = rng.next_int_bound(8);
        gb!(level, t, box_, mir, rot, 0, 5, 0, 0, 5, r, b);
        let r = rng.next_int_bound(8);
        gb!(level, t, box_, mir, rot, 4, 5, 0, 4, 5, r, b);
        for ix in 0..=4 {
            let i1 = rng.next_int_bound(5);
            gb!(level, t, box_, mir, rot, ix, 2, 0, ix, 2, i1, b);
        }
        for ix in 0..=4 {
            for i1 in 0..=1 {
                let r = rng.next_int_bound(3);
                gb!(level, t, box_, mir, rot, ix, i1, 0, ix, i1, r, b);
            }
        }
    }
}

impl FortressExtra for BridgeEndFiller {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Seed", Tag::Int(self.self_seed));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.self_seed = tag.get_int("Seed").unwrap_or(0);
    }
}

// EndFiller: особый impl (self_seed save/load + нет add_children).
impl StructurePiece for BridgeEndFiller {
    fn bounding_box(&self) -> BoundingBox { self.bounding_box.clone() }
    fn bounding_box_mut(&mut self) -> Option<&mut BoundingBox> { Some(&mut self.bounding_box) }
    fn gen_depth(&self) -> i32 { self.gen_depth }
    fn set_gen_depth(&mut self, gen_depth: i32) { self.gen_depth = gen_depth; }
    impl_orientation!();
    fn rotation(&self) -> Rotation { self.rotation }
    fn mirror(&self) -> Mirror { self.mirror }
    fn piece_type_id(&self) -> Identifier { Identifier::minecraft("nether_fortress_bridge_end_filler") }
    fn add_additional_save_data(&self, tag: &mut CompoundTag) { FortressExtra::save_extra(self, tag); }
    fn load_additional_save_data(&mut self, tag: &CompoundTag) { FortressExtra::load_extra(self, tag); }
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
        _structure_manager: &dyn StructureManagerAccess,
        _generator: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        self.build(level, box_, random);
    }
}

// ---------- BridgeCrossing ----------

pub struct BridgeCrossing {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl BridgeCrossing {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 7, 3, 0, 11, 4, 18, b);
        gb!(level, t, box_, mir, rot, 0, 3, 7, 18, 4, 11, b);
        gb!(level, t, box_, mir, rot, 8, 5, 0, 10, 7, 18, a);
        gb!(level, t, box_, mir, rot, 0, 5, 8, 18, 7, 10, a);
        for &(x1, x2, z1, z2) in &[
            (7, 7, 0, 7), (7, 7, 11, 18), (11, 11, 0, 7), (11, 11, 11, 18),
            (0, 7, 7, 7), (11, 18, 7, 7), (0, 7, 11, 11), (11, 18, 11, 11),
        ] {
            gb!(level, t, box_, mir, rot, x1, 5, z1, x2, 5, z2, b);
        }
        gb!(level, t, box_, mir, rot, 7, 2, 0, 11, 2, 5, b);
        gb!(level, t, box_, mir, rot, 7, 2, 13, 11, 2, 18, b);
        gb!(level, t, box_, mir, rot, 7, 0, 0, 11, 1, 3, b);
        gb!(level, t, box_, mir, rot, 7, 0, 15, 11, 1, 18, b);
        for i in 7..=11 {
            for i1 in 0..=2 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
                fc!(level, t, box_, mir, rot, b, i, -1, 18 - i1);
            }
        }
        gb!(level, t, box_, mir, rot, 0, 2, 7, 5, 2, 11, b);
        gb!(level, t, box_, mir, rot, 13, 2, 7, 18, 2, 11, b);
        gb!(level, t, box_, mir, rot, 0, 0, 7, 3, 1, 11, b);
        gb!(level, t, box_, mir, rot, 15, 0, 7, 18, 1, 11, b);
        for i in 0..=2 {
            for i1 in 7..=11 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
                fc!(level, t, box_, mir, rot, b, 18 - i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(BridgeCrossing, "nether_fortress_bridge_crossing", build);

// ---------- CastleCorridorStairsPiece ----------

pub struct CastleCorridorStairs {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl CastleCorridorStairs {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let stair_s = stairs("south");
        let fence_ns = fence(true, true, false, false);
        let (b, a) = (bricks(), air());
        for i in 0..=9 {
            let max = (7 - i).max(1);
            let min = (max + 5).max(14 - i).min(13);
            let i1 = i;
            gb!(level, t, box_, mir, rot, 0, 0, i, 4, max, i, b);
            gb!(level, t, box_, mir, rot, 1, max + 1, i, 3, min - 1, i, a);
            if i <= 6 {
                pb!(level, t, box_, mir, rot, stair_s.clone(), 1, max + 1, i);
                pb!(level, t, box_, mir, rot, stair_s.clone(), 2, max + 1, i);
                pb!(level, t, box_, mir, rot, stair_s.clone(), 3, max + 1, i);
            }
            gb!(level, t, box_, mir, rot, 0, min, i, 4, min, i, b);
            gb!(level, t, box_, mir, rot, 0, max + 1, i, 0, min - 1, i, b);
            gb!(level, t, box_, mir, rot, 4, max + 1, i, 4, min - 1, i, b);
            if i % 2 == 0 {
                gb!(level, t, box_, mir, rot, 0, max + 2, i, 0, max + 3, i, fence_ns.clone());
                gb!(level, t, box_, mir, rot, 4, max + 2, i, 4, max + 3, i, fence_ns.clone());
            }
            for i2 in 0..=4 {
                fc!(level, t, box_, mir, rot, b, i2, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(CastleCorridorStairs, "nether_fortress_castle_corridor_stairs", build);

// ---------- CastleCorridorTBalconyPiece ----------

pub struct CastleCorridorTBalcony {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl CastleCorridorTBalcony {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let fence_ns = fence(true, true, false, false);
        let fence_we = fence(false, false, true, true);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 0, 0, 8, 1, 8, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 8, 5, 8, a);
        gb!(level, t, box_, mir, rot, 0, 6, 0, 8, 6, 5, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 2, 5, 0, b);
        gb!(level, t, box_, mir, rot, 6, 2, 0, 8, 5, 0, b);
        gb!(level, t, box_, mir, rot, 1, 3, 0, 1, 4, 0, fence_we.clone());
        gb!(level, t, box_, mir, rot, 7, 3, 0, 7, 4, 0, fence_we.clone());
        gb!(level, t, box_, mir, rot, 0, 2, 4, 8, 2, 8, b);
        gb!(level, t, box_, mir, rot, 1, 1, 4, 2, 2, 4, a);
        gb!(level, t, box_, mir, rot, 6, 1, 4, 7, 2, 4, a);
        gb!(level, t, box_, mir, rot, 1, 3, 8, 7, 3, 8, fence_we.clone());
        // E+S и W+S углы.
        pb!(level, t, box_, mir, rot, fence(false, true, true, false), 0, 3, 8);
        pb!(level, t, box_, mir, rot, fence(false, true, false, true), 8, 3, 8);
        gb!(level, t, box_, mir, rot, 0, 3, 6, 0, 3, 7, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 8, 3, 6, 8, 3, 7, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 0, 3, 4, 0, 5, 5, b);
        gb!(level, t, box_, mir, rot, 8, 3, 4, 8, 5, 5, b);
        gb!(level, t, box_, mir, rot, 1, 3, 5, 2, 5, 5, b);
        gb!(level, t, box_, mir, rot, 6, 3, 5, 7, 5, 5, b);
        gb!(level, t, box_, mir, rot, 1, 4, 5, 1, 5, 5, fence_we.clone());
        gb!(level, t, box_, mir, rot, 7, 4, 5, 7, 5, 5, fence_we.clone());
        for i in 0..=5 {
            for i1 in 0..=8 {
                fc!(level, t, box_, mir, rot, b, i1, -1, i);
            }
        }
    }
}

impl_fortress_piece!(CastleCorridorTBalcony, "nether_fortress_castle_corridor_t_balcony", build);

// ---------- CastleEntrance ----------

pub struct CastleEntrance {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl CastleEntrance {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        // Java: blockState = W+E; blockState1 = N+S.
        let fence_we_full = fence(false, false, true, true);
        let fence_ns_full = fence(true, true, false, false);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 3, 0, 12, 4, 12, b);
        gb!(level, t, box_, mir, rot, 0, 5, 0, 12, 13, 12, a);
        gb!(level, t, box_, mir, rot, 0, 5, 0, 1, 12, 12, b);
        gb!(level, t, box_, mir, rot, 11, 5, 0, 12, 12, 12, b);
        gb!(level, t, box_, mir, rot, 2, 5, 11, 4, 12, 12, b);
        gb!(level, t, box_, mir, rot, 8, 5, 11, 10, 12, 12, b);
        gb!(level, t, box_, mir, rot, 5, 9, 11, 7, 12, 12, b);
        gb!(level, t, box_, mir, rot, 2, 5, 0, 4, 12, 1, b);
        gb!(level, t, box_, mir, rot, 8, 5, 0, 10, 12, 1, b);
        gb!(level, t, box_, mir, rot, 5, 9, 0, 7, 12, 1, b);
        gb!(level, t, box_, mir, rot, 2, 11, 2, 10, 12, 10, b);
        gb!(level, t, box_, mir, rot, 5, 8, 0, 7, 8, 0, plain_fence());
        let mut i = 1;
        while i <= 11 {
            gb!(level, t, box_, mir, rot, i, 10, 0, i, 11, 0, fence_we_full.clone());
            gb!(level, t, box_, mir, rot, i, 10, 12, i, 11, 12, fence_we_full.clone());
            gb!(level, t, box_, mir, rot, 0, 10, i, 0, 11, i, fence_ns_full.clone());
            gb!(level, t, box_, mir, rot, 12, 10, i, 12, 11, i, fence_ns_full.clone());
            pb!(level, t, box_, mir, rot, b.clone(), i, 13, 0);
            pb!(level, t, box_, mir, rot, b.clone(), i, 13, 12);
            pb!(level, t, box_, mir, rot, b.clone(), 0, 13, i);
            pb!(level, t, box_, mir, rot, b.clone(), 12, 13, i);
            if i != 11 {
                pb!(level, t, box_, mir, rot, fence_we_full.clone(), i + 1, 13, 0);
                pb!(level, t, box_, mir, rot, fence_we_full.clone(), i + 1, 13, 12);
                pb!(level, t, box_, mir, rot, fence_ns_full.clone(), 0, 13, i + 1);
                pb!(level, t, box_, mir, rot, fence_ns_full.clone(), 12, 13, i + 1);
            }
            i += 2;
        }
        // Углы: N+E, S+E, S+W, N+W.
        pb!(level, t, box_, mir, rot, fence(true, false, true, false), 0, 13, 0);
        pb!(level, t, box_, mir, rot, fence(false, true, true, false), 0, 13, 12);
        pb!(level, t, box_, mir, rot, fence(false, true, false, true), 12, 13, 12);
        pb!(level, t, box_, mir, rot, fence(true, false, false, true), 12, 13, 0);
        // blockState1.setValue(WEST/EAST) = N+S+W / N+S+E.
        let fence_ns_west = with_bool_property(fence_ns_full.clone(), "west", true);
        let fence_ns_east = with_bool_property(fence_ns_full.clone(), "east", true);
        let mut ix = 3;
        while ix <= 9 {
            gb!(level, t, box_, mir, rot, 1, 7, ix, 1, 8, ix, fence_ns_west.clone());
            gb!(level, t, box_, mir, rot, 11, 7, ix, 11, 8, ix, fence_ns_east.clone());
            ix += 2;
        }
        gb!(level, t, box_, mir, rot, 4, 2, 0, 8, 2, 12, b);
        gb!(level, t, box_, mir, rot, 0, 2, 4, 12, 2, 8, b);
        gb!(level, t, box_, mir, rot, 4, 0, 0, 8, 1, 3, b);
        gb!(level, t, box_, mir, rot, 4, 0, 9, 8, 1, 12, b);
        gb!(level, t, box_, mir, rot, 0, 0, 4, 3, 1, 8, b);
        gb!(level, t, box_, mir, rot, 9, 0, 4, 12, 1, 8, b);
        for ix in 4..=8 {
            for i1 in 0..=2 {
                fc!(level, t, box_, mir, rot, b, ix, -1, i1);
                fc!(level, t, box_, mir, rot, b, ix, -1, 12 - i1);
            }
        }
        for ix in 0..=2 {
            for i1 in 4..=8 {
                fc!(level, t, box_, mir, rot, b, ix, -1, i1);
                fc!(level, t, box_, mir, rot, b, 12 - ix, -1, i1);
            }
        }
        gb!(level, t, box_, mir, rot, 5, 5, 5, 7, 5, 7, b);
        gb!(level, t, box_, mir, rot, 6, 1, 6, 6, 4, 6, a);
        pb!(level, t, box_, mir, rot, b.clone(), 6, 0, 6);
        pb!(level, t, box_, mir, rot, lava(), 6, 5, 6);
        let world_pos = self.world_pos(6, 5, 6);
        if box_.is_inside_pos(world_pos) {
            level.schedule_tick(world_pos, &Identifier::minecraft("lava"), 0);
        }
    }
}

impl_fortress_piece!(CastleEntrance, "nether_fortress_castle_entrance", build);

// ---------- CastleSmallCorridorCrossingPiece ----------

pub struct CastleSmallCorridorCrossing {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl CastleSmallCorridorCrossing {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 0, 0, 4, 1, 4, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 4, 5, 4, a);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 0, 5, 0, b);
        gb!(level, t, box_, mir, rot, 4, 2, 0, 4, 5, 0, b);
        gb!(level, t, box_, mir, rot, 0, 2, 4, 0, 5, 4, b);
        gb!(level, t, box_, mir, rot, 4, 2, 4, 4, 5, 4, b);
        gb!(level, t, box_, mir, rot, 0, 6, 0, 4, 6, 4, b);
        for i in 0..=4 {
            for i1 in 0..=4 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(CastleSmallCorridorCrossing, "nether_fortress_castle_small_corridor_crossing", build);

// ---------- CastleSmallCorridorLeftTurnPiece ----------

pub struct CastleSmallCorridorLeftTurn {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    is_needing_chest: bool,
}

impl CastleSmallCorridorLeftTurn {
    /// `new(genDepth, random, box, orientation)` —
    /// `isNeedingChest = random.nextInt(3) == 0` в Java-ctor.
    fn with_chest(gen_depth: i32, bounding_box: BoundingBox, orientation: Direction, is_needing_chest: bool) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            is_needing_chest,
        };
        s.set_orientation(Some(orientation));
        s
    }

    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        // Локальный клон bb — PieceTransform не должен держать
        // borrow на self (нужна мутация is_needing_chest/
        // has_placed_spawner ниже).
        let bb_owned = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb_owned, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let fence_ns = fence(true, true, false, false);
        let fence_we = fence(false, false, true, true);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 0, 0, 4, 1, 4, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 4, 5, 4, a);
        gb!(level, t, box_, mir, rot, 4, 2, 0, 4, 5, 4, b);
        gb!(level, t, box_, mir, rot, 4, 3, 1, 4, 4, 1, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 4, 3, 3, 4, 4, 3, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 0, 2, 0, 0, 5, 0, b);
        gb!(level, t, box_, mir, rot, 0, 2, 4, 3, 5, 4, b);
        gb!(level, t, box_, mir, rot, 1, 3, 4, 1, 4, 4, fence_we.clone());
        gb!(level, t, box_, mir, rot, 3, 3, 4, 3, 4, 4, fence_we.clone());
        if self.is_needing_chest && box_.is_inside_pos(self.world_pos(3, 2, 3)) {
            self.is_needing_chest = false;
            create_chest(
                level,
                &self.bounding_box,
                self.orientation,
                self.mirror,
                self.rotation,
                box_,
                random,
                3,
                2,
                3,
                &Identifier::minecraft(NETHER_BRIDGE_LOOT),
            );
        }
        gb!(level, t, box_, mir, rot, 0, 6, 0, 4, 6, 4, b);
        for i in 0..=4 {
            for i1 in 0..=4 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(CastleSmallCorridorLeftTurn, "nether_fortress_castle_small_corridor_left_turn", build, is_needing_chest: false);

impl FortressExtra for CastleSmallCorridorLeftTurn {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Chest", Tag::Byte(if self.is_needing_chest { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.is_needing_chest = tag.get_bool("Chest").unwrap_or(false);
    }
}

// ---------- CastleSmallCorridorPiece ----------

pub struct CastleSmallCorridor {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl CastleSmallCorridor {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let fence_ns = fence(true, true, false, false);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 0, 0, 4, 1, 4, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 4, 5, 4, a);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 0, 5, 4, b);
        gb!(level, t, box_, mir, rot, 4, 2, 0, 4, 5, 4, b);
        gb!(level, t, box_, mir, rot, 0, 3, 1, 0, 4, 1, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 0, 3, 3, 0, 4, 3, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 4, 3, 1, 4, 4, 1, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 4, 3, 3, 4, 4, 3, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 0, 6, 0, 4, 6, 4, b);
        for i in 0..=4 {
            for i1 in 0..=4 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(CastleSmallCorridor, "nether_fortress_castle_small_corridor", build);

// ---------- CastleSmallCorridorRightTurnPiece ----------

pub struct CastleSmallCorridorRightTurn {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    is_needing_chest: bool,
}

impl CastleSmallCorridorRightTurn {
    /// `new(genDepth, random, box, orientation)`.
    fn with_chest(gen_depth: i32, bounding_box: BoundingBox, orientation: Direction, is_needing_chest: bool) -> Self {
        let mut s = Self {
            bounding_box,
            gen_depth,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            is_needing_chest,
        };
        s.set_orientation(Some(orientation));
        s
    }

    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, random: &mut dyn RandomSource) {
        // Локальный клон bb — PieceTransform не должен держать
        // borrow на self (нужна мутация is_needing_chest/
        // has_placed_spawner ниже).
        let bb_owned = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb_owned, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let fence_ns = fence(true, true, false, false);
        let fence_we = fence(false, false, true, true);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 0, 0, 4, 1, 4, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 4, 5, 4, a);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 0, 5, 4, b);
        gb!(level, t, box_, mir, rot, 0, 3, 1, 0, 4, 1, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 0, 3, 3, 0, 4, 3, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 4, 2, 0, 4, 5, 0, b);
        gb!(level, t, box_, mir, rot, 1, 2, 4, 4, 5, 4, b);
        gb!(level, t, box_, mir, rot, 1, 3, 4, 1, 4, 4, fence_we.clone());
        gb!(level, t, box_, mir, rot, 3, 3, 4, 3, 4, 4, fence_we.clone());
        if self.is_needing_chest && box_.is_inside_pos(self.world_pos(1, 2, 3)) {
            self.is_needing_chest = false;
            create_chest(
                level,
                &self.bounding_box,
                self.orientation,
                self.mirror,
                self.rotation,
                box_,
                random,
                1,
                2,
                3,
                &Identifier::minecraft(NETHER_BRIDGE_LOOT),
            );
        }
        gb!(level, t, box_, mir, rot, 0, 6, 0, 4, 6, 4, b);
        for i in 0..=4 {
            for i1 in 0..=4 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(CastleSmallCorridorRightTurn, "nether_fortress_castle_small_corridor_right_turn", build, is_needing_chest: false);

impl FortressExtra for CastleSmallCorridorRightTurn {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Chest", Tag::Byte(if self.is_needing_chest { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.is_needing_chest = tag.get_bool("Chest").unwrap_or(false);
    }
}

// ---------- CastleStalkRoom ----------

pub struct CastleStalkRoom {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl CastleStalkRoom {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        // Java: blockState = W+E, blockState1 = N+S,
        // blockState2 = N+S+W, blockState3 = N+S+E.
        let bs = fence(false, false, true, true);
        let bs1 = fence(true, true, false, false);
        let bs2 = with_bool_property(bs1.clone(), "west", true);
        let bs3 = with_bool_property(bs1.clone(), "east", true);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 3, 0, 12, 4, 12, b);
        gb!(level, t, box_, mir, rot, 0, 5, 0, 12, 13, 12, a);
        gb!(level, t, box_, mir, rot, 0, 5, 0, 1, 12, 12, b);
        gb!(level, t, box_, mir, rot, 11, 5, 0, 12, 12, 12, b);
        gb!(level, t, box_, mir, rot, 2, 5, 11, 4, 12, 12, b);
        gb!(level, t, box_, mir, rot, 8, 5, 11, 10, 12, 12, b);
        gb!(level, t, box_, mir, rot, 5, 9, 11, 7, 12, 12, b);
        gb!(level, t, box_, mir, rot, 2, 5, 0, 4, 12, 1, b);
        gb!(level, t, box_, mir, rot, 8, 5, 0, 10, 12, 1, b);
        gb!(level, t, box_, mir, rot, 5, 9, 0, 7, 12, 1, b);
        gb!(level, t, box_, mir, rot, 2, 11, 2, 10, 12, 10, b);
        let mut i = 1;
        while i <= 11 {
            gb!(level, t, box_, mir, rot, i, 10, 0, i, 11, 0, bs.clone());
            gb!(level, t, box_, mir, rot, i, 10, 12, i, 11, 12, bs.clone());
            gb!(level, t, box_, mir, rot, 0, 10, i, 0, 11, i, bs1.clone());
            gb!(level, t, box_, mir, rot, 12, 10, i, 12, 11, i, bs1.clone());
            pb!(level, t, box_, mir, rot, b.clone(), i, 13, 0);
            pb!(level, t, box_, mir, rot, b.clone(), i, 13, 12);
            pb!(level, t, box_, mir, rot, b.clone(), 0, 13, i);
            pb!(level, t, box_, mir, rot, b.clone(), 12, 13, i);
            if i != 11 {
                pb!(level, t, box_, mir, rot, bs.clone(), i + 1, 13, 0);
                pb!(level, t, box_, mir, rot, bs.clone(), i + 1, 13, 12);
                pb!(level, t, box_, mir, rot, bs1.clone(), 0, 13, i + 1);
                pb!(level, t, box_, mir, rot, bs1.clone(), 12, 13, i + 1);
            }
            i += 2;
        }
        // Углы: N+E, S+E, S+W, N+W.
        pb!(level, t, box_, mir, rot, fence(true, false, true, false), 0, 13, 0);
        pb!(level, t, box_, mir, rot, fence(false, true, true, false), 0, 13, 12);
        pb!(level, t, box_, mir, rot, fence(false, true, false, true), 12, 13, 12);
        pb!(level, t, box_, mir, rot, fence(true, false, false, true), 12, 13, 0);
        let mut ix = 3;
        while ix <= 9 {
            gb!(level, t, box_, mir, rot, 1, 7, ix, 1, 8, ix, bs2.clone());
            gb!(level, t, box_, mir, rot, 11, 7, ix, 11, 8, ix, bs3.clone());
            ix += 2;
        }
        let stair_n = stairs("north");
        for i1 in 0..=6 {
            let i2 = i1 + 4;
            for i3 in 5..=7 {
                pb!(level, t, box_, mir, rot, stair_n.clone(), i3, 5 + i1, i2);
            }
            if (5..=8).contains(&i2) {
                gb!(level, t, box_, mir, rot, 5, 5, i2, 7, i1 + 4, i2, b);
            } else if (9..=10).contains(&i2) {
                gb!(level, t, box_, mir, rot, 5, 8, i2, 7, i1 + 4, i2, b);
            }
            if i1 >= 1 {
                gb!(level, t, box_, mir, rot, 5, 6 + i1, i2, 7, 9 + i1, i2, a);
            }
        }
        for i1 in 5..=7 {
            pb!(level, t, box_, mir, rot, stair_n.clone(), i1, 12, 11);
        }
        gb!(level, t, box_, mir, rot, 5, 6, 7, 5, 7, 7, bs3.clone());
        gb!(level, t, box_, mir, rot, 7, 6, 7, 7, 7, 7, bs2.clone());
        gb!(level, t, box_, mir, rot, 5, 13, 12, 7, 13, 12, a);
        gb!(level, t, box_, mir, rot, 2, 5, 2, 3, 5, 3, b);
        gb!(level, t, box_, mir, rot, 2, 5, 9, 3, 5, 10, b);
        gb!(level, t, box_, mir, rot, 2, 5, 4, 2, 5, 8, b);
        gb!(level, t, box_, mir, rot, 9, 5, 2, 10, 5, 3, b);
        gb!(level, t, box_, mir, rot, 9, 5, 9, 10, 5, 10, b);
        gb!(level, t, box_, mir, rot, 10, 5, 4, 10, 5, 8, b);
        let stair_e = with_string_property(stair_n.clone(), "facing", "east");
        let stair_w = with_string_property(stair_n.clone(), "facing", "west");
        pb!(level, t, box_, mir, rot, stair_w.clone(), 4, 5, 2);
        pb!(level, t, box_, mir, rot, stair_w.clone(), 4, 5, 3);
        pb!(level, t, box_, mir, rot, stair_w.clone(), 4, 5, 9);
        pb!(level, t, box_, mir, rot, stair_w.clone(), 4, 5, 10);
        pb!(level, t, box_, mir, rot, stair_e.clone(), 8, 5, 2);
        pb!(level, t, box_, mir, rot, stair_e.clone(), 8, 5, 3);
        pb!(level, t, box_, mir, rot, stair_e.clone(), 8, 5, 9);
        pb!(level, t, box_, mir, rot, stair_e.clone(), 8, 5, 10);
        gb!(level, t, box_, mir, rot, 3, 4, 4, 4, 4, 8, soul_sand());
        gb!(level, t, box_, mir, rot, 8, 4, 4, 9, 4, 8, soul_sand());
        gb!(level, t, box_, mir, rot, 3, 5, 4, 4, 5, 8, nether_wart());
        gb!(level, t, box_, mir, rot, 8, 5, 4, 9, 5, 8, nether_wart());
        gb!(level, t, box_, mir, rot, 4, 2, 0, 8, 2, 12, b);
        gb!(level, t, box_, mir, rot, 0, 2, 4, 12, 2, 8, b);
        gb!(level, t, box_, mir, rot, 4, 0, 0, 8, 1, 3, b);
        gb!(level, t, box_, mir, rot, 4, 0, 9, 8, 1, 12, b);
        gb!(level, t, box_, mir, rot, 0, 0, 4, 3, 1, 8, b);
        gb!(level, t, box_, mir, rot, 9, 0, 4, 12, 1, 8, b);
        for i3 in 4..=8 {
            for i4 in 0..=2 {
                fc!(level, t, box_, mir, rot, b, i3, -1, i4);
                fc!(level, t, box_, mir, rot, b, i3, -1, 12 - i4);
            }
        }
        for i3 in 0..=2 {
            for i4 in 4..=8 {
                fc!(level, t, box_, mir, rot, b, i3, -1, i4);
                fc!(level, t, box_, mir, rot, b, 12 - i3, -1, i4);
            }
        }
    }
}

impl_fortress_piece!(CastleStalkRoom, "nether_fortress_castle_stalk_room", build);

// ---------- MonsterThrone ----------

pub struct MonsterThrone {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    has_placed_spawner: bool,
}

impl MonsterThrone {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        // Локальный клон bb — PieceTransform не должен держать
        // borrow на self (нужна мутация is_needing_chest/
        // has_placed_spawner ниже).
        let bb_owned = self.bounding_box.clone();
        let t = PieceTransform { bb: &bb_owned, orientation: self.orientation };
        let (mir, rot) = (self.mirror, self.rotation);
        let fence_we = fence(false, false, true, true);
        let fence_ns = fence(true, true, false, false);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 2, 0, 6, 7, 7, a);
        gb!(level, t, box_, mir, rot, 1, 0, 0, 5, 1, 7, b);
        gb!(level, t, box_, mir, rot, 1, 2, 1, 5, 2, 7, b);
        gb!(level, t, box_, mir, rot, 1, 3, 2, 5, 3, 7, b);
        gb!(level, t, box_, mir, rot, 1, 4, 3, 5, 4, 7, b);
        gb!(level, t, box_, mir, rot, 1, 2, 0, 1, 4, 2, b);
        gb!(level, t, box_, mir, rot, 5, 2, 0, 5, 4, 2, b);
        gb!(level, t, box_, mir, rot, 1, 5, 2, 1, 5, 3, b);
        gb!(level, t, box_, mir, rot, 5, 5, 2, 5, 5, 3, b);
        gb!(level, t, box_, mir, rot, 0, 5, 3, 0, 5, 8, b);
        gb!(level, t, box_, mir, rot, 6, 5, 3, 6, 5, 8, b);
        gb!(level, t, box_, mir, rot, 1, 5, 8, 5, 5, 8, b);
        // Java: (0,6,3)=E+N, (6,6,3)=W+N, (0,6,8)=E+S, (6,6,8)=W+S.
        pb!(level, t, box_, mir, rot, fence(false, false, false, true), 1, 6, 3);
        pb!(level, t, box_, mir, rot, fence(false, false, true, false), 5, 6, 3);
        pb!(level, t, box_, mir, rot, fence(true, false, true, false), 0, 6, 3);
        pb!(level, t, box_, mir, rot, fence(true, false, false, true), 6, 6, 3);
        gb!(level, t, box_, mir, rot, 0, 6, 4, 0, 6, 7, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 6, 6, 4, 6, 6, 7, fence_ns.clone());
        pb!(level, t, box_, mir, rot, fence(false, true, true, false), 0, 6, 8);
        pb!(level, t, box_, mir, rot, fence(false, true, false, true), 6, 6, 8);
        gb!(level, t, box_, mir, rot, 1, 6, 8, 5, 6, 8, fence_we.clone());
        pb!(level, t, box_, mir, rot, fence(false, false, true, false), 1, 7, 8);
        gb!(level, t, box_, mir, rot, 2, 7, 8, 4, 7, 8, fence_we.clone());
        pb!(level, t, box_, mir, rot, fence(false, false, false, true), 5, 7, 8);
        pb!(level, t, box_, mir, rot, fence(false, false, true, false), 2, 8, 8);
        pb!(level, t, box_, mir, rot, fence_we.clone(), 3, 8, 8);
        pb!(level, t, box_, mir, rot, fence(false, false, false, true), 4, 8, 8);
        if !self.has_placed_spawner {
            let world_pos = self.world_pos(3, 5, 5);
            if box_.is_inside_pos(world_pos) {
                self.has_placed_spawner = true;
                // placeCraftSpawner(level, worldPos, BLAZE, UPDATE_CLIENTS)
                level.set_block_state(world_pos, &block_state("spawner"));
                level.place_structure_spawner(world_pos, "blaze");
            }
        }
        for i in 0..=6 {
            for i1 in 0..=6 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(MonsterThrone, "nether_fortress_monster_throne", build, has_placed_spawner: false);

impl FortressExtra for MonsterThrone {
    fn save_extra(&self, tag: &mut CompoundTag) {
        tag.put("Mob", Tag::Byte(if self.has_placed_spawner { 1 } else { 0 }));
    }
    fn load_extra(&mut self, tag: &CompoundTag) {
        self.has_placed_spawner = tag.get_bool("Mob").unwrap_or(false);
    }
}

// ---------- RoomCrossing ----------

pub struct RoomCrossing {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl RoomCrossing {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let fence_we = fence(false, false, true, true);
        let fence_ns = fence(true, true, false, false);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 0, 0, 6, 1, 6, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 6, 7, 6, a);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 1, 6, 0, b);
        gb!(level, t, box_, mir, rot, 0, 2, 6, 1, 6, 6, b);
        gb!(level, t, box_, mir, rot, 5, 2, 0, 6, 6, 0, b);
        gb!(level, t, box_, mir, rot, 5, 2, 6, 6, 6, 6, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 0, 6, 1, b);
        gb!(level, t, box_, mir, rot, 0, 2, 5, 0, 6, 6, b);
        gb!(level, t, box_, mir, rot, 6, 2, 0, 6, 6, 1, b);
        gb!(level, t, box_, mir, rot, 6, 2, 5, 6, 6, 6, b);
        gb!(level, t, box_, mir, rot, 2, 6, 0, 4, 6, 0, b);
        gb!(level, t, box_, mir, rot, 2, 5, 0, 4, 5, 0, fence_we.clone());
        gb!(level, t, box_, mir, rot, 2, 6, 6, 4, 6, 6, b);
        gb!(level, t, box_, mir, rot, 2, 5, 6, 4, 5, 6, fence_we.clone());
        gb!(level, t, box_, mir, rot, 0, 6, 2, 0, 6, 4, b);
        gb!(level, t, box_, mir, rot, 0, 5, 2, 0, 5, 4, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 6, 6, 2, 6, 6, 4, b);
        gb!(level, t, box_, mir, rot, 6, 5, 2, 6, 5, 4, fence_ns.clone());
        for i in 0..=6 {
            for i1 in 0..=6 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(RoomCrossing, "nether_fortress_room_crossing", build);

// ---------- StairsRoom ----------

pub struct StairsRoom {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
}

impl StairsRoom {
    fn build(&mut self, level: &dyn LevelReader, box_: &BoundingBox, _random: &mut dyn RandomSource) {
        let t = self.transform();
        let (mir, rot) = (self.mirror, self.rotation);
        let fence_we = fence(false, false, true, true);
        let fence_ns = fence(true, true, false, false);
        let (b, a) = (bricks(), air());
        gb!(level, t, box_, mir, rot, 0, 0, 0, 6, 1, 6, b);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 6, 10, 6, a);
        gb!(level, t, box_, mir, rot, 0, 2, 0, 1, 8, 0, b);
        gb!(level, t, box_, mir, rot, 5, 2, 0, 6, 8, 0, b);
        gb!(level, t, box_, mir, rot, 0, 2, 1, 0, 8, 6, b);
        gb!(level, t, box_, mir, rot, 6, 2, 1, 6, 8, 6, b);
        gb!(level, t, box_, mir, rot, 1, 2, 6, 5, 8, 6, b);
        gb!(level, t, box_, mir, rot, 0, 3, 2, 0, 5, 4, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 6, 3, 2, 6, 5, 2, fence_ns.clone());
        gb!(level, t, box_, mir, rot, 6, 3, 4, 6, 5, 4, fence_ns.clone());
        pb!(level, t, box_, mir, rot, b.clone(), 5, 2, 5);
        gb!(level, t, box_, mir, rot, 4, 2, 5, 4, 3, 5, b);
        gb!(level, t, box_, mir, rot, 3, 2, 5, 3, 4, 5, b);
        gb!(level, t, box_, mir, rot, 2, 2, 5, 2, 5, 5, b);
        gb!(level, t, box_, mir, rot, 1, 2, 5, 1, 6, 5, b);
        gb!(level, t, box_, mir, rot, 1, 7, 1, 5, 7, 4, b);
        gb!(level, t, box_, mir, rot, 6, 8, 2, 6, 8, 4, a);
        gb!(level, t, box_, mir, rot, 2, 6, 0, 4, 8, 0, b);
        gb!(level, t, box_, mir, rot, 2, 5, 0, 4, 5, 0, fence_we.clone());
        for i in 0..=6 {
            for i1 in 0..=6 {
                fc!(level, t, box_, mir, rot, b, i, -1, i1);
            }
        }
    }
}

impl_fortress_piece!(StairsRoom, "nether_fortress_stairs_room", build);

// `FortressExtra` default-имплементации для кусков без
// дополнительных NBT-полей.
impl FortressExtra for BridgeStraight {}
impl FortressExtra for BridgeCrossing {}
impl FortressExtra for CastleCorridorStairs {}
impl FortressExtra for CastleCorridorTBalcony {}
impl FortressExtra for CastleEntrance {}
impl FortressExtra for CastleSmallCorridorCrossing {}
impl FortressExtra for CastleSmallCorridor {}
impl FortressExtra for CastleStalkRoom {}
impl FortressExtra for RoomCrossing {}
impl FortressExtra for StairsRoom {}

// ======================================================================
// StartPiece + addChildren-dispatch + генерация
// ======================================================================

/// `StartPiece` — `BridgeCrossing`-бокс 19x10x19 @ y=64,
/// хранит `StartPieceData`.
pub struct StartPiece {
    bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    data: Rc<RefCell<StartPieceData>>,
}

impl StartPiece {
    fn new(random: &mut dyn RandomSource, x: i32, z: i32) -> Self {
        let orientation = get_random_horizontal_direction(random);
        let bb = make_bounding_box(x, MAGIC_START_Y, z, orientation, 19, 10, 19);
        let mut s = Self {
            bounding_box: bb,
            gen_depth: 0,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            data: Rc::new(RefCell::new(StartPieceData::new())),
        };
        s.set_orientation(Some(orientation));
        s
    }

    fn transform(&self) -> PieceTransform<'_> {
        PieceTransform { bb: &self.bounding_box, orientation: self.orientation }
    }
}

/// `BridgeCrossing.postProcess` — общий для StartPiece
/// (Java: StartPiece extends BridgeCrossing без переопределения).
fn bridge_crossing_build(level: &dyn LevelReader, t: &PieceTransform, box_: &BoundingBox, mir: Mirror, rot: Rotation) {
    let (b, a) = (bricks(), air());
    gb!(level, t, box_, mir, rot, 7, 3, 0, 11, 4, 18, b);
    gb!(level, t, box_, mir, rot, 0, 3, 7, 18, 4, 11, b);
    gb!(level, t, box_, mir, rot, 8, 5, 0, 10, 7, 18, a);
    gb!(level, t, box_, mir, rot, 0, 5, 8, 18, 7, 10, a);
    for &(x1, x2, z1, z2) in &[
        (7, 7, 0, 7), (7, 7, 11, 18), (11, 11, 0, 7), (11, 11, 11, 18),
        (0, 7, 7, 7), (11, 18, 7, 7), (0, 7, 11, 11), (11, 18, 11, 11),
    ] {
        gb!(level, t, box_, mir, rot, x1, 5, z1, x2, 5, z2, b);
    }
    gb!(level, t, box_, mir, rot, 7, 2, 0, 11, 2, 5, b);
    gb!(level, t, box_, mir, rot, 7, 2, 13, 11, 2, 18, b);
    gb!(level, t, box_, mir, rot, 7, 0, 0, 11, 1, 3, b);
    gb!(level, t, box_, mir, rot, 7, 0, 15, 11, 1, 18, b);
    for i in 7..=11 {
        for i1 in 0..=2 {
            fc!(level, t, box_, mir, rot, b, i, -1, i1);
            fc!(level, t, box_, mir, rot, b, i, -1, 18 - i1);
        }
    }
    gb!(level, t, box_, mir, rot, 0, 2, 7, 5, 2, 11, b);
    gb!(level, t, box_, mir, rot, 13, 2, 7, 18, 2, 11, b);
    gb!(level, t, box_, mir, rot, 0, 0, 7, 3, 1, 11, b);
    gb!(level, t, box_, mir, rot, 15, 0, 7, 18, 1, 11, b);
    for i in 0..=2 {
        for i1 in 7..=11 {
            fc!(level, t, box_, mir, rot, b, i, -1, i1);
            fc!(level, t, box_, mir, rot, b, 18 - i, -1, i1);
        }
    }
}

impl StructurePiece for StartPiece {
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
    fn piece_type_id(&self) -> Identifier { Identifier::minecraft("nether_fortress_start") }
    fn add_additional_save_data(&self, _tag: &mut CompoundTag) {}
    fn load_additional_save_data(&mut self, _tag: &CompoundTag) {}
    fn as_any(&self) -> Option<&dyn std::any::Any> { Some(self) }
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
            fortress_add_children(
                "nether_fortress_start",
                &self.bounding_box,
                self.orientation,
                self.gen_depth,
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
        _random: &mut dyn RandomSource,
        box_: &BoundingBox,
        _chunk_pos: ChunkPos,
        _pos: BlockPos,
    ) {
        let t = self.transform();
        bridge_crossing_build(level, &t, box_, self.mirror, self.rotation);
    }
}

/// `addChildren`-dispatch по типу куска (Java: каждый класс
/// вызывает generateChild* в своём порядке).
fn fortress_add_children(
    ty: &str,
    bb: &BoundingBox,
    orientation: Option<Direction>,
    gen_depth: i32,
    data: &Rc<RefCell<StartPieceData>>,
    pieces: &mut dyn StructurePieceAccessor,
    random: &mut dyn RandomSource,
) {
    use Direction::*;
    match ty {
        // BridgeCrossing / StartPiece
        "nether_fortress_start" | "nether_fortress_bridge_crossing" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 8, 3, false);
            child_left(bb, orientation, gen_depth, data, pieces, random, 3, 8, false);
            child_right(bb, orientation, gen_depth, data, pieces, random, 3, 8, false);
        }
        "nether_fortress_bridge_straight" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 3, false);
        }
        "nether_fortress_room_crossing" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 2, 0, false);
            child_left(bb, orientation, gen_depth, data, pieces, random, 0, 2, false);
            child_right(bb, orientation, gen_depth, data, pieces, random, 0, 2, false);
        }
        "nether_fortress_stairs_room" => {
            child_right(bb, orientation, gen_depth, data, pieces, random, 6, 2, false);
        }
        "nether_fortress_monster_throne" | "nether_fortress_bridge_end_filler" => {}
        "nether_fortress_castle_entrance" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 5, 3, true);
        }
        "nether_fortress_castle_corridor_stairs" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 0, true);
        }
        "nether_fortress_castle_corridor_t_balcony" => {
            let i = if orientation == Some(WEST) || orientation == Some(NORTH) { 5 } else { 1 };
            let castle1 = random.next_int_bound(8) > 0;
            child_left(bb, orientation, gen_depth, data, pieces, random, 0, i, castle1);
            let castle2 = random.next_int_bound(8) > 0;
            child_right(bb, orientation, gen_depth, data, pieces, random, 0, i, castle2);
        }
        "nether_fortress_castle_stalk_room" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 5, 3, true);
            child_forward(bb, orientation, gen_depth, data, pieces, random, 5, 11, true);
        }
        "nether_fortress_castle_small_corridor" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 0, true);
        }
        "nether_fortress_castle_small_corridor_crossing" => {
            child_forward(bb, orientation, gen_depth, data, pieces, random, 1, 0, true);
            child_left(bb, orientation, gen_depth, data, pieces, random, 0, 1, true);
            child_right(bb, orientation, gen_depth, data, pieces, random, 0, 1, true);
        }
        "nether_fortress_castle_small_corridor_left_turn" => {
            child_left(bb, orientation, gen_depth, data, pieces, random, 0, 1, true);
        }
        "nether_fortress_castle_small_corridor_right_turn" => {
            child_right(bb, orientation, gen_depth, data, pieces, random, 0, 1, true);
        }
        _ => {}
    }
}

/// `generatePieces` — StartPiece + addChildren + pending-цикл +
/// `moveInsideHeights(48, 70)`.
fn generate_pieces(out: &mut Vec<Box<dyn StructurePiece>>, context: &mut GenerationContext) {
    let chunk = context.chunk_pos;
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
    fortress_add_children(&ty.path, &bb, orient, 0, &data, out, &mut context.random);

    loop {
        let next = {
            let mut d = data.borrow_mut();
            if d.pending_children.is_empty() {
                None
            } else {
                let i = context
                    .random
                    .next_int_bound(d.pending_children.len() as i32) as usize;
                Some(d.pending_children.remove(i))
            }
        };
        let Some(pc) = next else { break };
        fortress_add_children(
            &pc.ty.path,
            &pc.bb,
            pc.orientation,
            pc.gen_depth,
            &data,
            out,
            &mut context.random,
        );
    }
    move_inside_heights(out, &mut context.random, MIN_HEIGHT, MAX_HEIGHT);
}

/// `NetherFortressStructure` — `findGenerationPoint` =
/// `(minBlockX, 64, minBlockZ)` + `generatePieces`.
pub fn nether_fortress_structure(
    id_: Identifier,
    settings: StructureSettings,
) -> Structure {
    let structure_type = register_structure_type(Identifier::minecraft("nether_fortress"));
    let mut structure = Structure::new(id_, settings, structure_type);
    structure.find_generation_point =
        Some(Arc::new(|context: &mut GenerationContext| -> Option<GenerationStub> {
            let chunk_pos = context.chunk_pos;
            let pos = BlockPos::new(chunk_pos.get_min_block_x(), MAGIC_START_Y, chunk_pos.get_min_block_z());
            Some(GenerationStub::new(pos, |out, ctx| {
                generate_pieces(out, ctx);
            }))
        }));
    structure
}
