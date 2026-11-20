//! Перенос `structures/OceanMonumentStructure.java` +
//! `structures/OceanMonumentPieces.java` — океанический монумент:
//! граф из 75+3 `RoomDefinition`, фиттеры кусков, 12 типов кусков,
//! water-fill генерация, elder guardian'ы.
//!
//! Все куски — один `OceanMonumentPiece` с `kind`-дискриминантом
//! (в Java — hierarchy над общим базовым классом).

use super::super::bounding_box::BoundingBox;
use super::super::pieces_container::PiecesContainer;
use super::super::structure::{
    on_top_of_chunk_center, GenerationContext, GenerationStub, Structure,
    StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{
    fill_column_down, generate_box, get_block, make_bounding_box,
    piece_set_orientation,
    place_block, PieceTransform, StructurePiece, StructurePieceAccessor,
};
use super::super::structure_type::{register_structure_type, StructureType};
use super::block_state;
use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::nbt::compound::CompoundTag;
use crate::random::{legacy::LegacyRandomSource, RandomSource};
use crate::resources::identifier::Identifier;
use crate::util::shuffle;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::levelgen::heightmap::HeightmapType;
use crate::world::level::levelgen::structure::pools::structure_pool_element::PoolGeneratorAccess;
use crate::world::level::levelgen::worldgen_random::WorldgenRandom;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

// ------------------------------------------------------------------
// Блок-состояния (static, lazy через OnceLock)
// ------------------------------------------------------------------

fn base_gray() -> &'static BlockState {
    static S: std::sync::OnceLock<BlockState> = std::sync::OnceLock::new();
    S.get_or_init(|| block_state("prismarine"))
}
fn base_light() -> &'static BlockState {
    static S: std::sync::OnceLock<BlockState> = std::sync::OnceLock::new();
    S.get_or_init(|| block_state("prismarine_bricks"))
}
fn base_black() -> &'static BlockState {
    static S: std::sync::OnceLock<BlockState> = std::sync::OnceLock::new();
    S.get_or_init(|| block_state("dark_prismarine"))
}
fn lamp_block() -> &'static BlockState {
    static S: std::sync::OnceLock<BlockState> = std::sync::OnceLock::new();
    S.get_or_init(|| block_state("sea_lantern"))
}
fn fill_block() -> &'static BlockState {
    static S: std::sync::OnceLock<BlockState> = std::sync::OnceLock::new();
    S.get_or_init(|| block_state("water"))
}
fn gold_block() -> &'static BlockState {
    static S: std::sync::OnceLock<BlockState> = std::sync::OnceLock::new();
    S.get_or_init(|| block_state("gold_block"))
}
fn wet_sponge() -> &'static BlockState {
    static S: std::sync::OnceLock<BlockState> = std::sync::OnceLock::new();
    S.get_or_init(|| block_state("wet_sponge"))
}

/// `FILL_KEEP` — блоки, которые generateWaterBox не трогает.
fn fill_keep(state: &BlockState) -> bool {
    matches!(
        state.block.id.path.as_str(),
        "ice" | "packed_ice" | "blue_ice" | "water"
    )
}

// ------------------------------------------------------------------
// RoomDefinition
// ------------------------------------------------------------------

/// `RoomDefinition` — узел графа комнат монумента. `Rc<RefCell>`
/// нужен для двунаправленных `connections` (циклы — как в Java,
/// живут до конца генерации).
pub struct RoomDefinition {
    /// `index` — индекс в 5×3×5 сетке (y*25 + z*5 + x) или 1001+.
    pub index: i32,
    /// `connections[6]` — по `Direction.get3DDataValue`.
    pub connections: [Option<Rc<RefCell<RoomDefinition>>>; 6],
    /// `hasOpening[6]`.
    pub has_opening: [bool; 6],
    /// `claimed`.
    pub claimed: bool,
    /// `isSource`.
    pub is_source: bool,
    /// `scanIndex` — метка обхода `findSource`.
    scan_index: i32,
}

impl RoomDefinition {
    fn new(index: i32) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            index,
            connections: [None, None, None, None, None, None],
            has_opening: [false; 6],
            claimed: false,
            is_source: false,
            scan_index: 0,
        }))
    }

    /// `setConnection(direction, connectingRoom)` — двунаправленно.
    fn set_connection(
        this: &Rc<RefCell<Self>>,
        direction: Direction,
        other: &Rc<RefCell<Self>>,
    ) {
        let di = direction.get_3d_data_value() as usize;
        let oi = direction.get_opposite().get_3d_data_value() as usize;
        this.borrow_mut().connections[di] = Some(other.clone());
        other.borrow_mut().connections[oi] = Some(this.clone());
    }

    /// `updateOpenings`.
    fn update_openings(&mut self) {
        for i in 0..6 {
            self.has_opening[i] = self.connections[i].is_some();
        }
    }

    /// `isSpecial` — index >= 75.
    fn is_special(&self) -> bool {
        self.index >= 75
    }

    /// `countOpenings`.
    fn count_openings(&self) -> i32 {
        self.has_opening.iter().filter(|b| **b).count() as i32
    }
}

/// `findSource(index)` — достижимость source-комнаты по открытым
/// проходам (рекурсивный DFS с метками scanIndex).
fn room_find_source(room: &Rc<RefCell<RoomDefinition>>, index: i32) -> bool {
    if room.borrow().is_source {
        return true;
    }
    room.borrow_mut().scan_index = index;
    for i in 0..6 {
        let (conn, opening) = {
            let r = room.borrow();
            (r.connections[i].clone(), r.has_opening[i])
        };
        if let Some(c) = conn {
            if opening && c.borrow().scan_index != index && room_find_source(&c, index) {
                return true;
            }
        }
    }
    false
}

// ------------------------------------------------------------------
// Общие хелперы OceanMonumentPiece
// ------------------------------------------------------------------

/// `getWorldX(x, z)`.
fn world_x(bb: &BoundingBox, o: Option<Direction>, x: i32, z: i32) -> i32 {
    PieceTransform { bb, orientation: o }.world_x(x, z)
}

/// `getWorldY(y)` — minY + y (vanilla не зависит от orientation;
/// для ориентированных кусков PieceTransform делает то же).
fn world_y(bb: &BoundingBox, y: i32) -> i32 {
    bb.min_y + y
}

/// `getWorldZ(x, z)`.
fn world_z(bb: &BoundingBox, o: Option<Direction>, x: i32, z: i32) -> i32 {
    PieceTransform { bb, orientation: o }.world_z(x, z)
}

/// `getWorldPos(x, y, z)`.
fn world_pos(bb: &BoundingBox, o: Option<Direction>, x: i32, y: i32, z: i32) -> BlockPos {
    PieceTransform { bb, orientation: o }.world_pos(x, y, z)
}

/// `BoundingBox.fromCorners` по BlockPos.
fn corners_bb(a: BlockPos, b: BlockPos) -> BoundingBox {
    BoundingBox::from_corners(
        crate::core::vec3i::Vec3i::new(a.x, a.y, a.z),
        crate::core::vec3i::Vec3i::new(b.x, b.y, b.z),
    )
}

/// `generateBox` короткая форма (existing_only=false).
#[allow(clippy::too_many_arguments)]
fn gb(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    o: Option<Direction>,
    m: Mirror,
    r: Rotation,
    bx: &BoundingBox,
    x1: i32,
    y1: i32,
    z1: i32,
    x2: i32,
    y2: i32,
    z2: i32,
    boundary: &BlockState,
    inside: &BlockState,
) {
    generate_box(level, bb, o, m, r, bx, x1, y1, z1, x2, y2, z2, boundary, inside, false);
}

/// `placeBlock` короткая форма.
#[allow(clippy::too_many_arguments)]
fn pb(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    o: Option<Direction>,
    m: Mirror,
    r: Rotation,
    bx: &BoundingBox,
    state: &BlockState,
    x: i32,
    y: i32,
    z: i32,
) {
    place_block(level, bb, o, m, r, state.clone(), x, y, z, bx);
}

/// `generateWaterBox` — вода/воздух по уровню моря, не трогая
/// FILL_KEEP.
#[allow(clippy::too_many_arguments)]
fn gw(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    o: Option<Direction>,
    m: Mirror,
    r: Rotation,
    bx: &BoundingBox,
    x1: i32,
    y1: i32,
    z1: i32,
    x2: i32,
    y2: i32,
    z2: i32,
) {
    for i in y1..=y2 {
        for i1 in x1..=x2 {
            for i2 in z1..=z2 {
                let block = get_block(level, bb, o, i1, i, i2, bx);
                if !fill_keep(&block) {
                    if world_y(bb, i) >= level.sea_level()
                        && block.block.id != fill_block().block.id
                    {
                        pb(level, bb, o, m, r, bx, &BlockState::air(), i1, i, i2);
                    } else {
                        pb(level, bb, o, m, r, bx, fill_block(), i1, i, i2);
                    }
                }
            }
        }
    }
}

/// `generateDefaultFloor` — пол 8×8 с проёмом вниз.
#[allow(clippy::too_many_arguments)]
fn generate_default_floor(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    o: Option<Direction>,
    m: Mirror,
    r: Rotation,
    bx: &BoundingBox,
    x: i32,
    z: i32,
    has_opening_downwards: bool,
) {
    if has_opening_downwards {
        gb(level, bb, o, m, r, bx, x + 0, 0, z + 0, x + 2, 0, z + 8 - 1, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, x + 5, 0, z + 0, x + 8 - 1, 0, z + 8 - 1, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, x + 3, 0, z + 0, x + 4, 0, z + 2, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, x + 3, 0, z + 5, x + 4, 0, z + 8 - 1, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, x + 3, 0, z + 2, x + 4, 0, z + 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, x + 3, 0, z + 5, x + 4, 0, z + 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, x + 2, 0, z + 3, x + 2, 0, z + 4, base_light(), base_light());
        gb(level, bb, o, m, r, bx, x + 5, 0, z + 3, x + 5, 0, z + 4, base_light(), base_light());
    } else {
        gb(level, bb, o, m, r, bx, x + 0, 0, z + 0, x + 8 - 1, 0, z + 8 - 1, base_gray(), base_gray());
    }
}

/// `generateBoxOnFillOnly` — ставит state только в FILL_BLOCK.
#[allow(clippy::too_many_arguments)]
fn generate_box_on_fill_only(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    o: Option<Direction>,
    m: Mirror,
    r: Rotation,
    bx: &BoundingBox,
    min_x: i32,
    min_y: i32,
    min_z: i32,
    max_x: i32,
    max_y: i32,
    max_z: i32,
    state: &BlockState,
) {
    for i in min_y..=max_y {
        for i1 in min_x..=max_x {
            for i2 in min_z..=max_z {
                if get_block(level, bb, o, i1, i, i2, bx).block.id == fill_block().block.id {
                    pb(level, bb, o, m, r, bx, state, i1, i, i2);
                }
            }
        }
    }
}

/// `chunkIntersects` — 2D-пересечение куска с box по осям X/Z.
fn chunk_intersects(
    bb: &BoundingBox,
    o: Option<Direction>,
    bx: &BoundingBox,
    min_x: i32,
    min_z: i32,
    max_x: i32,
    max_z: i32,
) -> bool {
    let wx = world_x(bb, o, min_x, min_z);
    let wz = world_z(bb, o, min_x, min_z);
    let wx1 = world_x(bb, o, max_x, max_z);
    let wz1 = world_z(bb, o, max_x, max_z);
    bx.intersects_2d(
        wx.min(wx1),
        wz.min(wz1),
        wx.max(wx1),
        wz.max(wz1),
    )
}

/// `spawnElder` — elder guardian в центре блока.
fn spawn_elder(
    level: &dyn LevelReader,
    bb: &BoundingBox,
    o: Option<Direction>,
    bx: &BoundingBox,
    x: i32,
    y: i32,
    z: i32,
) {
    let wp = world_pos(bb, o, x, y, z);
    if bx.is_inside_pos(wp) {
        level.spawn_elder_guardian(wp);
    }
}

/// `getRoomIndex(x, y, z)` = y*25 + z*5 + x.
fn get_room_index(x: i32, y: i32, z: i32) -> i32 {
    y * 25 + z * 5 + x
}

const GRIDROOM_SOURCE_INDEX: i32 = 2; // getRoomIndex(2,0,0)
const GRIDROOM_TOP_CONNECT_INDEX: i32 = 52; // getRoomIndex(2,2,0)
const GRIDROOM_LEFTWING_CONNECT_INDEX: i32 = 25; // getRoomIndex(0,1,0)
const GRIDROOM_RIGHTWING_CONNECT_INDEX: i32 = 29; // getRoomIndex(4,1,0)

/// `makeBoundingBox(direction, roomDefinition, x, y, z)` —
/// локальный bounding box комнаты в мировых координатах
/// (комнаты строятся вокруг origin, потом сдвигаются).
fn room_bounding_box(
    direction: Direction,
    room_index: i32,
    x: i32,
    y: i32,
    z: i32,
) -> BoundingBox {
    let i1 = room_index % 5;
    let i2 = room_index / 5 % 5;
    let i3 = room_index / 25;
    let mut bb = make_bounding_box(0, 0, 0, direction, x * 8, y * 4, z * 8);
    match direction {
        Direction::NORTH => bb.move_(i1 * 8, i3 * 4, -(i2 + z) * 8 + 1),
        Direction::SOUTH => bb.move_(i1 * 8, i3 * 4, i2 * 8),
        Direction::WEST => bb.move_(-(i2 + z) * 8 + 1, i3 * 4, i1 * 8),
        _ => bb.move_(i2 * 8, i3 * 4, i1 * 8),
    }
    bb
}

// ------------------------------------------------------------------
// MonumentRoomFitter — fits/create для 7 фиттеров
// ------------------------------------------------------------------

type RoomRef = Rc<RefCell<RoomDefinition>>;

fn has_opening(room: &RoomRef, d: Direction) -> bool {
    room.borrow().has_opening[d.get_3d_data_value() as usize]
}

fn conn(room: &RoomRef, d: Direction) -> Option<RoomRef> {
    room.borrow().connections[d.get_3d_data_value() as usize].clone()
}

fn conn_claimed(room: &RoomRef, d: Direction) -> bool {
    conn(room, d).map(|c| c.borrow().claimed).unwrap_or(false)
}

fn claim(room: &RoomRef, d: Option<Direction>) {
    room.borrow_mut().claimed = true;
    if let Some(dir) = d {
        if let Some(c) = conn(room, dir) {
            c.borrow_mut().claimed = true;
        }
    }
}

fn fit_double_xy_fits(room: &RoomRef) -> bool {
    if has_opening(room, Direction::EAST)
        && !conn_claimed(room, Direction::EAST)
        && has_opening(room, Direction::UP)
        && !conn_claimed(room, Direction::UP)
    {
        if let Some(east) = conn(room, Direction::EAST) {
            return has_opening(&east, Direction::UP)
                && !conn_claimed(&east, Direction::UP);
        }
    }
    false
}

fn fit_double_yz_fits(room: &RoomRef) -> bool {
    if has_opening(room, Direction::NORTH)
        && !conn_claimed(room, Direction::NORTH)
        && has_opening(room, Direction::UP)
        && !conn_claimed(room, Direction::UP)
    {
        if let Some(north) = conn(room, Direction::NORTH) {
            return has_opening(&north, Direction::UP)
                && !conn_claimed(&north, Direction::UP);
        }
    }
    false
}

fn fit_double_z_fits(room: &RoomRef) -> bool {
    has_opening(room, Direction::NORTH) && !conn_claimed(room, Direction::NORTH)
}

fn fit_double_x_fits(room: &RoomRef) -> bool {
    has_opening(room, Direction::EAST) && !conn_claimed(room, Direction::EAST)
}

fn fit_double_y_fits(room: &RoomRef) -> bool {
    has_opening(room, Direction::UP) && !conn_claimed(room, Direction::UP)
}

fn fit_simple_top_fits(room: &RoomRef) -> bool {
    !has_opening(room, Direction::WEST)
        && !has_opening(room, Direction::EAST)
        && !has_opening(room, Direction::NORTH)
        && !has_opening(room, Direction::SOUTH)
        && !has_opening(room, Direction::UP)
}

fn fit_simple_fits(_room: &RoomRef) -> bool {
    true
}

// ------------------------------------------------------------------
// OceanMonumentPiece — единый тип куска (kind-дискриминант)
// ------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum MonumentKind {
    Building,
    CoreRoom,
    DoubleXRoom,
    DoubleXYRoom,
    DoubleYRoom,
    DoubleYZRoom,
    DoubleZRoom,
    EntryRoom,
    Penthouse,
    SimpleRoom,
    SimpleTopRoom,
    WingRoom,
}

impl MonumentKind {
    fn piece_id(self) -> &'static str {
        match self {
            MonumentKind::Building => "monument_building",
            MonumentKind::CoreRoom => "monument_core_room",
            MonumentKind::DoubleXRoom => "monument_double_x_room",
            MonumentKind::DoubleXYRoom => "monument_double_xy_room",
            MonumentKind::DoubleYRoom => "monument_double_y_room",
            MonumentKind::DoubleYZRoom => "monument_double_yz_room",
            MonumentKind::DoubleZRoom => "monument_double_z_room",
            MonumentKind::EntryRoom => "monument_entry_room",
            MonumentKind::Penthouse => "monument_penthouse",
            MonumentKind::SimpleRoom => "monument_simple_room",
            MonumentKind::SimpleTopRoom => "monument_simple_top_room",
            MonumentKind::WingRoom => "monument_wing_room",
        }
    }
}

pub struct OceanMonumentPiece {
    pub bounding_box: BoundingBox,
    gen_depth: i32,
    orientation: Option<Direction>,
    mirror: Mirror,
    rotation: Rotation,
    /// `roomDefinition` — Some для всех room-кусков.
    room_definition: Option<RoomRef>,
    /// `sourceRoom`/`coreRoom` — только у MonumentBuilding.
    source_room: Option<RoomRef>,
    core_room: Option<RoomRef>,
    /// `childPieces` — только у MonumentBuilding.
    child_pieces: Vec<Box<dyn StructurePiece>>,
    /// `mainDesign` — SimpleRoom (0..3) / WingRoom (flag & 1).
    main_design: i32,
    kind: MonumentKind,
}

impl OceanMonumentPiece {
    fn rd(&self) -> &RoomRef {
        self.room_definition.as_ref().unwrap()
    }

    fn rd_has(&self, d: Direction) -> bool {
        self.rd().borrow().has_opening[d.get_3d_data_value() as usize]
    }

    fn rd_conn(&self, d: Direction) -> Option<RoomRef> {
        self.rd()
            .borrow()
            .connections[d.get_3d_data_value() as usize]
            .clone()
    }

    fn rd_index(&self) -> i32 {
        self.rd().borrow().index
    }

    /// Конструктор room-куска: `OceanMonumentPiece(type, 1,
    /// direction, room, x, y, z)`.
    fn room_piece(
        kind: MonumentKind,
        direction: Direction,
        room: &RoomRef,
        x: i32,
        y: i32,
        z: i32,
    ) -> Self {
        let index = room.borrow().index;
        let bb = room_bounding_box(direction, index, x, y, z);
        let mut p = Self {
            bounding_box: bb,
            gen_depth: 1,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            room_definition: Some(room.clone()),
            source_room: None,
            core_room: None,
            child_pieces: Vec::new(),
            main_design: 0,
            kind,
        };
        p.set_orientation_impl(Some(direction));
        p
    }

    fn set_orientation_impl(&mut self, o: Option<Direction>) {
        piece_set_orientation(o, &mut self.mirror, &mut self.rotation);
        self.orientation = o;
    }

    // --------------------------------------------------------------
    // MonumentBuilding
    // --------------------------------------------------------------

    /// `MonumentBuilding(random, x, z, orientation)`.
    fn monument_building(
        random: &mut dyn RandomSource,
        x: i32,
        z: i32,
        orientation: Direction,
    ) -> Self {
        let bb = make_bounding_box(x, 39, z, orientation, 58, 23, 58);
        let mut p = Self {
            bounding_box: bb,
            gen_depth: 0,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            room_definition: None,
            source_room: None,
            core_room: None,
            child_pieces: Vec::new(),
            main_design: 0,
            kind: MonumentKind::Building,
        };
        p.set_orientation_impl(Some(orientation));

        let graph = Self::generate_room_graph(random);
        graph.source.borrow_mut().claimed = true;
        p.source_room = Some(graph.source.clone());
        p.core_room = Some(graph.core.clone());

        p.child_pieces.push(Box::new(Self::room_piece(
            MonumentKind::EntryRoom,
            orientation,
            &graph.source,
            1,
            1,
            1,
        )));
        p.child_pieces.push(Box::new(Self::room_piece(
            MonumentKind::CoreRoom,
            orientation,
            &graph.core,
            2,
            2,
            2,
        )));

        // Порядок фиттеров — как в Java.
        for room in &graph.rooms {
            if !room.borrow().claimed && !room.borrow().is_special() {
                let piece: Option<OceanMonumentPiece> = if fit_double_xy_fits(room) {
                    claim_multi_xy(room);
                    Some(Self::room_piece(MonumentKind::DoubleXYRoom, orientation, room, 2, 2, 1))
                } else if fit_double_yz_fits(room) {
                    claim_multi_yz(room);
                    Some(Self::room_piece(MonumentKind::DoubleYZRoom, orientation, room, 1, 2, 2))
                } else if fit_double_z_fits(room) {
                    let target = fit_double_z_target(room);
                    Some(Self::room_piece(MonumentKind::DoubleZRoom, orientation, &target, 1, 1, 2))
                } else if fit_double_x_fits(room) {
                    claim(room, Some(Direction::EAST));
                    Some(Self::room_piece(MonumentKind::DoubleXRoom, orientation, room, 2, 1, 1))
                } else if fit_double_y_fits(room) {
                    claim(room, Some(Direction::UP));
                    Some(Self::room_piece(MonumentKind::DoubleYRoom, orientation, room, 1, 2, 1))
                } else if fit_simple_top_fits(room) {
                    claim(room, None);
                    Some(Self::room_piece(MonumentKind::SimpleTopRoom, orientation, room, 1, 1, 1))
                } else {
                    // FitSimpleRoom — fits всегда true.
                    claim(room, None);
                    let mut piece =
                        Self::room_piece(MonumentKind::SimpleRoom, orientation, room, 1, 1, 1);
                    piece.main_design = random.next_int_bound(3);
                    Some(piece)
                };
                if let Some(pc) = piece {
                    p.child_pieces.push(Box::new(pc));
                }
            }
        }

        // Сдвиг всех комнатных кусков на worldPos(9,0,22).
        let wp = world_pos(&p.bounding_box, p.orientation, 9, 0, 22);
        for child in &mut p.child_pieces {
            if let Some(b) = child.bounding_box_mut() {
                b.move_(wp.get_x(), wp.get_y(), wp.get_z());
            }
        }

        // Крылья и пентхаус — уже в мировых координатах.
        let bb1 = corners_bb(
            world_pos(&p.bounding_box, p.orientation, 1, 1, 1),
            world_pos(&p.bounding_box, p.orientation, 23, 8, 21),
        );
        let bb2 = corners_bb(
            world_pos(&p.bounding_box, p.orientation, 34, 1, 1),
            world_pos(&p.bounding_box, p.orientation, 56, 8, 21),
        );
        let bb3 = corners_bb(
            world_pos(&p.bounding_box, p.orientation, 22, 13, 22),
            world_pos(&p.bounding_box, p.orientation, 35, 17, 35),
        );
        let mut random_int = random.next_int();
        p.child_pieces.push(Box::new(Self::wing_room(orientation, bb1, random_int)));
        random_int += 1;
        p.child_pieces.push(Box::new(Self::wing_room(orientation, bb2, random_int)));
        p.child_pieces.push(Box::new(Self::penthouse(orientation, bb3)));
        p
    }

    /// `OceanMonumentWingRoom(direction, box, flag)`.
    fn wing_room(direction: Direction, bb: BoundingBox, flag: i32) -> Self {
        let mut p = Self {
            bounding_box: bb,
            gen_depth: 1,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            room_definition: None,
            source_room: None,
            core_room: None,
            child_pieces: Vec::new(),
            main_design: flag & 1,
            kind: MonumentKind::WingRoom,
        };
        p.set_orientation_impl(Some(direction));
        p
    }

    /// `OceanMonumentPenthouse(direction, box)`.
    fn penthouse(direction: Direction, bb: BoundingBox) -> Self {
        let mut p = Self {
            bounding_box: bb,
            gen_depth: 1,
            orientation: None,
            mirror: Mirror::NONE,
            rotation: Rotation::NONE,
            room_definition: None,
            source_room: None,
            core_room: None,
            child_pieces: Vec::new(),
            main_design: 0,
            kind: MonumentKind::Penthouse,
        };
        p.set_orientation_impl(Some(direction));
        p
    }

    // --------------------------------------------------------------
    // generateRoomGraph
    // --------------------------------------------------------------

    /// `MonumentBuilding.generateRoomGraph` — возвращает
    /// (rooms, source, core).
    fn generate_room_graph(
        random: &mut dyn RandomSource,
    ) -> RoomGraph {
        let mut room_definitions: Vec<Option<RoomRef>> = (0..75).map(|_| None).collect();

        for i in 0..5 {
            for i1 in 0..4 {
                let room_index = get_room_index(i, 0, i1);
                room_definitions[room_index as usize] = Some(RoomDefinition::new(room_index));
            }
        }
        for i in 0..5 {
            for i1 in 0..4 {
                let room_index = get_room_index(i, 1, i1);
                room_definitions[room_index as usize] = Some(RoomDefinition::new(room_index));
            }
        }
        for i in 1..4 {
            for i1 in 0..2 {
                let room_index = get_room_index(i, 2, i1);
                room_definitions[room_index as usize] = Some(RoomDefinition::new(room_index));
            }
        }

        let source_room = room_definitions[GRIDROOM_SOURCE_INDEX as usize]
            .as_ref()
            .unwrap()
            .clone();

        for i in 0..5 {
            for i1 in 0..5 {
                for i2 in 0..3 {
                    let room_index = get_room_index(i, i2, i1);
                    if let Some(room) = &room_definitions[room_index as usize] {
                        for direction in Direction::VALUES {
                            let i3 = i + direction.get_step_x();
                            let i4 = i2 + direction.get_step_y();
                            let i5 = i1 + direction.get_step_z();
                            if i3 >= 0
                                && i3 < 5
                                && i5 >= 0
                                && i5 < 5
                                && i4 >= 0
                                && i4 < 3
                            {
                                let room_index1 = get_room_index(i3, i4, i5);
                                if let Some(neighbor) =
                                    &room_definitions[room_index1 as usize]
                                {
                                    if i5 == i1 {
                                        RoomDefinition::set_connection(
                                            room,
                                            direction,
                                            neighbor,
                                        );
                                    } else {
                                        RoomDefinition::set_connection(
                                            room,
                                            direction.get_opposite(),
                                            neighbor,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Специальные комнаты-крылья/пентхаус.
        let penthouse_room = RoomDefinition::new(1003);
        let left_wing = RoomDefinition::new(1001);
        let right_wing = RoomDefinition::new(1002);
        let top_connect = room_definitions[GRIDROOM_TOP_CONNECT_INDEX as usize]
            .as_ref()
            .unwrap()
            .clone();
        let left_connect = room_definitions[GRIDROOM_LEFTWING_CONNECT_INDEX as usize]
            .as_ref()
            .unwrap()
            .clone();
        let right_connect = room_definitions[GRIDROOM_RIGHTWING_CONNECT_INDEX as usize]
            .as_ref()
            .unwrap()
            .clone();
        RoomDefinition::set_connection(&top_connect, Direction::UP, &penthouse_room);
        RoomDefinition::set_connection(&left_connect, Direction::SOUTH, &left_wing);
        RoomDefinition::set_connection(&right_connect, Direction::SOUTH, &right_wing);
        penthouse_room.borrow_mut().claimed = true;
        left_wing.borrow_mut().claimed = true;
        right_wing.borrow_mut().claimed = true;
        source_room.borrow_mut().is_source = true;

        let core_room = room_definitions
            [get_room_index(random.next_int_bound(4), 0, 2) as usize]
            .as_ref()
            .unwrap()
            .clone();
        core_room.borrow_mut().claimed = true;
        for d in [
            Direction::EAST,
            Direction::NORTH,
            Direction::UP,
        ] {
            if let Some(c) = conn(&core_room, d) {
                c.borrow_mut().claimed = true;
            }
            // EAST+NORTH, EAST+UP, NORTH+UP, EAST+NORTH+UP —
            // помечаются ниже отдельными проходами.
        }
        // Точные claimed-флаги из Java.
        if let (Some(e), Some(n)) = (conn(&core_room, Direction::EAST), conn(&core_room, Direction::NORTH)) {
            if let Some(c) = conn(&e, Direction::NORTH) {
                c.borrow_mut().claimed = true;
            }
            if let Some(c) = conn(&e, Direction::UP) {
                c.borrow_mut().claimed = true;
            }
            if let Some(c) = conn(&n, Direction::UP) {
                c.borrow_mut().claimed = true;
            }
            let _ = n;
        }
        if let Some(e) = conn(&core_room, Direction::EAST) {
            if let Some(en) = conn(&e, Direction::NORTH) {
                if let Some(c) = conn(&en, Direction::UP) {
                    c.borrow_mut().claimed = true;
                }
            }
        }

        let mut list: Vec<RoomRef> = Vec::new();
        for room in room_definitions.iter().flatten() {
            room.borrow_mut().update_openings();
            list.push(room.clone());
        }
        penthouse_room.borrow_mut().update_openings();
        shuffle(&mut list, random);

        let mut i6 = 1;
        for room in &list {
            let mut i7 = 0;
            let mut i3 = 0;
            while i7 < 2 && i3 < 5 {
                i3 += 1;
                let i4 = random.next_int_bound(6);
                let has = room.borrow().has_opening[i4 as usize];
                if has {
                    let dir_opp =
                        Direction::from_3d_data_value(i4).get_opposite().get_3d_data_value() as usize;
                    let connection = room.borrow().connections[i4 as usize].clone();
                    room.borrow_mut().has_opening[i4 as usize] = false;
                    if let Some(c) = &connection {
                        c.borrow_mut().has_opening[dir_opp] = false;
                    }
                    let ok1 = room_find_source(room, i6);
                    i6 += 1;
                    let ok2 = if ok1 {
                        match &connection {
                            Some(c) => {
                                let v = room_find_source(c, i6);
                                i6 += 1;
                                v
                            }
                            None => false,
                        }
                    } else {
                        false
                    };
                    if ok1 && ok2 {
                        i7 += 1;
                    } else {
                        room.borrow_mut().has_opening[i4 as usize] = true;
                        if let Some(c) = &connection {
                            c.borrow_mut().has_opening[dir_opp] = true;
                        }
                    }
                }
            }
        }

        list.push(penthouse_room);
        list.push(left_wing);
        list.push(right_wing);
        RoomGraph {
            rooms: list,
            source: source_room,
            core: core_room,
        }
    }

    // --------------------------------------------------------------
    // postProcess — диспетчер по kind
    // --------------------------------------------------------------

    fn generate_monument(
        &mut self,
        level: &dyn LevelReader,
        _sm: &dyn StructureManagerAccess,
        _gen: Option<&dyn PoolGeneratorAccess>,
        random: &mut dyn RandomSource,
        bx: &BoundingBox,
        _cp: ChunkPos,
        _pos: BlockPos,
    ) {
        let bb = self.bounding_box.clone();
        let o = self.orientation;
        let m = self.mirror;
        let r = self.rotation;
        match self.kind {
            MonumentKind::Building => {}
            MonumentKind::CoreRoom => self.post_core_room(level, bx, &bb, o, m, r),
            MonumentKind::DoubleXRoom => self.post_double_x_room(level, bx, &bb, o, m, r),
            MonumentKind::DoubleXYRoom => self.post_double_xy_room(level, bx, &bb, o, m, r),
            MonumentKind::DoubleYRoom => self.post_double_y_room(level, bx, &bb, o, m, r),
            MonumentKind::DoubleYZRoom => self.post_double_yz_room(level, bx, &bb, o, m, r),
            MonumentKind::DoubleZRoom => self.post_double_z_room(level, bx, &bb, o, m, r),
            MonumentKind::EntryRoom => self.post_entry_room(level, bx, &bb, o, m, r),
            MonumentKind::Penthouse => self.post_penthouse(level, bx, &bb, o, m, r),
            MonumentKind::SimpleRoom => self.post_simple_room(level, bx, random, &bb, o, m, r),
            MonumentKind::SimpleTopRoom => self.post_simple_top_room(level, bx, random, &bb, o, m, r),
            MonumentKind::WingRoom => self.post_wing_room(level, bx, &bb, o, m, r),
        }
    }

    fn building_wing(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        wing: bool,
        x: i32,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if chunk_intersects(bb, o, bx, x, 0, x + 23, 20) {
            gb(level, bb, o, m, r, bx, x + 0, 0, 0, x + 24, 0, 20, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, x + 0, 1, 0, x + 24, 10, 20);
            for i1 in 0..4 {
                gb(level, bb, o, m, r, bx, x + i1, i1 + 1, i1, x + i1, i1 + 1, 20, base_light(), base_light());
                gb(level, bb, o, m, r, bx, x + i1 + 7, i1 + 5, i1 + 7, x + i1 + 7, i1 + 5, 20, base_light(), base_light());
                gb(level, bb, o, m, r, bx, x + 17 - i1, i1 + 5, i1 + 7, x + 17 - i1, i1 + 5, 20, base_light(), base_light());
                gb(level, bb, o, m, r, bx, x + 24 - i1, i1 + 1, i1, x + 24 - i1, i1 + 1, 20, base_light(), base_light());
                gb(level, bb, o, m, r, bx, x + i1 + 1, i1 + 1, i1, x + 23 - i1, i1 + 1, i1, base_light(), base_light());
                gb(level, bb, o, m, r, bx, x + i1 + 8, i1 + 5, i1 + 7, x + 16 - i1, i1 + 5, i1 + 7, base_light(), base_light());
            }
            gb(level, bb, o, m, r, bx, x + 4, 4, 4, x + 6, 4, 20, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, x + 7, 4, 4, x + 17, 4, 6, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, x + 18, 4, 4, x + 20, 4, 20, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, x + 11, 8, 11, x + 13, 8, 20, base_gray(), base_gray());
            pb(level, bb, o, m, r, bx, base_light(), x + 12, 9, 12);
            pb(level, bb, o, m, r, bx, base_light(), x + 12, 9, 15);
            pb(level, bb, o, m, r, bx, base_light(), x + 12, 9, 18);
            let i1 = x + if wing { 19 } else { 5 };
            let i2 = x + if wing { 5 } else { 19 };
            let mut i3 = 20;
            while i3 >= 5 {
                pb(level, bb, o, m, r, bx, base_light(), i1, 5, i3);
                i3 -= 3;
            }
            let mut i3 = 19;
            while i3 >= 7 {
                pb(level, bb, o, m, r, bx, base_light(), i2, 5, i3);
                i3 -= 3;
            }
            for i3 in 0..4 {
                let i4 = if wing { x + 24 - (17 - i3 * 3) } else { x + 17 - i3 * 3 };
                pb(level, bb, o, m, r, bx, base_light(), i4, 5, 5);
            }
            pb(level, bb, o, m, r, bx, base_light(), i2, 5, 5);
            gb(level, bb, o, m, r, bx, x + 11, 1, 12, x + 13, 7, 12, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, x + 12, 1, 11, x + 12, 7, 13, base_gray(), base_gray());
        }
    }

    fn building_entrance_archs(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if chunk_intersects(bb, o, bx, 22, 5, 35, 17) {
            gw(level, bb, o, m, r, bx, 25, 0, 0, 32, 8, 20);
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, 24, 2, 5 + i * 4, 24, 4, 5 + i * 4, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 22, 4, 5 + i * 4, 23, 4, 5 + i * 4, base_light(), base_light());
                pb(level, bb, o, m, r, bx, base_light(), 25, 5, 5 + i * 4);
                pb(level, bb, o, m, r, bx, base_light(), 26, 6, 5 + i * 4);
                pb(level, bb, o, m, r, bx, lamp_block(), 26, 5, 5 + i * 4);
                gb(level, bb, o, m, r, bx, 33, 2, 5 + i * 4, 33, 4, 5 + i * 4, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 34, 4, 5 + i * 4, 35, 4, 5 + i * 4, base_light(), base_light());
                pb(level, bb, o, m, r, bx, base_light(), 32, 5, 5 + i * 4);
                pb(level, bb, o, m, r, bx, base_light(), 31, 6, 5 + i * 4);
                pb(level, bb, o, m, r, bx, lamp_block(), 31, 5, 5 + i * 4);
                gb(level, bb, o, m, r, bx, 27, 6, 5 + i * 4, 30, 6, 5 + i * 4, base_gray(), base_gray());
            }
        }
    }

    fn building_entrance_wall(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if !chunk_intersects(bb, o, bx, 15, 20, 42, 21) {
            return;
        }
        gb(level, bb, o, m, r, bx, 15, 0, 21, 42, 0, 21, base_gray(), base_gray());
        gw(level, bb, o, m, r, bx, 26, 1, 21, 31, 3, 21);
        gb(level, bb, o, m, r, bx, 21, 12, 21, 36, 12, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 17, 11, 21, 40, 11, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 16, 10, 21, 41, 10, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 15, 7, 21, 42, 9, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 16, 6, 21, 41, 6, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 17, 5, 21, 40, 5, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 21, 4, 21, 36, 4, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 22, 3, 21, 26, 3, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 31, 3, 21, 35, 3, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 23, 2, 21, 25, 2, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 32, 2, 21, 34, 2, 21, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 28, 4, 20, 29, 4, 21, base_light(), base_light());
        pb(level, bb, o, m, r, bx, base_light(), 27, 3, 21);
        pb(level, bb, o, m, r, bx, base_light(), 30, 3, 21);
        pb(level, bb, o, m, r, bx, base_light(), 26, 2, 21);
        pb(level, bb, o, m, r, bx, base_light(), 31, 2, 21);
        pb(level, bb, o, m, r, bx, base_light(), 25, 1, 21);
        pb(level, bb, o, m, r, bx, base_light(), 32, 1, 21);
        for i in 0..7 {
            pb(level, bb, o, m, r, bx, base_black(), 28 - i, 6 + i, 21);
            pb(level, bb, o, m, r, bx, base_black(), 29 + i, 6 + i, 21);
        }
        for i in 0..4 {
            pb(level, bb, o, m, r, bx, base_black(), 28 - i, 9 + i, 21);
            pb(level, bb, o, m, r, bx, base_black(), 29 + i, 9 + i, 21);
        }
        pb(level, bb, o, m, r, bx, base_black(), 28, 12, 21);
        pb(level, bb, o, m, r, bx, base_black(), 29, 12, 21);
        for i in 0..3 {
            pb(level, bb, o, m, r, bx, base_black(), 22 - i * 2, 8, 21);
            pb(level, bb, o, m, r, bx, base_black(), 22 - i * 2, 9, 21);
            pb(level, bb, o, m, r, bx, base_black(), 35 + i * 2, 8, 21);
            pb(level, bb, o, m, r, bx, base_black(), 35 + i * 2, 9, 21);
        }
        gw(level, bb, o, m, r, bx, 15, 13, 21, 42, 15, 21);
        gw(level, bb, o, m, r, bx, 15, 1, 21, 15, 6, 21);
        gw(level, bb, o, m, r, bx, 16, 1, 21, 16, 5, 21);
        gw(level, bb, o, m, r, bx, 17, 1, 21, 20, 4, 21);
        gw(level, bb, o, m, r, bx, 21, 1, 21, 21, 3, 21);
        gw(level, bb, o, m, r, bx, 22, 1, 21, 22, 2, 21);
        gw(level, bb, o, m, r, bx, 23, 1, 21, 24, 1, 21);
        gw(level, bb, o, m, r, bx, 42, 1, 21, 42, 6, 21);
        gw(level, bb, o, m, r, bx, 41, 1, 21, 41, 5, 21);
        gw(level, bb, o, m, r, bx, 37, 1, 21, 40, 4, 21);
        gw(level, bb, o, m, r, bx, 36, 1, 21, 36, 3, 21);
        gw(level, bb, o, m, r, bx, 33, 1, 21, 34, 1, 21);
        gw(level, bb, o, m, r, bx, 35, 1, 21, 35, 2, 21);
    }

    fn building_roof_piece(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if !chunk_intersects(bb, o, bx, 21, 21, 36, 36) {
            return;
        }
        gb(level, bb, o, m, r, bx, 21, 0, 22, 36, 0, 36, base_gray(), base_gray());
        gw(level, bb, o, m, r, bx, 21, 1, 22, 36, 23, 36);
        for i in 0..4 {
            gb(level, bb, o, m, r, bx, 21 + i, 13 + i, 21 + i, 36 - i, 13 + i, 21 + i, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 21 + i, 13 + i, 36 - i, 36 - i, 13 + i, 36 - i, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 21 + i, 13 + i, 22 + i, 21 + i, 13 + i, 35 - i, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 36 - i, 13 + i, 22 + i, 36 - i, 13 + i, 35 - i, base_light(), base_light());
        }
        gb(level, bb, o, m, r, bx, 25, 16, 25, 32, 16, 32, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 25, 17, 25, 25, 19, 25, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 32, 17, 25, 32, 19, 25, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 25, 17, 32, 25, 19, 32, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 32, 17, 32, 32, 19, 32, base_light(), base_light());
        pb(level, bb, o, m, r, bx, base_light(), 26, 20, 26);
        pb(level, bb, o, m, r, bx, base_light(), 27, 21, 27);
        pb(level, bb, o, m, r, bx, lamp_block(), 27, 20, 27);
        pb(level, bb, o, m, r, bx, base_light(), 26, 20, 31);
        pb(level, bb, o, m, r, bx, base_light(), 27, 21, 30);
        pb(level, bb, o, m, r, bx, lamp_block(), 27, 20, 30);
        pb(level, bb, o, m, r, bx, base_light(), 31, 20, 31);
        pb(level, bb, o, m, r, bx, base_light(), 30, 21, 30);
        pb(level, bb, o, m, r, bx, lamp_block(), 30, 20, 30);
        pb(level, bb, o, m, r, bx, base_light(), 31, 20, 26);
        pb(level, bb, o, m, r, bx, base_light(), 30, 21, 27);
        pb(level, bb, o, m, r, bx, lamp_block(), 30, 20, 27);
        gb(level, bb, o, m, r, bx, 28, 21, 27, 29, 21, 27, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 27, 21, 28, 27, 21, 29, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 28, 21, 30, 29, 21, 30, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 30, 21, 28, 30, 21, 29, base_gray(), base_gray());
    }

    fn building_lower_wall(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if chunk_intersects(bb, o, bx, 0, 21, 6, 58) {
            gb(level, bb, o, m, r, bx, 0, 0, 21, 6, 0, 57, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 0, 1, 21, 6, 7, 57);
            gb(level, bb, o, m, r, bx, 4, 4, 21, 6, 4, 53, base_gray(), base_gray());
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, i, i + 1, 21, i, i + 1, 57 - i, base_light(), base_light());
            }
            let mut i = 23;
            while i < 53 {
                pb(level, bb, o, m, r, bx, base_light(), 5, 5, i);
                i += 3;
            }
            pb(level, bb, o, m, r, bx, base_light(), 5, 5, 52);
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, i, i + 1, 21, i, i + 1, 57 - i, base_light(), base_light());
            }
            gb(level, bb, o, m, r, bx, 4, 1, 52, 6, 3, 52, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 5, 1, 51, 5, 3, 53, base_gray(), base_gray());
        }
        if chunk_intersects(bb, o, bx, 51, 21, 58, 58) {
            gb(level, bb, o, m, r, bx, 51, 0, 21, 57, 0, 57, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 51, 1, 21, 57, 7, 57);
            gb(level, bb, o, m, r, bx, 51, 4, 21, 53, 4, 53, base_gray(), base_gray());
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, 57 - i, i + 1, 21, 57 - i, i + 1, 57 - i, base_light(), base_light());
            }
            let mut i = 23;
            while i < 53 {
                pb(level, bb, o, m, r, bx, base_light(), 52, 5, i);
                i += 3;
            }
            pb(level, bb, o, m, r, bx, base_light(), 52, 5, 52);
            gb(level, bb, o, m, r, bx, 51, 1, 52, 53, 3, 52, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 52, 1, 51, 52, 3, 53, base_gray(), base_gray());
        }
        if chunk_intersects(bb, o, bx, 0, 51, 57, 57) {
            gb(level, bb, o, m, r, bx, 7, 0, 51, 50, 0, 57, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 7, 1, 51, 50, 10, 57);
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, i + 1, i + 1, 57 - i, 56 - i, i + 1, 57 - i, base_light(), base_light());
            }
        }
    }

    fn building_middle_wall(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if chunk_intersects(bb, o, bx, 7, 21, 13, 50) {
            gb(level, bb, o, m, r, bx, 7, 0, 21, 13, 0, 50, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 7, 1, 21, 13, 10, 50);
            gb(level, bb, o, m, r, bx, 11, 8, 21, 13, 8, 53, base_gray(), base_gray());
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, i + 7, i + 5, 21, i + 7, i + 5, 54, base_light(), base_light());
            }
            let mut i = 21;
            while i <= 45 {
                pb(level, bb, o, m, r, bx, base_light(), 12, 9, i);
                i += 3;
            }
        }
        if chunk_intersects(bb, o, bx, 44, 21, 50, 54) {
            gb(level, bb, o, m, r, bx, 44, 0, 21, 50, 0, 50, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 44, 1, 21, 50, 10, 50);
            gb(level, bb, o, m, r, bx, 44, 8, 21, 46, 8, 53, base_gray(), base_gray());
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, 50 - i, i + 5, 21, 50 - i, i + 5, 54, base_light(), base_light());
            }
            let mut i = 21;
            while i <= 45 {
                pb(level, bb, o, m, r, bx, base_light(), 45, 9, i);
                i += 3;
            }
        }
        if chunk_intersects(bb, o, bx, 8, 44, 49, 54) {
            gb(level, bb, o, m, r, bx, 14, 0, 44, 43, 0, 50, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 14, 1, 44, 43, 10, 50);
            let mut i = 12;
            while i <= 45 {
                pb(level, bb, o, m, r, bx, base_light(), i, 9, 45);
                pb(level, bb, o, m, r, bx, base_light(), i, 9, 52);
                if i == 12 || i == 18 || i == 24 || i == 33 || i == 39 || i == 45 {
                    pb(level, bb, o, m, r, bx, base_light(), i, 9, 47);
                    pb(level, bb, o, m, r, bx, base_light(), i, 9, 50);
                    pb(level, bb, o, m, r, bx, base_light(), i, 10, 45);
                    pb(level, bb, o, m, r, bx, base_light(), i, 10, 46);
                    pb(level, bb, o, m, r, bx, base_light(), i, 10, 51);
                    pb(level, bb, o, m, r, bx, base_light(), i, 10, 52);
                    pb(level, bb, o, m, r, bx, base_light(), i, 11, 47);
                    pb(level, bb, o, m, r, bx, base_light(), i, 11, 50);
                    pb(level, bb, o, m, r, bx, base_light(), i, 12, 48);
                    pb(level, bb, o, m, r, bx, base_light(), i, 12, 49);
                }
                i += 3;
            }
            for ix in 0..3 {
                gb(level, bb, o, m, r, bx, 8 + ix, 5 + ix, 54, 49 - ix, 5 + ix, 54, base_gray(), base_gray());
            }
            gb(level, bb, o, m, r, bx, 11, 8, 54, 46, 8, 54, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 14, 8, 44, 43, 8, 53, base_gray(), base_gray());
        }
    }

    fn building_upper_wall(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if chunk_intersects(bb, o, bx, 14, 21, 20, 43) {
            gb(level, bb, o, m, r, bx, 14, 0, 21, 20, 0, 43, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 14, 1, 22, 20, 14, 43);
            gb(level, bb, o, m, r, bx, 18, 12, 22, 20, 12, 39, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 18, 12, 21, 20, 12, 21, base_light(), base_light());
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, i + 14, i + 9, 21, i + 14, i + 9, 43 - i, base_light(), base_light());
            }
            let mut i = 23;
            while i <= 39 {
                pb(level, bb, o, m, r, bx, base_light(), 19, 13, i);
                i += 3;
            }
        }
        if chunk_intersects(bb, o, bx, 37, 21, 43, 43) {
            gb(level, bb, o, m, r, bx, 37, 0, 21, 43, 0, 43, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 37, 1, 22, 43, 14, 43);
            gb(level, bb, o, m, r, bx, 37, 12, 22, 39, 12, 39, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 37, 12, 21, 39, 12, 21, base_light(), base_light());
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, 43 - i, i + 9, 21, 43 - i, i + 9, 43 - i, base_light(), base_light());
            }
            let mut i = 23;
            while i <= 39 {
                pb(level, bb, o, m, r, bx, base_light(), 38, 13, i);
                i += 3;
            }
        }
        if chunk_intersects(bb, o, bx, 15, 37, 42, 43) {
            gb(level, bb, o, m, r, bx, 21, 0, 37, 36, 0, 43, base_gray(), base_gray());
            gw(level, bb, o, m, r, bx, 21, 1, 37, 36, 14, 43);
            gb(level, bb, o, m, r, bx, 21, 12, 37, 36, 12, 39, base_gray(), base_gray());
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, 15 + i, i + 9, 43 - i, 42 - i, i + 9, 43 - i, base_light(), base_light());
            }
            let mut i = 21;
            while i <= 36 {
                pb(level, bb, o, m, r, bx, base_light(), i, 13, 38);
                i += 3;
            }
        }
    }

    // -- CoreRoom ---------------------------------------------------------

    fn post_core_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 8, 0, 14, 8, 14, base_gray());
        let mut i = 7;
        let mut block_state = base_light();
        gb(level, bb, o, m, r, bx, 0, 7, 0, 0, 7, 15, block_state, block_state);
        gb(level, bb, o, m, r, bx, 15, 7, 0, 15, 7, 15, block_state, block_state);
        gb(level, bb, o, m, r, bx, 1, 7, 0, 15, 7, 0, block_state, block_state);
        gb(level, bb, o, m, r, bx, 1, 7, 15, 14, 7, 15, block_state, block_state);
        for ix in 1..=6 {
            block_state = if ix == 2 || ix == 6 { base_gray() } else { base_light() };
            for i1 in [0, 15] {
                gb(level, bb, o, m, r, bx, i1, ix, 0, i1, ix, 1, block_state, block_state);
                gb(level, bb, o, m, r, bx, i1, ix, 6, i1, ix, 9, block_state, block_state);
                gb(level, bb, o, m, r, bx, i1, ix, 14, i1, ix, 15, block_state, block_state);
            }
            gb(level, bb, o, m, r, bx, 1, ix, 0, 1, ix, 0, block_state, block_state);
            gb(level, bb, o, m, r, bx, 6, ix, 0, 9, ix, 0, block_state, block_state);
            gb(level, bb, o, m, r, bx, 14, ix, 0, 14, ix, 0, block_state, block_state);
            gb(level, bb, o, m, r, bx, 1, ix, 15, 14, ix, 15, block_state, block_state);
        }
        gb(level, bb, o, m, r, bx, 6, 3, 6, 9, 6, 9, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 7, 4, 7, 8, 5, 8, gold_block(), gold_block());
        for ix in [3, 6] {
            for i2 in [6, 9] {
                pb(level, bb, o, m, r, bx, lamp_block(), i2, ix, 6);
                pb(level, bb, o, m, r, bx, lamp_block(), i2, ix, 9);
            }
        }
        let _ = i;
        gb(level, bb, o, m, r, bx, 5, 1, 6, 5, 2, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 9, 5, 2, 9, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 1, 6, 10, 2, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 1, 9, 10, 2, 9, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 1, 5, 6, 2, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 9, 1, 5, 9, 2, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 1, 10, 6, 2, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 9, 1, 10, 9, 2, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 2, 5, 5, 6, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 2, 10, 5, 6, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 2, 5, 10, 6, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 2, 10, 10, 6, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 7, 1, 5, 7, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 7, 1, 10, 7, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 7, 9, 5, 7, 14, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 7, 9, 10, 7, 14, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 7, 5, 6, 7, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 7, 10, 6, 7, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 9, 7, 5, 14, 7, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 9, 7, 10, 14, 7, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 1, 2, 2, 1, 3, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 3, 1, 2, 3, 1, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 13, 1, 2, 13, 1, 3, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 12, 1, 2, 12, 1, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 1, 12, 2, 1, 13, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 3, 1, 13, 3, 1, 13, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 13, 1, 12, 13, 1, 13, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 12, 1, 13, 12, 1, 13, base_light(), base_light());
    }

    // -- DoubleXRoom -------------------------------------------------------

    fn post_double_x_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        let room_east = self.rd_conn(Direction::EAST).unwrap();
        let room_self = self.rd().clone();
        if self.rd_index() / 25 > 0 {
            generate_default_floor(level, bb, o, m, r, bx, 8, 0, has_opening(&room_east, Direction::DOWN));
            generate_default_floor(level, bb, o, m, r, bx, 0, 0, has_opening(&room_self, Direction::DOWN));
        }
        if conn(&room_self, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 4, 1, 7, 4, 6, base_gray());
        }
        if conn(&room_east, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 8, 4, 1, 14, 4, 6, base_gray());
        }
        gb(level, bb, o, m, r, bx, 0, 3, 0, 0, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 15, 3, 0, 15, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 0, 15, 3, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 7, 14, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 2, 0, 0, 2, 7, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 15, 2, 0, 15, 2, 7, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 1, 2, 0, 15, 2, 0, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 1, 2, 7, 14, 2, 7, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 0, 1, 0, 0, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 15, 1, 0, 15, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 0, 15, 1, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 7, 14, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 0, 10, 1, 4, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 2, 0, 9, 2, 3, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 5, 3, 0, 10, 3, 4, base_light(), base_light());
        pb(level, bb, o, m, r, bx, lamp_block(), 6, 2, 3);
        pb(level, bb, o, m, r, bx, lamp_block(), 9, 2, 3);
        if has_opening(&room_self, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0);
        }
        if has_opening(&room_self, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 7, 4, 2, 7);
        }
        if has_opening(&room_self, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 3, 0, 2, 4);
        }
        if has_opening(&room_east, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 11, 1, 0, 12, 2, 0);
        }
        if has_opening(&room_east, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 11, 1, 7, 12, 2, 7);
        }
        if has_opening(&room_east, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 15, 1, 3, 15, 2, 4);
        }
    }

    // -- DoubleXYRoom ------------------------------------------------------

    fn post_double_xy_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        let room_east = self.rd_conn(Direction::EAST).unwrap();
        let room_self = self.rd().clone();
        let room_self_up = conn(&room_self, Direction::UP).unwrap();
        let room_east_up = conn(&room_east, Direction::UP).unwrap();
        if self.rd_index() / 25 > 0 {
            generate_default_floor(level, bb, o, m, r, bx, 8, 0, has_opening(&room_east, Direction::DOWN));
            generate_default_floor(level, bb, o, m, r, bx, 0, 0, has_opening(&room_self, Direction::DOWN));
        }
        if conn(&room_self_up, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 8, 1, 7, 8, 6, base_gray());
        }
        if conn(&room_east_up, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 8, 8, 1, 14, 8, 6, base_gray());
        }
        for i in 1..=7 {
            let block_state = if i == 2 || i == 6 { base_gray() } else { base_light() };
            gb(level, bb, o, m, r, bx, 0, i, 0, 0, i, 7, block_state, block_state);
            gb(level, bb, o, m, r, bx, 15, i, 0, 15, i, 7, block_state, block_state);
            gb(level, bb, o, m, r, bx, 1, i, 0, 15, i, 0, block_state, block_state);
            gb(level, bb, o, m, r, bx, 1, i, 7, 14, i, 7, block_state, block_state);
        }
        gb(level, bb, o, m, r, bx, 2, 1, 3, 2, 7, 4, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 3, 1, 2, 4, 7, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 3, 1, 5, 4, 7, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 13, 1, 3, 13, 7, 4, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 11, 1, 2, 12, 7, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 11, 1, 5, 12, 7, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 3, 5, 3, 4, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 1, 3, 10, 3, 4, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 7, 2, 10, 7, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 5, 2, 5, 7, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 5, 2, 10, 7, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 5, 5, 5, 7, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 10, 5, 5, 10, 7, 5, base_light(), base_light());
        pb(level, bb, o, m, r, bx, base_light(), 6, 6, 2);
        pb(level, bb, o, m, r, bx, base_light(), 9, 6, 2);
        pb(level, bb, o, m, r, bx, base_light(), 6, 6, 5);
        pb(level, bb, o, m, r, bx, base_light(), 9, 6, 5);
        gb(level, bb, o, m, r, bx, 5, 4, 3, 6, 4, 4, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 9, 4, 3, 10, 4, 4, base_light(), base_light());
        pb(level, bb, o, m, r, bx, lamp_block(), 5, 4, 2);
        pb(level, bb, o, m, r, bx, lamp_block(), 5, 4, 5);
        pb(level, bb, o, m, r, bx, lamp_block(), 10, 4, 2);
        pb(level, bb, o, m, r, bx, lamp_block(), 10, 4, 5);
        if has_opening(&room_self, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0);
        }
        if has_opening(&room_self, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 7, 4, 2, 7);
        }
        if has_opening(&room_self, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 3, 0, 2, 4);
        }
        if has_opening(&room_east, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 11, 1, 0, 12, 2, 0);
        }
        if has_opening(&room_east, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 11, 1, 7, 12, 2, 7);
        }
        if has_opening(&room_east, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 15, 1, 3, 15, 2, 4);
        }
        if has_opening(&room_self_up, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 5, 0, 4, 6, 0);
        }
        if has_opening(&room_self_up, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 5, 7, 4, 6, 7);
        }
        if has_opening(&room_self_up, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 5, 3, 0, 6, 4);
        }
        if has_opening(&room_east_up, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 11, 5, 0, 12, 6, 0);
        }
        if has_opening(&room_east_up, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 11, 5, 7, 12, 6, 7);
        }
        if has_opening(&room_east_up, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 15, 5, 3, 15, 6, 4);
        }
    }

    // -- DoubleYRoom --------------------------------------------------------

    fn post_double_y_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if self.rd_index() / 25 > 0 {
            generate_default_floor(level, bb, o, m, r, bx, 0, 0, self.rd_has(Direction::DOWN));
        }
        let room_up = self.rd_conn(Direction::UP).unwrap();
        if conn(&room_up, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 8, 1, 6, 8, 6, base_gray());
        }
        gb(level, bb, o, m, r, bx, 0, 4, 0, 0, 4, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 4, 0, 7, 4, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 4, 0, 6, 4, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 4, 7, 6, 4, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 4, 1, 2, 4, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 4, 2, 1, 4, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 4, 1, 5, 4, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 4, 2, 6, 4, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 4, 5, 2, 4, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 4, 5, 1, 4, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 4, 5, 5, 4, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 4, 5, 6, 4, 5, base_light(), base_light());
        let mut room_definition1 = self.rd().clone();
        for i in [1, 5] {
            if has_opening(&room_definition1, Direction::SOUTH) {
                gb(level, bb, o, m, r, bx, 2, i, 0, 2, i + 2, 0, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 5, i, 0, 5, i + 2, 0, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 3, i + 2, 0, 4, i + 2, 0, base_light(), base_light());
            } else {
                gb(level, bb, o, m, r, bx, 0, i, 0, 7, i + 2, 0, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 0, i + 1, 0, 7, i + 1, 0, base_gray(), base_gray());
            }
            if has_opening(&room_definition1, Direction::NORTH) {
                gb(level, bb, o, m, r, bx, 2, i, 7, 2, i + 2, 7, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 5, i, 7, 5, i + 2, 7, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 3, i + 2, 7, 4, i + 2, 7, base_light(), base_light());
            } else {
                gb(level, bb, o, m, r, bx, 0, i, 7, 7, i + 2, 7, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 0, i + 1, 7, 7, i + 1, 7, base_gray(), base_gray());
            }
            if has_opening(&room_definition1, Direction::WEST) {
                gb(level, bb, o, m, r, bx, 0, i, 2, 0, i + 2, 2, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 0, i, 5, 0, i + 2, 5, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 0, i + 2, 3, 0, i + 2, 4, base_light(), base_light());
            } else {
                gb(level, bb, o, m, r, bx, 0, i, 0, 0, i + 2, 7, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 0, i + 1, 0, 0, i + 1, 7, base_gray(), base_gray());
            }
            if has_opening(&room_definition1, Direction::EAST) {
                gb(level, bb, o, m, r, bx, 7, i, 2, 7, i + 2, 2, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 7, i, 5, 7, i + 2, 5, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 7, i + 2, 3, 7, i + 2, 4, base_light(), base_light());
            } else {
                gb(level, bb, o, m, r, bx, 7, i, 0, 7, i + 2, 7, base_light(), base_light());
                gb(level, bb, o, m, r, bx, 7, i + 1, 0, 7, i + 1, 7, base_gray(), base_gray());
            }
            room_definition1 = conn(&room_definition1, Direction::UP).unwrap_or_else(|| room_definition1.clone());
        }
    }

    // -- DoubleYZRoom -------------------------------------------------------

    fn post_double_yz_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        let room_north = self.rd_conn(Direction::NORTH).unwrap();
        let room_self = self.rd().clone();
        let room_north_up = conn(&room_north, Direction::UP).unwrap();
        let room_self_up = conn(&room_self, Direction::UP).unwrap();
        if self.rd_index() / 25 > 0 {
            generate_default_floor(level, bb, o, m, r, bx, 0, 8, has_opening(&room_north, Direction::DOWN));
            generate_default_floor(level, bb, o, m, r, bx, 0, 0, has_opening(&room_self, Direction::DOWN));
        }
        if conn(&room_self_up, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 8, 1, 6, 8, 7, base_gray());
        }
        if conn(&room_north_up, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 8, 8, 6, 8, 14, base_gray());
        }
        for i in 1..=7 {
            let block_state = if i == 2 || i == 6 { base_gray() } else { base_light() };
            gb(level, bb, o, m, r, bx, 0, i, 0, 0, i, 15, block_state, block_state);
            gb(level, bb, o, m, r, bx, 7, i, 0, 7, i, 15, block_state, block_state);
            gb(level, bb, o, m, r, bx, 1, i, 0, 6, i, 0, block_state, block_state);
            gb(level, bb, o, m, r, bx, 1, i, 15, 6, i, 15, block_state, block_state);
        }
        for i in 1..=7 {
            let block_state = if i == 2 || i == 6 { lamp_block() } else { base_black() };
            gb(level, bb, o, m, r, bx, 3, i, 7, 4, i, 8, block_state, block_state);
        }
        if has_opening(&room_self, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0);
        }
        if has_opening(&room_self, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 7, 1, 3, 7, 2, 4);
        }
        if has_opening(&room_self, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 3, 0, 2, 4);
        }
        if has_opening(&room_north, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 15, 4, 2, 15);
        }
        if has_opening(&room_north, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 11, 0, 2, 12);
        }
        if has_opening(&room_north, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 7, 1, 11, 7, 2, 12);
        }
        if has_opening(&room_self_up, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 5, 0, 4, 6, 0);
        }
        if has_opening(&room_self_up, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 7, 5, 3, 7, 6, 4);
            gb(level, bb, o, m, r, bx, 5, 4, 2, 6, 4, 5, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 6, 1, 2, 6, 3, 2, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 6, 1, 5, 6, 3, 5, base_light(), base_light());
        }
        if has_opening(&room_self_up, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 5, 3, 0, 6, 4);
            gb(level, bb, o, m, r, bx, 1, 4, 2, 2, 4, 5, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 1, 1, 2, 1, 3, 2, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 1, 1, 5, 1, 3, 5, base_light(), base_light());
        }
        if has_opening(&room_north_up, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 5, 15, 4, 6, 15);
        }
        if has_opening(&room_north_up, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 5, 11, 0, 6, 12);
            gb(level, bb, o, m, r, bx, 1, 4, 10, 2, 4, 13, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 1, 1, 10, 1, 3, 10, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 1, 1, 13, 1, 3, 13, base_light(), base_light());
        }
        if has_opening(&room_north_up, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 7, 5, 11, 7, 6, 12);
            gb(level, bb, o, m, r, bx, 5, 4, 10, 6, 4, 13, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 6, 1, 10, 6, 3, 10, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 6, 1, 13, 6, 3, 13, base_light(), base_light());
        }
    }

    // -- DoubleZRoom --------------------------------------------------------

    fn post_double_z_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        let room_north = self.rd_conn(Direction::NORTH).unwrap();
        let room_self = self.rd().clone();
        if self.rd_index() / 25 > 0 {
            generate_default_floor(level, bb, o, m, r, bx, 0, 8, has_opening(&room_north, Direction::DOWN));
            generate_default_floor(level, bb, o, m, r, bx, 0, 0, has_opening(&room_self, Direction::DOWN));
        }
        if conn(&room_self, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 4, 1, 6, 4, 7, base_gray());
        }
        if conn(&room_north, Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 4, 8, 6, 4, 14, base_gray());
        }
        gb(level, bb, o, m, r, bx, 0, 3, 0, 0, 3, 15, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 3, 0, 7, 3, 15, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 0, 7, 3, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 15, 6, 3, 15, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 2, 0, 0, 2, 15, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 7, 2, 0, 7, 2, 15, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 1, 2, 0, 7, 2, 0, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 1, 2, 15, 6, 2, 15, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 0, 1, 0, 0, 1, 15, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 1, 0, 7, 1, 15, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 0, 7, 1, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 15, 6, 1, 15, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 1, 1, 1, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 1, 1, 6, 1, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 1, 1, 3, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 3, 1, 6, 3, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 13, 1, 1, 14, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 1, 13, 6, 1, 14, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 13, 1, 3, 14, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 3, 13, 6, 3, 14, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 1, 6, 2, 3, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 6, 5, 3, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 1, 9, 2, 3, 9, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 9, 5, 3, 9, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 3, 2, 6, 4, 2, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 3, 2, 9, 4, 2, 9, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 2, 7, 2, 2, 8, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 2, 7, 5, 2, 8, base_light(), base_light());
        pb(level, bb, o, m, r, bx, lamp_block(), 2, 2, 5);
        pb(level, bb, o, m, r, bx, lamp_block(), 5, 2, 5);
        pb(level, bb, o, m, r, bx, lamp_block(), 2, 2, 10);
        pb(level, bb, o, m, r, bx, lamp_block(), 5, 2, 10);
        pb(level, bb, o, m, r, bx, base_light(), 2, 3, 5);
        pb(level, bb, o, m, r, bx, base_light(), 5, 3, 5);
        pb(level, bb, o, m, r, bx, base_light(), 2, 3, 10);
        pb(level, bb, o, m, r, bx, base_light(), 5, 3, 10);
        if has_opening(&room_self, Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0);
        }
        if has_opening(&room_self, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 7, 1, 3, 7, 2, 4);
        }
        if has_opening(&room_self, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 3, 0, 2, 4);
        }
        if has_opening(&room_north, Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 15, 4, 2, 15);
        }
        if has_opening(&room_north, Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 11, 0, 2, 12);
        }
        if has_opening(&room_north, Direction::EAST) {
            gw(level, bb, o, m, r, bx, 7, 1, 11, 7, 2, 12);
        }
    }

    // -- EntryRoom ----------------------------------------------------------

    fn post_entry_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        gb(level, bb, o, m, r, bx, 0, 3, 0, 2, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 3, 0, 7, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 2, 0, 1, 2, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 2, 0, 7, 2, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 1, 0, 0, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 1, 0, 7, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 1, 7, 7, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 0, 2, 3, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 0, 6, 3, 0, base_light(), base_light());
        if self.rd_has(Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 7, 4, 2, 7);
        }
        if self.rd_has(Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 3, 1, 2, 4);
        }
        if self.rd_has(Direction::EAST) {
            gw(level, bb, o, m, r, bx, 6, 1, 3, 7, 2, 4);
        }
    }

    // -- Penthouse ----------------------------------------------------------

    fn post_penthouse(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        gb(level, bb, o, m, r, bx, 2, -1, 2, 11, -1, 11, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, -1, 0, 1, -1, 11, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 12, -1, 0, 13, -1, 11, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 2, -1, 0, 11, -1, 1, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 2, -1, 12, 11, -1, 13, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 0, 0, 0, 0, 0, 13, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 13, 0, 0, 13, 0, 13, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 0, 0, 12, 0, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 0, 13, 12, 0, 13, base_light(), base_light());
        for i in [2, 5, 8, 11] {
            pb(level, bb, o, m, r, bx, lamp_block(), 0, 0, i);
            pb(level, bb, o, m, r, bx, lamp_block(), 13, 0, i);
            pb(level, bb, o, m, r, bx, lamp_block(), i, 0, 0);
        }
        gb(level, bb, o, m, r, bx, 2, 0, 3, 4, 0, 9, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 9, 0, 3, 11, 0, 9, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 4, 0, 9, 9, 0, 11, base_light(), base_light());
        pb(level, bb, o, m, r, bx, base_light(), 5, 0, 8);
        pb(level, bb, o, m, r, bx, base_light(), 8, 0, 8);
        pb(level, bb, o, m, r, bx, base_light(), 10, 0, 10);
        pb(level, bb, o, m, r, bx, base_light(), 3, 0, 10);
        gb(level, bb, o, m, r, bx, 3, 0, 3, 3, 0, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 10, 0, 3, 10, 0, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 6, 0, 10, 7, 0, 10, base_black(), base_black());
        let mut i = 3;
        for _ in 0..2 {
            for i2 in [2, 5, 8] {
                gb(level, bb, o, m, r, bx, i, 0, i2, i, 2, i2, base_light(), base_light());
            }
            i = 10;
        }
        gb(level, bb, o, m, r, bx, 5, 0, 10, 5, 2, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 8, 0, 10, 8, 2, 10, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, -1, 7, 7, -1, 8, base_black(), base_black());
        gw(level, bb, o, m, r, bx, 6, -1, 3, 7, -1, 4);
        spawn_elder(level, bb, o, bx, 6, 1, 6);
    }

    // -- SimpleRoom ---------------------------------------------------------

    fn post_simple_room(
        &mut self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        random: &mut dyn RandomSource,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if self.rd_index() / 25 > 0 {
            generate_default_floor(level, bb, o, m, r, bx, 0, 0, self.rd_has(Direction::DOWN));
        }
        if conn(self.rd(), Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 4, 1, 6, 4, 6, base_gray());
        }
        let flag = self.main_design != 0
            && random.next_boolean()
            && !self.rd_has(Direction::DOWN)
            && !self.rd_has(Direction::UP)
            && {
                let room = self.rd();
                room.borrow().count_openings() > 1
            };
        match self.main_design {
            0 => self.simple_design_0(level, bx, bb, o, m, r),
            1 => self.simple_design_1(level, bx, bb, o, m, r),
            _ => self.simple_design_2(level, bx, bb, o, m, r),
        }
        if flag {
            gb(level, bb, o, m, r, bx, 3, 1, 3, 4, 1, 4, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 3, 2, 3, 4, 2, 4, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 3, 3, 3, 4, 3, 4, base_light(), base_light());
        }
    }

    fn simple_design_0(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        gb(level, bb, o, m, r, bx, 0, 1, 0, 2, 1, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 3, 0, 2, 3, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 2, 0, 0, 2, 2, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 1, 2, 0, 2, 2, 0, base_gray(), base_gray());
        pb(level, bb, o, m, r, bx, lamp_block(), 1, 2, 1);
        gb(level, bb, o, m, r, bx, 5, 1, 0, 7, 1, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 3, 0, 7, 3, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 2, 0, 7, 2, 2, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 5, 2, 0, 6, 2, 0, base_gray(), base_gray());
        pb(level, bb, o, m, r, bx, lamp_block(), 6, 2, 1);
        gb(level, bb, o, m, r, bx, 0, 1, 5, 2, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 3, 5, 2, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 2, 5, 0, 2, 7, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 1, 2, 7, 2, 2, 7, base_gray(), base_gray());
        pb(level, bb, o, m, r, bx, lamp_block(), 1, 2, 6);
        gb(level, bb, o, m, r, bx, 5, 1, 5, 7, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 3, 5, 7, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 2, 5, 7, 2, 7, base_gray(), base_gray());
        gb(level, bb, o, m, r, bx, 5, 2, 7, 6, 2, 7, base_gray(), base_gray());
        pb(level, bb, o, m, r, bx, lamp_block(), 6, 2, 6);
        // Проёмы по сторонам.
        if self.rd_has(Direction::SOUTH) {
            gb(level, bb, o, m, r, bx, 3, 3, 0, 4, 3, 0, base_light(), base_light());
        } else {
            gb(level, bb, o, m, r, bx, 3, 3, 0, 4, 3, 1, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 3, 2, 0, 4, 2, 0, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 3, 1, 0, 4, 1, 1, base_light(), base_light());
        }
        if self.rd_has(Direction::NORTH) {
            gb(level, bb, o, m, r, bx, 3, 3, 7, 4, 3, 7, base_light(), base_light());
        } else {
            gb(level, bb, o, m, r, bx, 3, 3, 6, 4, 3, 7, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 3, 2, 7, 4, 2, 7, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 3, 1, 6, 4, 1, 7, base_light(), base_light());
        }
        if self.rd_has(Direction::WEST) {
            gb(level, bb, o, m, r, bx, 0, 3, 3, 0, 3, 4, base_light(), base_light());
        } else {
            gb(level, bb, o, m, r, bx, 0, 3, 3, 1, 3, 4, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 0, 2, 3, 0, 2, 4, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 0, 1, 3, 1, 1, 4, base_light(), base_light());
        }
        if self.rd_has(Direction::EAST) {
            gb(level, bb, o, m, r, bx, 7, 3, 3, 7, 3, 4, base_light(), base_light());
        } else {
            gb(level, bb, o, m, r, bx, 6, 3, 3, 7, 3, 4, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 7, 2, 3, 7, 2, 4, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 6, 1, 3, 7, 1, 4, base_light(), base_light());
        }
    }

    fn simple_design_1(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        gb(level, bb, o, m, r, bx, 2, 1, 2, 2, 3, 2, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 2, 1, 5, 2, 3, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 5, 5, 3, 5, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 5, 1, 2, 5, 3, 2, base_light(), base_light());
        pb(level, bb, o, m, r, bx, lamp_block(), 2, 2, 2);
        pb(level, bb, o, m, r, bx, lamp_block(), 2, 2, 5);
        pb(level, bb, o, m, r, bx, lamp_block(), 5, 2, 5);
        pb(level, bb, o, m, r, bx, lamp_block(), 5, 2, 2);
        gb(level, bb, o, m, r, bx, 0, 1, 0, 1, 3, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 1, 1, 0, 3, 1, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 1, 7, 1, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 1, 6, 0, 3, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 1, 7, 7, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 1, 6, 7, 3, 6, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 6, 1, 0, 7, 3, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 1, 1, 7, 3, 1, base_light(), base_light());
        pb(level, bb, o, m, r, bx, base_gray(), 1, 2, 0);
        pb(level, bb, o, m, r, bx, base_gray(), 0, 2, 1);
        pb(level, bb, o, m, r, bx, base_gray(), 1, 2, 7);
        pb(level, bb, o, m, r, bx, base_gray(), 0, 2, 6);
        pb(level, bb, o, m, r, bx, base_gray(), 6, 2, 7);
        pb(level, bb, o, m, r, bx, base_gray(), 7, 2, 6);
        pb(level, bb, o, m, r, bx, base_gray(), 6, 2, 0);
        pb(level, bb, o, m, r, bx, base_gray(), 7, 2, 1);
        if !self.rd_has(Direction::SOUTH) {
            gb(level, bb, o, m, r, bx, 1, 3, 0, 6, 3, 0, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 1, 2, 0, 6, 2, 0, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 1, 1, 0, 6, 1, 0, base_light(), base_light());
        }
        if !self.rd_has(Direction::NORTH) {
            gb(level, bb, o, m, r, bx, 1, 3, 7, 6, 3, 7, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 1, 2, 7, 6, 2, 7, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 1, 1, 7, 6, 1, 7, base_light(), base_light());
        }
        if !self.rd_has(Direction::WEST) {
            gb(level, bb, o, m, r, bx, 0, 3, 1, 0, 3, 6, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 0, 2, 1, 0, 2, 6, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 0, 1, 1, 0, 1, 6, base_light(), base_light());
        }
        if !self.rd_has(Direction::EAST) {
            gb(level, bb, o, m, r, bx, 7, 3, 1, 7, 3, 6, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 7, 2, 1, 7, 2, 6, base_gray(), base_gray());
            gb(level, bb, o, m, r, bx, 7, 1, 1, 7, 1, 6, base_light(), base_light());
        }
    }

    fn simple_design_2(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        gb(level, bb, o, m, r, bx, 0, 1, 0, 0, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 1, 0, 7, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 0, 6, 1, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 7, 6, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 2, 0, 0, 2, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 7, 2, 0, 7, 2, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 1, 2, 0, 6, 2, 0, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 1, 2, 7, 6, 2, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 0, 3, 0, 0, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 3, 0, 7, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 0, 6, 3, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 7, 6, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 1, 3, 0, 2, 4, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 7, 1, 3, 7, 2, 4, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 3, 1, 7, 4, 2, 7, base_black(), base_black());
        if self.rd_has(Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0);
        }
        if self.rd_has(Direction::NORTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 7, 4, 2, 7);
        }
        if self.rd_has(Direction::WEST) {
            gw(level, bb, o, m, r, bx, 0, 1, 3, 0, 2, 4);
        }
        if self.rd_has(Direction::EAST) {
            gw(level, bb, o, m, r, bx, 7, 1, 3, 7, 2, 4);
        }
    }

    // -- SimpleTopRoom ------------------------------------------------------

    fn post_simple_top_room(
        &mut self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        random: &mut dyn RandomSource,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if self.rd_index() / 25 > 0 {
            generate_default_floor(level, bb, o, m, r, bx, 0, 0, self.rd_has(Direction::DOWN));
        }
        if conn(self.rd(), Direction::UP).is_none() {
            generate_box_on_fill_only(level, bb, o, m, r, bx, 1, 4, 1, 6, 4, 6, base_gray());
        }
        for i in 1..=6 {
            for i1 in 1..=6 {
                if random.next_int_bound(3) != 0 {
                    let i2 = 2 + if random.next_int_bound(4) == 0 { 0 } else { 1 };
                    gb(level, bb, o, m, r, bx, i, i2, i1, i, 3, i1, wet_sponge(), wet_sponge());
                }
            }
        }
        gb(level, bb, o, m, r, bx, 0, 1, 0, 0, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 1, 0, 7, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 0, 6, 1, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 1, 7, 6, 1, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 2, 0, 0, 2, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 7, 2, 0, 7, 2, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 1, 2, 0, 6, 2, 0, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 1, 2, 7, 6, 2, 7, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 0, 3, 0, 0, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 7, 3, 0, 7, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 0, 6, 3, 0, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 1, 3, 7, 6, 3, 7, base_light(), base_light());
        gb(level, bb, o, m, r, bx, 0, 1, 3, 0, 2, 4, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 7, 1, 3, 7, 2, 4, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0, base_black(), base_black());
        gb(level, bb, o, m, r, bx, 3, 1, 7, 4, 2, 7, base_black(), base_black());
        if self.rd_has(Direction::SOUTH) {
            gw(level, bb, o, m, r, bx, 3, 1, 0, 4, 2, 0);
        }
    }

    // -- WingRoom -----------------------------------------------------------

    fn post_wing_room(
        &self,
        level: &dyn LevelReader,
        bx: &BoundingBox,
        bb: &BoundingBox,
        o: Option<Direction>,
        m: Mirror,
        r: Rotation,
    ) {
        if self.main_design == 0 {
            for i in 0..4 {
                gb(level, bb, o, m, r, bx, 10 - i, 3 - i, 20 - i, 12 + i, 3 - i, 20, base_light(), base_light());
            }
            gb(level, bb, o, m, r, bx, 7, 0, 6, 15, 0, 16, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 6, 0, 6, 6, 3, 20, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 16, 0, 6, 16, 3, 20, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 7, 1, 7, 7, 1, 20, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 15, 1, 7, 15, 1, 20, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 7, 1, 6, 9, 3, 6, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 13, 1, 6, 15, 3, 6, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 8, 1, 7, 9, 1, 7, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 13, 1, 7, 14, 1, 7, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 9, 0, 5, 13, 0, 5, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 10, 0, 7, 12, 0, 7, base_black(), base_black());
            gb(level, bb, o, m, r, bx, 8, 0, 10, 8, 0, 12, base_black(), base_black());
            gb(level, bb, o, m, r, bx, 14, 0, 10, 14, 0, 12, base_black(), base_black());
            let mut i = 18;
            while i >= 7 {
                pb(level, bb, o, m, r, bx, lamp_block(), 6, 3, i);
                pb(level, bb, o, m, r, bx, lamp_block(), 16, 3, i);
                i -= 3;
            }
            pb(level, bb, o, m, r, bx, lamp_block(), 10, 0, 10);
            pb(level, bb, o, m, r, bx, lamp_block(), 12, 0, 10);
            pb(level, bb, o, m, r, bx, lamp_block(), 10, 0, 12);
            pb(level, bb, o, m, r, bx, lamp_block(), 12, 0, 12);
            pb(level, bb, o, m, r, bx, lamp_block(), 8, 3, 6);
            pb(level, bb, o, m, r, bx, lamp_block(), 14, 3, 6);
            pb(level, bb, o, m, r, bx, base_light(), 4, 2, 4);
            pb(level, bb, o, m, r, bx, lamp_block(), 4, 1, 4);
            pb(level, bb, o, m, r, bx, base_light(), 4, 0, 4);
            pb(level, bb, o, m, r, bx, base_light(), 18, 2, 4);
            pb(level, bb, o, m, r, bx, lamp_block(), 18, 1, 4);
            pb(level, bb, o, m, r, bx, base_light(), 18, 0, 4);
            pb(level, bb, o, m, r, bx, base_light(), 4, 2, 18);
            pb(level, bb, o, m, r, bx, lamp_block(), 4, 1, 18);
            pb(level, bb, o, m, r, bx, base_light(), 4, 0, 18);
            pb(level, bb, o, m, r, bx, base_light(), 18, 2, 18);
            pb(level, bb, o, m, r, bx, lamp_block(), 18, 1, 18);
            pb(level, bb, o, m, r, bx, base_light(), 18, 0, 18);
            pb(level, bb, o, m, r, bx, base_light(), 9, 7, 20);
            pb(level, bb, o, m, r, bx, base_light(), 13, 7, 20);
            gb(level, bb, o, m, r, bx, 6, 0, 21, 7, 4, 21, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 15, 0, 21, 16, 4, 21, base_light(), base_light());
            spawn_elder(level, bb, o, bx, 11, 2, 16);
        } else if self.main_design == 1 {
            gb(level, bb, o, m, r, bx, 9, 3, 18, 13, 3, 20, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 9, 0, 18, 9, 2, 18, base_light(), base_light());
            gb(level, bb, o, m, r, bx, 13, 0, 18, 13, 2, 18, base_light(), base_light());
            let mut i = 9;
            for _ in 0..2 {
                pb(level, bb, o, m, r, bx, base_light(), i, 6, 20);
                pb(level, bb, o, m, r, bx, lamp_block(), i, 5, 20);
                pb(level, bb, o, m, r, bx, base_light(), i, 4, 20);
                i = 13;
            }
            gb(level, bb, o, m, r, bx, 7, 3, 7, 15, 3, 14, base_light(), base_light());
            let mut var14 = 10;
            for _ in 0..2 {
                gb(level, bb, o, m, r, bx, var14, 0, 10, var14, 6, 10, base_light(), base_light());
                gb(level, bb, o, m, r, bx, var14, 0, 12, var14, 6, 12, base_light(), base_light());
                pb(level, bb, o, m, r, bx, lamp_block(), var14, 0, 10);
                pb(level, bb, o, m, r, bx, lamp_block(), var14, 0, 12);
                pb(level, bb, o, m, r, bx, lamp_block(), var14, 4, 10);
                pb(level, bb, o, m, r, bx, lamp_block(), var14, 4, 12);
                var14 = 12;
            }
            var14 = 8;
            for _ in 0..2 {
                gb(level, bb, o, m, r, bx, var14, 0, 7, var14, 2, 7, base_light(), base_light());
                gb(level, bb, o, m, r, bx, var14, 0, 14, var14, 2, 14, base_light(), base_light());
                var14 = 14;
            }
            gb(level, bb, o, m, r, bx, 8, 3, 8, 8, 3, 13, base_black(), base_black());
            gb(level, bb, o, m, r, bx, 14, 3, 8, 14, 3, 13, base_black(), base_black());
            spawn_elder(level, bb, o, bx, 11, 5, 13);
        }
    }
}

/// Результат `generateRoomGraph`.
struct RoomGraph {
    rooms: Vec<RoomRef>,
    source: RoomRef,
    core: RoomRef,
}

// ------------------------------------------------------------------
// Фиттеры — claim-логика (порядок как в Java list1)
// ------------------------------------------------------------------

fn claim_multi_xy(room: &RoomRef) {
    // FitDoubleXYRoom.create
    room.borrow_mut().claimed = true;
    if let Some(e) = conn(room, Direction::EAST) {
        e.borrow_mut().claimed = true;
        if let Some(u) = conn(room, Direction::UP) {
            u.borrow_mut().claimed = true;
            if let Some(eu) = conn(&e, Direction::UP) {
                eu.borrow_mut().claimed = true;
            }
        }
    }
}

fn claim_multi_yz(room: &RoomRef) {
    // FitDoubleYZRoom.create
    room.borrow_mut().claimed = true;
    if let Some(n) = conn(room, Direction::NORTH) {
        n.borrow_mut().claimed = true;
        if let Some(u) = conn(room, Direction::UP) {
            u.borrow_mut().claimed = true;
            if let Some(nu) = conn(&n, Direction::UP) {
                nu.borrow_mut().claimed = true;
            }
        }
    }
}

/// FitDoubleZRoom.create — выбирает room или SOUTH-соседа.
fn fit_double_z_target(room: &RoomRef) -> RoomRef {
    let mut target = room.clone();
    if !has_opening(room, Direction::NORTH) || conn_claimed(room, Direction::NORTH) {
        if let Some(s) = conn(room, Direction::SOUTH) {
            target = s;
        }
    }
    target.borrow_mut().claimed = true;
    if let Some(n) = conn(&target, Direction::NORTH) {
        n.borrow_mut().claimed = true;
    }
    target
}

// ------------------------------------------------------------------
// StructurePiece impl
// ------------------------------------------------------------------

impl StructurePiece for OceanMonumentPiece {
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
        self.set_orientation_impl(orientation);
    }

    fn rotation(&self) -> Rotation {
        self.rotation
    }

    fn mirror(&self) -> Mirror {
        self.mirror
    }

    fn piece_type_id(&self) -> Identifier {
        Identifier::minecraft(self.kind.piece_id())
    }

    fn move_by(&mut self, x: i32, y: i32, z: i32) {
        self.bounding_box.move_(x, y, z);
        // Дочерние куски MonumentBuilding живут в тех же
        // координатах — сдвигаем вместе с родителем.
        for child in &mut self.child_pieces {
            if let Some(b) = child.bounding_box_mut() {
                b.move_(x, y, z);
            }
        }
    }

    fn add_additional_save_data(&self, _tag: &mut CompoundTag) {}

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
        chunk_pos: ChunkPos,
        pos: BlockPos,
    ) {
        if self.kind == MonumentKind::Building {
            // Сначала собственная геометрия, затем дети.
            let bb = self.bounding_box.clone();
            let o = self.orientation;
            let m = self.mirror;
            let r = self.rotation;
            let i = level.sea_level().max(64) - bb.min_y;
            gw(level, &bb, o, m, r, box_, 0, 0, 0, 58, i, 58);
            self.building_wing(level, box_, false, 0, &bb, o, m, r);
            self.building_wing(level, box_, true, 33, &bb, o, m, r);
            self.building_entrance_archs(level, box_, &bb, o, m, r);
            self.building_entrance_wall(level, box_, &bb, o, m, r);
            self.building_roof_piece(level, box_, &bb, o, m, r);
            self.building_lower_wall(level, box_, &bb, o, m, r);
            self.building_middle_wall(level, box_, &bb, o, m, r);
            self.building_upper_wall(level, box_, &bb, o, m, r);
            let mut i1 = 0;
            while i1 < 7 {
                let mut i2 = 0;
                while i2 < 7 {
                    if i2 == 0 && i1 == 3 {
                        i2 = 6;
                    }
                    let i3 = i1 * 9;
                    let i4 = i2 * 9;
                    for i5 in 0..4 {
                        for i6 in 0..4 {
                            pb(level, &bb, o, m, r, box_, base_light(), i3 + i5, 0, i4 + i6);
                            fill_column_down(level, &bb, o, m, r, box_, base_light(), i3 + i5, -1, i4 + i6);
                        }
                    }
                    if i1 != 0 && i1 != 6 {
                        i2 += 6;
                    } else {
                        i2 += 1;
                    }
                }
                i1 += 1;
            }
            for i1 in 0..5 {
                gw(level, &bb, o, m, r, box_, -1 - i1, 0 + i1 * 2, -1 - i1, -1 - i1, 23, 58 + i1);
                gw(level, &bb, o, m, r, box_, 58 + i1, 0 + i1 * 2, -1 - i1, 58 + i1, 23, 58 + i1);
                gw(level, &bb, o, m, r, box_, 0 - i1, 0 + i1 * 2, -1 - i1, 57 + i1, 23, -1 - i1);
                gw(level, &bb, o, m, r, box_, 0 - i1, 0 + i1 * 2, 58 + i1, 57 + i1, 23, 58 + i1);
            }
            let mut children = std::mem::take(&mut self.child_pieces);
            for child in children.iter_mut() {
                if child.bounding_box().intersects(box_) {
                    child.post_process(
                        level,
                        structure_manager,
                        generator,
                        random,
                        box_,
                        chunk_pos,
                        pos,
                    );
                }
            }
            self.child_pieces = children;
            return;
        }
        self.generate_monument(level, structure_manager, generator, random, box_, chunk_pos, pos);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

// ------------------------------------------------------------------
// OceanMonumentStructure
// ------------------------------------------------------------------

/// `ocean_monument_structure(settings)` — фабрика
/// `OceanMonumentStructure`.
pub fn ocean_monument_structure(settings: StructureSettings) -> Structure {
    let structure_type = register_structure_type(Identifier::minecraft("ocean_monument"));
    let mut structure = Structure::new(
        Identifier::minecraft("ocean_monument"),
        settings,
        structure_type,
    );
    structure.find_generation_point = Some(Arc::new(|context: &mut GenerationContext| {
        let chunk_pos = context.chunk_pos;
        // `chunkPos.getBlockX(9)` = minBlockX + 9*16.
        let block_x = chunk_pos.get_min_block_x() + 9 * 16;
        let block_z = chunk_pos.get_min_block_z() + 9 * 16;
        let sea_level = context.level.sea_level();
        for holder in context.level.biomes_within(block_x, sea_level, block_z, 29) {
            if !context.level.biome_is_ocean_monument_surrounding(&holder) {
                return None;
            }
        }
        Some(on_top_of_chunk_center(
            context,
            HeightmapType::OceanFloorWg,
            |pieces, ctx| {
                let direction =
                    super::super::structure_piece::get_random_horizontal_direction(&mut ctx.random);
                let i = ctx.chunk_pos.get_min_block_x() - 29;
                let i1 = ctx.chunk_pos.get_min_block_z() - 29;
                pieces.push(Box::new(OceanMonumentPiece::monument_building(
                    &mut ctx.random,
                    i,
                    i1,
                    direction,
                )));
            },
        ))
    }));
    structure
}

/// `OceanMonumentStructure.regeneratePiecesAfterLoad` —
/// пересоздание монумента после загрузки из сохранённых кусков.
pub fn ocean_monument_regenerate_pieces_after_load(
    chunk_pos: ChunkPos,
    seed: i64,
    pieces_container: PiecesContainer,
) -> PiecesContainer {
    if pieces_container.is_empty() {
        return pieces_container;
    }
    let mut worldgen_random = WorldgenRandom::new(Box::new(LegacyRandomSource::new(0)));
    worldgen_random.set_large_feature_seed(seed, chunk_pos.x, chunk_pos.z);
    let piece = &pieces_container.pieces()[0];
    let bounding_box = piece.bounding_box();
    let min_x = bounding_box.min_x;
    let min_z = bounding_box.min_z;
    let random_direction =
        super::super::structure_piece::get_random_horizontal_direction(&mut worldgen_random);
    let direction = piece.orientation().unwrap_or(random_direction);
    let building = OceanMonumentPiece::monument_building(
        &mut worldgen_random,
        min_x,
        min_z,
        direction,
    );
    PiecesContainer::new(vec![Box::new(building)])
}
