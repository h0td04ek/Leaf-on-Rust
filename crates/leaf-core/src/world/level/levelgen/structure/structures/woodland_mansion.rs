//! Перенос `structures/WoodlandMansionStructure.java` +
//! `structures/WoodlandMansionPieces.java` — лесной особняк:
//! `MansionGrid`/`SimpleGrid` (layout 11x11 с коридорами и
//! комнатами), `MansionPiecePlacer` (стены, крыши, комнаты
//! 1x1/1x2/2x2 + секретные) и `WoodlandMansionPiece`
//! (TemplateStructurePiece + data-маркеры сундуков/мобов).

use super::super::bounding_box::BoundingBox;
use super::super::pieces_container::PiecesContainer;
use super::super::structure::{
    get_lowest_y_in_5by5_box_offset_7_blocks, GenerationContext, GenerationStub,
    Structure, StructureSettings,
};
use super::super::structure_manager::StructureManagerAccess;
use super::super::structure_piece::{create_chest_at, StructurePiece};
use super::super::structure_type::register_structure_type;
use super::super::template_structure_piece::{DataMarkerHandler, TemplateStructurePiece};
use super::super::templatesystem::processors::BlockIgnoreProcessor;
use super::super::templatesystem::structure_place_settings::StructurePlaceSettings;
use super::super::templatesystem::structure_template::StructureTemplate;
use super::super::templatesystem::structure_template_manager::StructureTemplateManager;
use super::{block_state, with_string_property};
use crate::core::block_pos::BlockPos;
use crate::core::direction::{Direction, Plane};
use crate::random::RandomSource;
use crate::resources::identifier::Identifier;
use crate::world::level::block::mirror::Mirror;
use crate::world::level::block::rotation::Rotation;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::level::level_reader::LevelReader;
use std::rc::Rc;
use std::sync::Arc;

/// `BuiltInLootTables.WOODLAND_MANSION`.
const WOODLAND_MANSION_LOOT: &str = "minecraft:chests/woodland_mansion";

// `MansionGrid` константы (SimpleGrid-ячейки).
const CLEAR: i32 = 0;
const CORRIDOR: i32 = 1;
const ROOM: i32 = 2;
const START_ROOM: i32 = 3;
const TEST_ROOM: i32 = 4;
const BLOCKED: i32 = 5;
const ROOM_1X1: i32 = 65536;
const ROOM_1X2: i32 = 131072;
const ROOM_2X2: i32 = 262144;
const ROOM_ORIGIN_FLAG: i32 = 1048576;
const ROOM_DOOR_FLAG: i32 = 2097152;
const ROOM_STAIRS_FLAG: i32 = 4194304;
const ROOM_CORRIDOR_FLAG: i32 = 8388608;
const ROOM_TYPE_MASK: i32 = 983040;
const ROOM_ID_MASK: i32 = 65535;
const DEFAULT_SIZE: i32 = 11;

/// `SimpleGrid` — int[width][height] с valueIfOutside для OOB.
/// Java `new int[w][h]` — все ячейки 0 (CLEAR), `valueIfOutside`
/// применяется только к `get` за границами.
pub struct SimpleGrid {
    data: Vec<i32>,
    pub width: i32,
    pub height: i32,
    value_if_outside: i32,
}

impl SimpleGrid {
    pub fn new(width: i32, height: i32, value_if_outside: i32) -> Self {
        Self {
            data: vec![0; (width * height).max(0) as usize],
            width,
            height,
            value_if_outside,
        }
    }

    fn index(&self, x: i32, y: i32) -> usize {
        (x * self.height + y) as usize
    }

    pub fn set(&mut self, x: i32, y: i32, value: i32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            let i = self.index(x, y);
            self.data[i] = value;
        }
    }

    /// `set(minX, minY, maxX, maxY, value)` — Java порядок
    /// аргументов (x-диапазон, затем y-диапазон).
    pub fn set_range(&mut self, min_x: i32, min_y: i32, max_x: i32, max_y: i32, value: i32) {
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                self.set(x, y, value);
            }
        }
    }

    pub fn get(&self, x: i32, y: i32) -> i32 {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            self.data[self.index(x, y)]
        } else {
            self.value_if_outside
        }
    }

    pub fn setif(&mut self, x: i32, y: i32, old_value: i32, new_value: i32) {
        if self.get(x, y) == old_value {
            self.set(x, y, new_value);
        }
    }

    pub fn edges_to(&self, x: i32, y: i32, expected_value: i32) -> bool {
        self.get(x - 1, y) == expected_value
            || self.get(x + 1, y) == expected_value
            || self.get(x, y + 1) == expected_value
            || self.get(x, y - 1) == expected_value
    }
}

/// `FloorRoomCollection` — выбор имён шаблонов комнат.
pub trait FloorRoomCollection {
    fn get_1x1(&self, random: &mut dyn RandomSource) -> String;
    fn get_1x1_secret(&self, random: &mut dyn RandomSource) -> String;
    fn get_1x2_side_entrance(&self, random: &mut dyn RandomSource, is_stairs: bool) -> String;
    fn get_1x2_front_entrance(&self, random: &mut dyn RandomSource, is_stairs: bool) -> String;
    fn get_1x2_secret(&self, random: &mut dyn RandomSource) -> String;
    fn get_2x2(&self, random: &mut dyn RandomSource) -> String;
    fn get_2x2_secret(&self, random: &mut dyn RandomSource) -> String;
}

/// `FirstFloorRoomCollection`.
pub struct FirstFloorRoomCollection;
impl FloorRoomCollection for FirstFloorRoomCollection {
    fn get_1x1(&self, r: &mut dyn RandomSource) -> String {
        format!("1x1_a{}", r.next_int_bound(5) + 1)
    }
    fn get_1x1_secret(&self, r: &mut dyn RandomSource) -> String {
        format!("1x1_as{}", r.next_int_bound(4) + 1)
    }
    fn get_1x2_side_entrance(&self, r: &mut dyn RandomSource, _s: bool) -> String {
        format!("1x2_a{}", r.next_int_bound(9) + 1)
    }
    fn get_1x2_front_entrance(&self, r: &mut dyn RandomSource, _s: bool) -> String {
        format!("1x2_b{}", r.next_int_bound(5) + 1)
    }
    fn get_1x2_secret(&self, r: &mut dyn RandomSource) -> String {
        format!("1x2_s{}", r.next_int_bound(2) + 1)
    }
    fn get_2x2(&self, r: &mut dyn RandomSource) -> String {
        format!("2x2_a{}", r.next_int_bound(4) + 1)
    }
    fn get_2x2_secret(&self, _r: &mut dyn RandomSource) -> String {
        "2x2_s1".to_string()
    }
}

/// `SecondFloorRoomCollection`.
pub struct SecondFloorRoomCollection;
impl FloorRoomCollection for SecondFloorRoomCollection {
    fn get_1x1(&self, r: &mut dyn RandomSource) -> String {
        format!("1x1_b{}", r.next_int_bound(5) + 1)
    }
    fn get_1x1_secret(&self, r: &mut dyn RandomSource) -> String {
        format!("1x1_as{}", r.next_int_bound(4) + 1)
    }
    fn get_1x2_side_entrance(&self, r: &mut dyn RandomSource, is_stairs: bool) -> String {
        if is_stairs {
            "1x2_c_stairs".to_string()
        } else {
            format!("1x2_c{}", r.next_int_bound(4) + 1)
        }
    }
    fn get_1x2_front_entrance(&self, r: &mut dyn RandomSource, is_stairs: bool) -> String {
        if is_stairs {
            "1x2_d_stairs".to_string()
        } else {
            format!("1x2_d{}", r.next_int_bound(5) + 1)
        }
    }
    fn get_1x2_secret(&self, r: &mut dyn RandomSource) -> String {
        // Java: nextInt(1) — вызов сохраняем для RNG-последовательности.
        let _ = r.next_int_bound(1);
        "1x2_se1".to_string()
    }
    fn get_2x2(&self, r: &mut dyn RandomSource) -> String {
        format!("2x2_b{}", r.next_int_bound(5) + 1)
    }
    fn get_2x2_secret(&self, _r: &mut dyn RandomSource) -> String {
        "2x2_s1".to_string()
    }
}

/// `ThirdFloorRoomCollection extends SecondFloorRoomCollection` —
/// без переопределений, поведение идентично второму этажу.
pub struct ThirdFloorRoomCollection;
impl FloorRoomCollection for ThirdFloorRoomCollection {
    fn get_1x1(&self, r: &mut dyn RandomSource) -> String {
        SecondFloorRoomCollection.get_1x1(r)
    }
    fn get_1x1_secret(&self, r: &mut dyn RandomSource) -> String {
        SecondFloorRoomCollection.get_1x1_secret(r)
    }
    fn get_1x2_side_entrance(&self, r: &mut dyn RandomSource, s: bool) -> String {
        SecondFloorRoomCollection.get_1x2_side_entrance(r, s)
    }
    fn get_1x2_front_entrance(&self, r: &mut dyn RandomSource, s: bool) -> String {
        SecondFloorRoomCollection.get_1x2_front_entrance(r, s)
    }
    fn get_1x2_secret(&self, r: &mut dyn RandomSource) -> String {
        SecondFloorRoomCollection.get_1x2_secret(r)
    }
    fn get_2x2(&self, r: &mut dyn RandomSource) -> String {
        SecondFloorRoomCollection.get_2x2(r)
    }
    fn get_2x2_secret(&self, r: &mut dyn RandomSource) -> String {
        SecondFloorRoomCollection.get_2x2_secret(r)
    }
}

/// `MansionGrid` — layout особняка: baseGrid (1-2 этажи),
/// thirdFloorGrid, floorRooms[3] с room-id/флагами.
pub struct MansionGrid {
    pub base_grid: SimpleGrid,
    pub third_floor_grid: SimpleGrid,
    pub floor_rooms: [SimpleGrid; 3],
    pub entrance_x: i32,
    pub entrance_y: i32,
}

impl MansionGrid {
    pub fn new(random: &mut dyn RandomSource) -> Self {
        let entrance_x = 7;
        let entrance_y = 4;
        let mut base_grid = SimpleGrid::new(DEFAULT_SIZE, DEFAULT_SIZE, BLOCKED);
        base_grid.set_range(entrance_x, entrance_y, entrance_x + 1, entrance_y + 1, START_ROOM);
        base_grid.set_range(entrance_x - 1, entrance_y, entrance_x - 1, entrance_y + 1, ROOM);
        base_grid.set_range(entrance_x + 2, entrance_y - 2, entrance_x + 3, entrance_y + 3, BLOCKED);
        base_grid.set_range(entrance_x + 1, entrance_y - 2, entrance_x + 1, entrance_y - 1, CORRIDOR);
        base_grid.set_range(entrance_x + 1, entrance_y + 2, entrance_x + 1, entrance_y + 3, CORRIDOR);
        base_grid.set(entrance_x - 1, entrance_y - 1, CORRIDOR);
        base_grid.set(entrance_x - 1, entrance_y + 2, CORRIDOR);
        base_grid.set_range(0, 0, 11, 1, BLOCKED);
        base_grid.set_range(0, 9, 11, 11, BLOCKED);
        Self::recursive_corridor(random, &mut base_grid, entrance_x, entrance_y - 2, Direction::WEST, 6);
        Self::recursive_corridor(random, &mut base_grid, entrance_x, entrance_y + 3, Direction::WEST, 6);
        Self::recursive_corridor(random, &mut base_grid, entrance_x - 2, entrance_y - 1, Direction::WEST, 3);
        Self::recursive_corridor(random, &mut base_grid, entrance_x - 2, entrance_y + 2, Direction::WEST, 3);
        while Self::clean_edges(&mut base_grid) {}

        let [mut fr0, mut fr1, mut fr2]: [SimpleGrid; 3] = std::array::from_fn(|_| {
            SimpleGrid::new(DEFAULT_SIZE, DEFAULT_SIZE, BLOCKED)
        });
        Self::identify_rooms(random, &base_grid, &mut fr0);
        Self::identify_rooms(random, &base_grid, &mut fr1);
        fr0.set_range(entrance_x + 1, entrance_y, entrance_x + 1, entrance_y + 1, ROOM_CORRIDOR_FLAG);
        fr1.set_range(entrance_x + 1, entrance_y, entrance_x + 1, entrance_y + 1, ROOM_CORRIDOR_FLAG);
        let mut third_floor_grid = SimpleGrid::new(base_grid.width, base_grid.height, BLOCKED);
        Self::setup_third_floor(random, &base_grid, &mut fr1, &mut fr2, &mut third_floor_grid);
        Self::identify_rooms(random, &third_floor_grid, &mut fr2);

        Self {
            base_grid,
            third_floor_grid,
            floor_rooms: [fr0, fr1, fr2],
            entrance_x,
            entrance_y,
        }
    }

    /// `MansionGrid.isHouse` — статический, layout независим.
    pub fn is_house(layout: &SimpleGrid, x: i32, y: i32) -> bool {
        matches!(
            layout.get(x, y),
            CORRIDOR | ROOM | START_ROOM | TEST_ROOM
        )
    }

    /// `isRoomId` — Java игнорирует `layout` (использует только
    /// floorRooms[floor]); портировано как есть.
    pub fn is_room_id(&self, x: i32, y: i32, floor: usize, room_id: i32) -> bool {
        (self.floor_rooms[floor].get(x, y) & ROOM_ID_MASK) == room_id
    }

    /// `get1x2RoomDirection` — направление второй клетки 1x2
    /// комнаты (по room-id); `None` = Java null.
    pub fn get_1x2_room_direction(
        &self,
        x: i32,
        y: i32,
        floor: usize,
        room_id: i32,
    ) -> Option<Direction> {
        for direction in Plane::HORIZONTAL.faces() {
            if (self.floor_rooms[floor].get(
                x + direction.get_step_x(),
                y + direction.get_step_z(),
            ) & ROOM_ID_MASK)
                == room_id
            {
                return Some(*direction);
            }
        }
        None
    }

    /// Версия для `setupThirdFloor`, где self ещё не собран —
    /// работает напрямую по floor_rooms[1].
    fn get_1x2_room_direction_grid(floor_grid: &SimpleGrid, x: i32, y: i32, room_id: i32) -> Option<Direction> {
        for direction in Plane::HORIZONTAL.faces() {
            if (floor_grid.get(
                x + direction.get_step_x(),
                y + direction.get_step_z(),
            ) & ROOM_ID_MASK)
                == room_id
            {
                return Some(*direction);
            }
        }
        None
    }

    fn recursive_corridor(
        random: &mut dyn RandomSource,
        layout: &mut SimpleGrid,
        x: i32,
        y: i32,
        direction: Direction,
        length: i32,
    ) {
        if length <= 0 {
            return;
        }
        layout.set(x, y, CORRIDOR);
        layout.setif(
            x + direction.get_step_x(),
            y + direction.get_step_z(),
            CLEAR,
            CORRIDOR,
        );

        for _ in 0..8 {
            let direction1 = Direction::from_2d_data_value(random.next_int_bound(4));
            if direction1 != direction.get_opposite()
                && (direction1 != Direction::EAST || !random.next_boolean())
            {
                let i1 = x + direction.get_step_x();
                let i2 = y + direction.get_step_z();
                if layout.get(i1 + direction1.get_step_x(), i2 + direction1.get_step_z()) == CLEAR
                    && layout.get(
                        i1 + direction1.get_step_x() * 2,
                        i2 + direction1.get_step_z() * 2,
                    ) == CLEAR
                {
                    Self::recursive_corridor(
                        random,
                        layout,
                        x + direction.get_step_x() + direction1.get_step_x(),
                        y + direction.get_step_z() + direction1.get_step_z(),
                        direction1,
                        length - 1,
                    );
                    break;
                }
            }
        }

        let clock_wise = direction.get_clock_wise();
        let counter = direction.get_counter_clock_wise();
        layout.setif(x + clock_wise.get_step_x(), y + clock_wise.get_step_z(), CLEAR, ROOM);
        layout.setif(x + counter.get_step_x(), y + counter.get_step_z(), CLEAR, ROOM);
        layout.setif(
            x + direction.get_step_x() + clock_wise.get_step_x(),
            y + direction.get_step_z() + clock_wise.get_step_z(),
            CLEAR,
            ROOM,
        );
        layout.setif(
            x + direction.get_step_x() + counter.get_step_x(),
            y + direction.get_step_z() + counter.get_step_z(),
            CLEAR,
            ROOM,
        );
        layout.setif(x + direction.get_step_x() * 2, y + direction.get_step_z() * 2, CLEAR, ROOM);
        layout.setif(x + clock_wise.get_step_x() * 2, y + clock_wise.get_step_z() * 2, CLEAR, ROOM);
        layout.setif(x + counter.get_step_x() * 2, y + counter.get_step_z() * 2, CLEAR, ROOM);
    }

    fn clean_edges(grid: &mut SimpleGrid) -> bool {
        let mut flag = false;
        for i in 0..grid.height {
            for i1 in 0..grid.width {
                if grid.get(i1, i) == CLEAR {
                    let mut i2 = 0;
                    i2 += if Self::is_house(grid, i1 + 1, i) { 1 } else { 0 };
                    i2 += if Self::is_house(grid, i1 - 1, i) { 1 } else { 0 };
                    i2 += if Self::is_house(grid, i1, i + 1) { 1 } else { 0 };
                    i2 += if Self::is_house(grid, i1, i - 1) { 1 } else { 0 };
                    if i2 >= 3 {
                        grid.set(i1, i, ROOM);
                        flag = true;
                    } else if i2 == 2 {
                        let mut i3 = 0;
                        i3 += if Self::is_house(grid, i1 + 1, i + 1) { 1 } else { 0 };
                        i3 += if Self::is_house(grid, i1 - 1, i + 1) { 1 } else { 0 };
                        i3 += if Self::is_house(grid, i1 + 1, i - 1) { 1 } else { 0 };
                        i3 += if Self::is_house(grid, i1 - 1, i - 1) { 1 } else { 0 };
                        if i3 <= 1 {
                            grid.set(i1, i, ROOM);
                            flag = true;
                        }
                    }
                }
            }
        }
        flag
    }

    #[allow(clippy::too_many_arguments)]
    fn setup_third_floor(
        random: &mut dyn RandomSource,
        base_grid: &SimpleGrid,
        floor1: &mut SimpleGrid,
        floor2: &mut SimpleGrid,
        third: &mut SimpleGrid,
    ) {
        let mut list: Vec<(i32, i32)> = Vec::new();
        for i in 0..third.height {
            for i1 in 0..third.width {
                let i2 = floor1.get(i1, i);
                let i3 = i2 & ROOM_TYPE_MASK;
                if i3 == ROOM_1X2 && (i2 & ROOM_DOOR_FLAG) == ROOM_DOOR_FLAG {
                    list.push((i1, i));
                }
            }
        }

        if list.is_empty() {
            third.set_range(0, 0, third.width, third.height, BLOCKED);
        } else {
            let tuple = list[random.next_int_bound(list.len() as i32) as usize];
            let i1x = floor1.get(tuple.0, tuple.1);
            floor1.set(tuple.0, tuple.1, i1x | ROOM_STAIRS_FLAG);
            // Java: get1x2RoomDirection может вернуть null → NPE;
            // в валидном layout всегда находится (1x2-комната).
            let direction = Self::get_1x2_room_direction_grid(floor1, tuple.0, tuple.1, i1x & ROOM_ID_MASK)
                .unwrap_or(Direction::NORTH);
            let i3 = tuple.0 + direction.get_step_x();
            let i4 = tuple.1 + direction.get_step_z();

            for i5 in 0..third.height {
                for i6 in 0..third.width {
                    if !Self::is_house(base_grid, i6, i5) {
                        third.set(i6, i5, BLOCKED);
                    } else if i6 == tuple.0 && i5 == tuple.1 {
                        third.set(i6, i5, START_ROOM);
                    } else if i6 == i3 && i5 == i4 {
                        third.set(i6, i5, START_ROOM);
                        floor2.set(i6, i5, ROOM_CORRIDOR_FLAG);
                    }
                }
            }

            let mut list1: Vec<Direction> = Vec::new();
            for direction1 in Plane::HORIZONTAL.faces() {
                if third.get(i3 + direction1.get_step_x(), i4 + direction1.get_step_z()) == CLEAR {
                    list1.push(*direction1);
                }
            }

            if list1.is_empty() {
                third.set_range(0, 0, third.width, third.height, BLOCKED);
                floor1.set(tuple.0, tuple.1, i1x);
            } else {
                let direction2 = list1[random.next_int_bound(list1.len() as i32) as usize];
                Self::recursive_corridor(
                    random,
                    third,
                    i3 + direction2.get_step_x(),
                    i4 + direction2.get_step_z(),
                    direction2,
                    4,
                );
                while Self::clean_edges(third) {}
            }
        }
    }

    fn identify_rooms(random: &mut dyn RandomSource, grid: &SimpleGrid, floor_rooms: &mut SimpleGrid) {
        let mut list: Vec<(i32, i32)> = Vec::new();
        for i in 0..grid.height {
            for i1 in 0..grid.width {
                if grid.get(i1, i) == ROOM {
                    list.push((i1, i));
                }
            }
        }

        // `Util.shuffle(list, random)` — Fisher-Yates как в Java.
        let mut i = list.len() as i32;
        while i > 1 {
            let j = random.next_int_bound(i) as usize;
            list.swap((i - 1) as usize, j);
            i -= 1;
        }

        let mut room_id = 10;
        for (a, b) in list {
            if floor_rooms.get(a, b) == 0 {
                let (mut i2, mut i3, mut i4, mut i5) = (a, a, b, b);
                let mut i6 = ROOM_1X1;
                if floor_rooms.get(a + 1, b) == 0
                    && floor_rooms.get(a, b + 1) == 0
                    && floor_rooms.get(a + 1, b + 1) == 0
                    && grid.get(a + 1, b) == ROOM
                    && grid.get(a, b + 1) == ROOM
                    && grid.get(a + 1, b + 1) == ROOM
                {
                    i3 = a + 1;
                    i5 = b + 1;
                    i6 = ROOM_2X2;
                } else if floor_rooms.get(a - 1, b) == 0
                    && floor_rooms.get(a, b + 1) == 0
                    && floor_rooms.get(a - 1, b + 1) == 0
                    && grid.get(a - 1, b) == ROOM
                    && grid.get(a, b + 1) == ROOM
                    && grid.get(a - 1, b + 1) == ROOM
                {
                    i2 = a - 1;
                    i5 = b + 1;
                    i6 = ROOM_2X2;
                } else if floor_rooms.get(a - 1, b) == 0
                    && floor_rooms.get(a, b - 1) == 0
                    && floor_rooms.get(a - 1, b - 1) == 0
                    && grid.get(a - 1, b) == ROOM
                    && grid.get(a, b - 1) == ROOM
                    && grid.get(a - 1, b - 1) == ROOM
                {
                    i2 = a - 1;
                    i4 = b - 1;
                    i6 = ROOM_2X2;
                } else if floor_rooms.get(a + 1, b) == 0 && grid.get(a + 1, b) == ROOM {
                    i3 = a + 1;
                    i6 = ROOM_1X2;
                } else if floor_rooms.get(a, b + 1) == 0 && grid.get(a, b + 1) == ROOM {
                    i5 = b + 1;
                    i6 = ROOM_1X2;
                } else if floor_rooms.get(a - 1, b) == 0 && grid.get(a - 1, b) == ROOM {
                    i2 = a - 1;
                    i6 = ROOM_1X2;
                } else if floor_rooms.get(a, b - 1) == 0 && grid.get(a, b - 1) == ROOM {
                    i4 = b - 1;
                    i6 = ROOM_1X2;
                }

                let mut i7 = if random.next_boolean() { i2 } else { i3 };
                let mut i8 = if random.next_boolean() { i4 } else { i5 };
                let mut i9 = ROOM_DOOR_FLAG;
                if !grid.edges_to(i7, i8, CORRIDOR) {
                    i7 = if i7 == i2 { i3 } else { i2 };
                    i8 = if i8 == i4 { i5 } else { i4 };
                    if !grid.edges_to(i7, i8, CORRIDOR) {
                        i8 = if i8 == i4 { i5 } else { i4 };
                        if !grid.edges_to(i7, i8, CORRIDOR) {
                            i7 = if i7 == i2 { i3 } else { i2 };
                            i8 = if i8 == i4 { i5 } else { i4 };
                            if !grid.edges_to(i7, i8, CORRIDOR) {
                                i9 = 0;
                                i7 = i2;
                                i8 = i4;
                            }
                        }
                    }
                }

                for i10 in i4..=i5 {
                    for i11 in i2..=i3 {
                        if i11 == i7 && i10 == i8 {
                            floor_rooms.set(i11, i10, ROOM_ORIGIN_FLAG | i9 | i6 | room_id);
                        } else {
                            floor_rooms.set(i11, i10, i6 | room_id);
                        }
                    }
                }
                room_id += 1;
            }
        }
    }
}

/// `PlacementData` — текущее состояние обхода стен.
struct PlacementData {
    rotation: Rotation,
    position: BlockPos,
    wall_type: &'static str,
}

/// `MansionPiecePlacer`.
pub struct MansionPiecePlacer<'a> {
    manager: &'a StructureTemplateManager,
    random: &'a mut dyn RandomSource,
    start_x: i32,
    start_y: i32,
}

impl<'a> MansionPiecePlacer<'a> {
    pub fn new(manager: &'a StructureTemplateManager, random: &'a mut dyn RandomSource) -> Self {
        Self {
            manager,
            random,
            start_x: 0,
            start_y: 0,
        }
    }

    fn add_piece(
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        manager: &StructureTemplateManager,
        name: &str,
        pos: BlockPos,
        rotation: Rotation,
        mirror: Mirror,
    ) {
        let mut settings = StructurePlaceSettings::default();
        settings
            .set_ignore_entities(true)
            .set_rotation(rotation)
            .set_mirror(mirror)
            .add_processor(BlockIgnoreProcessor::structure_block());
        let location =
            Identifier::minecraft(&format!("woodland_mansion/{name}"));
        let mut piece = TemplateStructurePiece::new(
            Identifier::minecraft("woodland_mansion_piece"),
            0,
            manager,
            &location,
            name.to_string(),
            settings,
            pos,
        );
        piece.data_marker_handler = Some(mansion_data_marker_handler(rotation));
        pieces.push(Box::new(piece));
    }

    pub fn create_mansion(
        &mut self,
        pos: BlockPos,
        rotation: Rotation,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        grid: &MansionGrid,
    ) {
        let mut data = PlacementData {
            rotation,
            position: pos,
            wall_type: "wall_flat",
        };
        let mut data1 = PlacementData {
            rotation,
            position: pos.above_n(8),
            wall_type: "wall_window",
        };
        self.entrance(pieces, &mut data);
        data1.position = data.position.above_n(8);
        data1.rotation = data.rotation;
        data1.wall_type = "wall_window";

        let simple_grid = &grid.base_grid;
        let simple_grid1 = &grid.third_floor_grid;
        self.start_x = grid.entrance_x + 1;
        self.start_y = grid.entrance_y + 1;
        let i = grid.entrance_x + 1;
        let i1 = grid.entrance_y;
        self.traverse_outer_walls(
            pieces, &mut data, simple_grid, Direction::SOUTH, self.start_x, self.start_y, i, i1,
        );
        self.traverse_outer_walls(
            pieces, &mut data1, simple_grid, Direction::SOUTH, self.start_x, self.start_y, i, i1,
        );
        let mut data2 = PlacementData {
            rotation,
            position: pos.above_n(19),
            wall_type: "wall_window",
        };
        let mut flag = false;

        for i2 in 0..simple_grid1.height {
            if flag {
                break;
            }
            for i3 in (0..simple_grid1.width).rev() {
                if flag {
                    break;
                }
                if MansionGrid::is_house(simple_grid1, i3, i2) {
                    let south = rotation.rotate(Direction::SOUTH);
                    let east = rotation.rotate(Direction::EAST);
                    data2.position = data2
                        .position
                        .relative_n(south, 8 + (i2 - self.start_y) * 8)
                        .relative_n(east, (i3 - self.start_x) * 8);
                    self.traverse_wall_piece(pieces, &mut data2);
                    self.traverse_outer_walls(
                        pieces, &mut data2, simple_grid1, Direction::SOUTH, i3, i2, i3, i2,
                    );
                    flag = true;
                }
            }
        }

        self.create_roof(pieces, pos.above_n(16), rotation, simple_grid, Some(simple_grid1));
        self.create_roof(pieces, pos.above_n(27), rotation, simple_grid1, None);

        let floor_room_collections: [&dyn FloorRoomCollection; 3] = [
            &FirstFloorRoomCollection,
            &SecondFloorRoomCollection,
            &ThirdFloorRoomCollection,
        ];

        for i3x in 0..3usize {
            let block_pos = pos.above_n(8 * i3x as i32 + if i3x == 2 { 3 } else { 0 });
            let simple_grid2 = &grid.floor_rooms[i3x];
            let simple_grid3: &SimpleGrid = if i3x == 2 { simple_grid1 } else { simple_grid };
            let string: &str = if i3x == 0 { "carpet_south_1" } else { "carpet_south_2" };
            let string1: &str = if i3x == 0 { "carpet_west_1" } else { "carpet_west_2" };
            let south = rotation.rotate(Direction::SOUTH);
            let east = rotation.rotate(Direction::EAST);
            let west = rotation.rotate(Direction::WEST);
            let north = rotation.rotate(Direction::NORTH);

            for i4 in 0..simple_grid3.height {
                for i5 in 0..simple_grid3.width {
                    if simple_grid3.get(i5, i4) == CORRIDOR {
                        let mut block_pos1 = block_pos
                            .relative_n(south, 8 + (i4 - self.start_y) * 8)
                            .relative_n(east, (i5 - self.start_x) * 8);
                        Self::add_piece(
                            pieces, self.manager, "corridor_floor", block_pos1, rotation, Mirror::NONE,
                        );
                        if simple_grid3.get(i5, i4 - 1) == CORRIDOR
                            || (simple_grid2.get(i5, i4 - 1) & ROOM_CORRIDOR_FLAG)
                                == ROOM_CORRIDOR_FLAG
                        {
                            block_pos1 = block_pos1.relative_n(east, 1).above();
                            Self::add_piece(
                                pieces, self.manager, "carpet_north", block_pos1, rotation,
                                Mirror::NONE,
                            );
                        }
                        if simple_grid3.get(i5 + 1, i4) == CORRIDOR
                            || (simple_grid2.get(i5 + 1, i4) & ROOM_CORRIDOR_FLAG)
                                == ROOM_CORRIDOR_FLAG
                        {
                            let p = block_pos
                                .relative_n(south, 8 + (i4 - self.start_y) * 8)
                                .relative_n(east, (i5 - self.start_x) * 8)
                                .relative_n(south, 1)
                                .relative_n(east, 5)
                                .above();
                            Self::add_piece(
                                pieces, self.manager, "carpet_east", p, rotation, Mirror::NONE,
                            );
                        }
                        if simple_grid3.get(i5, i4 + 1) == CORRIDOR
                            || (simple_grid2.get(i5, i4 + 1) & ROOM_CORRIDOR_FLAG)
                                == ROOM_CORRIDOR_FLAG
                        {
                            let p = block_pos
                                .relative_n(south, 8 + (i4 - self.start_y) * 8)
                                .relative_n(east, (i5 - self.start_x) * 8)
                                .relative_n(south, 5)
                                .relative_n(west, 1);
                            Self::add_piece(
                                pieces, self.manager, string, p, rotation, Mirror::NONE,
                            );
                        }
                        if simple_grid3.get(i5 - 1, i4) == CORRIDOR
                            || (simple_grid2.get(i5 - 1, i4) & ROOM_CORRIDOR_FLAG)
                                == ROOM_CORRIDOR_FLAG
                        {
                            let p = block_pos
                                .relative_n(south, 8 + (i4 - self.start_y) * 8)
                                .relative_n(east, (i5 - self.start_x) * 8)
                                .relative_n(west, 1)
                                .relative_n(north, 1);
                            Self::add_piece(
                                pieces, self.manager, string1, p, rotation, Mirror::NONE,
                            );
                        }
                    }
                }
            }

            let string2: &str = if i3x == 0 { "indoors_wall_1" } else { "indoors_wall_2" };
            let string3: &str = if i3x == 0 { "indoors_door_1" } else { "indoors_door_2" };
            let mut list: Vec<Direction> = Vec::new();

            for i6 in 0..simple_grid3.height {
                for i7 in 0..simple_grid3.width {
                    let mut flag1 = i3x == 2
                        && simple_grid3.get(i7, i6) == START_ROOM;
                    if simple_grid3.get(i7, i6) == ROOM || flag1 {
                        let i8 = simple_grid2.get(i7, i6);
                        let i9 = i8 & ROOM_TYPE_MASK;
                        let i10 = i8 & ROOM_ID_MASK;
                        flag1 = flag1 && (i8 & ROOM_CORRIDOR_FLAG) == ROOM_CORRIDOR_FLAG;
                        list.clear();
                        if (i8 & ROOM_DOOR_FLAG) == ROOM_DOOR_FLAG {
                            for direction in Plane::HORIZONTAL.faces() {
                                if simple_grid3.get(
                                    i7 + direction.get_step_x(),
                                    i6 + direction.get_step_z(),
                                ) == CORRIDOR
                                {
                                    list.push(*direction);
                                }
                            }
                        }

                        let mut direction1: Option<Direction> = None;
                        if !list.is_empty() {
                            direction1 = Some(
                                list[self.random.next_int_bound(list.len() as i32) as usize],
                            );
                        } else if (i8 & ROOM_ORIGIN_FLAG) == ROOM_ORIGIN_FLAG {
                            direction1 = Some(Direction::UP);
                        }

                        let mut block_pos2 = block_pos
                            .relative_n(south, 8 + (i6 - self.start_y) * 8)
                            .relative_n(east, -1 + (i7 - self.start_x) * 8);
                        if MansionGrid::is_house(simple_grid3, i7 - 1, i6)
                            && !grid.is_room_id(i7 - 1, i6, i3x, i10)
                        {
                            let name = if direction1 == Some(Direction::WEST) {
                                string3
                            } else {
                                string2
                            };
                            Self::add_piece(
                                pieces, self.manager, name, block_pos2, rotation, Mirror::NONE,
                            );
                        }

                        if simple_grid3.get(i7 + 1, i6) == CORRIDOR && !flag1 {
                            let block_pos3 = block_pos2.relative_n(east, 8);
                            let name = if direction1 == Some(Direction::EAST) {
                                string3
                            } else {
                                string2
                            };
                            Self::add_piece(
                                pieces, self.manager, name, block_pos3, rotation, Mirror::NONE,
                            );
                        }

                        if MansionGrid::is_house(simple_grid3, i7, i6 + 1)
                            && !grid.is_room_id(i7, i6 + 1, i3x, i10)
                        {
                            let mut block_pos3 =
                                block_pos2.relative_n(south, 7).relative_n(east, 7);
                            let name = if direction1 == Some(Direction::SOUTH) {
                                string3
                            } else {
                                string2
                            };
                            Self::add_piece(
                                pieces,
                                self.manager,
                                name,
                                block_pos3,
                                rotation.get_rotated(Rotation::CLOCKWISE_90),
                                Mirror::NONE,
                            );
                        }

                        if simple_grid3.get(i7, i6 - 1) == CORRIDOR && !flag1 {
                            let mut block_pos3 =
                                block_pos2.relative_n(north, 1).relative_n(east, 7);
                            let name = if direction1 == Some(Direction::NORTH) {
                                string3
                            } else {
                                string2
                            };
                            Self::add_piece(
                                pieces,
                                self.manager,
                                name,
                                block_pos3,
                                rotation.get_rotated(Rotation::CLOCKWISE_90),
                                Mirror::NONE,
                            );
                        }

                        if i9 == ROOM_1X1 {
                            self.add_room_1x1(
                                pieces, block_pos2, rotation, direction1,
                                floor_room_collections[i3x],
                            );
                        } else if i9 == ROOM_1X2 && direction1.is_some() {
                            let direction2 = grid.get_1x2_room_direction(i7, i6, i3x, i10);
                            let flag2 = (i8 & ROOM_STAIRS_FLAG) == ROOM_STAIRS_FLAG;
                            self.add_room_1x2(
                                pieces,
                                block_pos2,
                                rotation,
                                direction2,
                                direction1.unwrap(),
                                floor_room_collections[i3x],
                                flag2,
                            );
                        } else if i9 == ROOM_2X2 && direction1.is_some() && direction1 != Some(Direction::UP) {
                            let d1 = direction1.unwrap();
                            let mut direction2 = d1.get_clock_wise();
                            if !grid.is_room_id(
                                i7 + direction2.get_step_x(),
                                i6 + direction2.get_step_z(),
                                i3x,
                                i10,
                            ) {
                                direction2 = direction2.get_opposite();
                            }
                            self.add_room_2x2(
                                pieces,
                                block_pos2,
                                rotation,
                                direction2,
                                d1,
                                floor_room_collections[i3x],
                            );
                        } else if i9 == ROOM_2X2 && direction1 == Some(Direction::UP) {
                            self.add_room_2x2_secret(
                                pieces,
                                block_pos2,
                                rotation,
                                floor_room_collections[i3x],
                            );
                        }
                    }
                }
            }
        }
    }

    fn traverse_outer_walls(
        &mut self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        data: &mut PlacementData,
        layout: &SimpleGrid,
        mut direction: Direction,
        start_x: i32,
        start_y: i32,
        entrance_x: i32,
        entrance_y: i32,
    ) {
        let direction1 = direction;
        let (mut i, mut i1) = (start_x, start_y);
        loop {
            if !MansionGrid::is_house(
                layout,
                i + direction.get_step_x(),
                i1 + direction.get_step_z(),
            ) {
                self.traverse_turn(pieces, data);
                direction = direction.get_clock_wise();
                if i != entrance_x || i1 != entrance_y || direction1 != direction {
                    self.traverse_wall_piece(pieces, data);
                }
            } else if MansionGrid::is_house(
                layout,
                i + direction.get_step_x(),
                i1 + direction.get_step_z(),
            ) && MansionGrid::is_house(
                layout,
                i + direction.get_step_x()
                    + direction.get_counter_clock_wise().get_step_x(),
                i1 + direction.get_step_z()
                    + direction.get_counter_clock_wise().get_step_z(),
            ) {
                self.traverse_inner_turn(pieces, data);
                i += direction.get_step_x();
                i1 += direction.get_step_z();
                direction = direction.get_counter_clock_wise();
            } else {
                i += direction.get_step_x();
                i1 += direction.get_step_z();
                if i != entrance_x || i1 != entrance_y || direction1 != direction {
                    self.traverse_wall_piece(pieces, data);
                }
            }
            if i == entrance_x && i1 == entrance_y && direction1 == direction {
                break;
            }
        }
    }

    fn create_roof(
        &mut self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        pos: BlockPos,
        rotation: Rotation,
        layout: &SimpleGrid,
        next_floor_layout: Option<&SimpleGrid>,
    ) {
        let south = rotation.rotate(Direction::SOUTH);
        let east = rotation.rotate(Direction::EAST);
        let west = rotation.rotate(Direction::WEST);
        for i in 0..layout.height {
            for i1 in 0..layout.width {
                let block_pos = pos
                    .relative_n(south, 8 + (i - self.start_y) * 8)
                    .relative_n(east, (i1 - self.start_x) * 8);
                let flag = next_floor_layout
                    .map(|l| MansionGrid::is_house(l, i1, i))
                    .unwrap_or(false);
                if MansionGrid::is_house(layout, i1, i) && !flag {
                    Self::add_piece(
                        pieces, self.manager, "roof", block_pos.above_n(3), rotation, Mirror::NONE,
                    );
                    if !MansionGrid::is_house(layout, i1 + 1, i) {
                        let block_pos1 = block_pos.relative_n(east, 6);
                        Self::add_piece(
                            pieces, self.manager, "roof_front", block_pos1, rotation, Mirror::NONE,
                        );
                    }
                    if !MansionGrid::is_house(layout, i1 - 1, i) {
                        let block_pos1 =
                            block_pos.relative_n(east, 0).relative_n(south, 7);
                        Self::add_piece(
                            pieces,
                            self.manager,
                            "roof_front",
                            block_pos1,
                            rotation.get_rotated(Rotation::CLOCKWISE_180),
                            Mirror::NONE,
                        );
                    }
                    if !MansionGrid::is_house(layout, i1, i - 1) {
                        let block_pos1 = block_pos.relative_n(west, 1);
                        Self::add_piece(
                            pieces,
                            self.manager,
                            "roof_front",
                            block_pos1,
                            rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90),
                            Mirror::NONE,
                        );
                    }
                    if !MansionGrid::is_house(layout, i1, i + 1) {
                        let block_pos1 =
                            block_pos.relative_n(east, 6).relative_n(south, 6);
                        Self::add_piece(
                            pieces,
                            self.manager,
                            "roof_front",
                            block_pos1,
                            rotation.get_rotated(Rotation::CLOCKWISE_90),
                            Mirror::NONE,
                        );
                    }
                }
            }
        }

        if let Some(next) = next_floor_layout {
            for i in 0..layout.height {
                for i1x in 0..layout.width {
                    let var17 = pos
                        .relative_n(south, 8 + (i - self.start_y) * 8)
                        .relative_n(east, (i1x - self.start_x) * 8);
                    let flag = MansionGrid::is_house(next, i1x, i);
                    if MansionGrid::is_house(layout, i1x, i) && flag {
                        if !MansionGrid::is_house(layout, i1x + 1, i) {
                            let block_pos1 = var17.relative_n(east, 7);
                            Self::add_piece(
                                pieces, self.manager, "small_wall", block_pos1, rotation,
                                Mirror::NONE,
                            );
                        }
                        if !MansionGrid::is_house(layout, i1x - 1, i) {
                            let block_pos1 = var17
                                .relative_n(west, 1)
                                .relative_n(south, 6);
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "small_wall",
                                block_pos1,
                                rotation.get_rotated(Rotation::CLOCKWISE_180),
                                Mirror::NONE,
                            );
                        }
                        if !MansionGrid::is_house(layout, i1x, i - 1) {
                            let block_pos1 = var17
                                .relative_n(west, 0)
                                .relative_n(rotation.rotate(Direction::NORTH), 1);
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "small_wall",
                                block_pos1,
                                rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90),
                                Mirror::NONE,
                            );
                        }
                        if !MansionGrid::is_house(layout, i1x, i + 1) {
                            let block_pos1 = var17
                                .relative_n(east, 6)
                                .relative_n(south, 7);
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "small_wall",
                                block_pos1,
                                rotation.get_rotated(Rotation::CLOCKWISE_90),
                                Mirror::NONE,
                            );
                        }
                        if !MansionGrid::is_house(layout, i1x + 1, i) {
                            if !MansionGrid::is_house(layout, i1x, i - 1) {
                                let block_pos1 = var17
                                    .relative_n(east, 7)
                                    .relative_n(rotation.rotate(Direction::NORTH), 2);
                                Self::add_piece(
                                    pieces, self.manager, "small_wall_corner", block_pos1,
                                    rotation, Mirror::NONE,
                                );
                            }
                            if !MansionGrid::is_house(layout, i1x, i + 1) {
                                let block_pos1 = var17
                                    .relative_n(east, 8)
                                    .relative_n(south, 7);
                                Self::add_piece(
                                    pieces,
                                    self.manager,
                                    "small_wall_corner",
                                    block_pos1,
                                    rotation.get_rotated(Rotation::CLOCKWISE_90),
                                    Mirror::NONE,
                                );
                            }
                        }
                        if !MansionGrid::is_house(layout, i1x - 1, i) {
                            if !MansionGrid::is_house(layout, i1x, i - 1) {
                                let block_pos1 = var17
                                    .relative_n(west, 2)
                                    .relative_n(rotation.rotate(Direction::NORTH), 1);
                                Self::add_piece(
                                    pieces,
                                    self.manager,
                                    "small_wall_corner",
                                    block_pos1,
                                    rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90),
                                    Mirror::NONE,
                                );
                            }
                            if !MansionGrid::is_house(layout, i1x, i + 1) {
                                let block_pos1 = var17
                                    .relative_n(west, 1)
                                    .relative_n(south, 8);
                                Self::add_piece(
                                    pieces,
                                    self.manager,
                                    "small_wall_corner",
                                    block_pos1,
                                    rotation.get_rotated(Rotation::CLOCKWISE_180),
                                    Mirror::NONE,
                                );
                            }
                        }
                    }
                }
            }
        }

        for i in 0..layout.height {
            for i1xx in 0..layout.width {
                let var19 = pos
                    .relative_n(south, 8 + (i - self.start_y) * 8)
                    .relative_n(east, (i1xx - self.start_x) * 8);
                let flag = next_floor_layout
                    .map(|l| MansionGrid::is_house(l, i1xx, i))
                    .unwrap_or(false);
                if MansionGrid::is_house(layout, i1xx, i) && !flag {
                    if !MansionGrid::is_house(layout, i1xx + 1, i) {
                        let block_pos1 = var19.relative_n(east, 6);
                        if !MansionGrid::is_house(layout, i1xx, i + 1) {
                            let block_pos2 = block_pos1.relative_n(south, 6);
                            Self::add_piece(
                                pieces, self.manager, "roof_corner", block_pos2, rotation,
                                Mirror::NONE,
                            );
                        } else if MansionGrid::is_house(layout, i1xx + 1, i + 1) {
                            let block_pos2 = block_pos1.relative_n(south, 5);
                            Self::add_piece(
                                pieces, self.manager, "roof_inner_corner", block_pos2,
                                rotation, Mirror::NONE,
                            );
                        }
                        if !MansionGrid::is_house(layout, i1xx, i - 1) {
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "roof_corner",
                                block_pos1,
                                rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90),
                                Mirror::NONE,
                            );
                        } else if MansionGrid::is_house(layout, i1xx + 1, i - 1) {
                            let block_pos2 = var19
                                .relative_n(east, 9)
                                .relative_n(rotation.rotate(Direction::NORTH), 2);
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "roof_inner_corner",
                                block_pos2,
                                rotation.get_rotated(Rotation::CLOCKWISE_90),
                                Mirror::NONE,
                            );
                        }
                    }
                    if !MansionGrid::is_house(layout, i1xx - 1, i) {
                        let block_pos1x = var19.relative_n(east, 0).relative_n(south, 0);
                        if !MansionGrid::is_house(layout, i1xx, i + 1) {
                            let block_pos2 = block_pos1x.relative_n(south, 6);
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "roof_corner",
                                block_pos2,
                                rotation.get_rotated(Rotation::CLOCKWISE_90),
                                Mirror::NONE,
                            );
                        } else if MansionGrid::is_house(layout, i1xx - 1, i + 1) {
                            let block_pos2 = block_pos1x
                                .relative_n(south, 8)
                                .relative_n(west, 3);
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "roof_inner_corner",
                                block_pos2,
                                rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90),
                                Mirror::NONE,
                            );
                        }
                        if !MansionGrid::is_house(layout, i1xx, i - 1) {
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "roof_corner",
                                block_pos1x,
                                rotation.get_rotated(Rotation::CLOCKWISE_180),
                                Mirror::NONE,
                            );
                        } else if MansionGrid::is_house(layout, i1xx - 1, i - 1) {
                            let block_pos2 = block_pos1x.relative_n(south, 1);
                            Self::add_piece(
                                pieces,
                                self.manager,
                                "roof_inner_corner",
                                block_pos2,
                                rotation.get_rotated(Rotation::CLOCKWISE_180),
                                Mirror::NONE,
                            );
                        }
                    }
                }
            }
        }
    }

    fn entrance(
        &self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        data: &mut PlacementData,
    ) {
        let direction = data.rotation.rotate(Direction::WEST);
        Self::add_piece(
            pieces,
            self.manager,
            "entrance",
            data.position.relative_n(direction, 9),
            data.rotation,
            Mirror::NONE,
        );
        data.position = data
            .position
            .relative_n(data.rotation.rotate(Direction::SOUTH), 16);
    }

    fn traverse_wall_piece(
        &self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        data: &mut PlacementData,
    ) {
        Self::add_piece(
            pieces,
            self.manager,
            data.wall_type,
            data.position.relative_n(data.rotation.rotate(Direction::EAST), 7),
            data.rotation,
            Mirror::NONE,
        );
        data.position = data
            .position
            .relative_n(data.rotation.rotate(Direction::SOUTH), 8);
    }

    fn traverse_turn(
        &self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        data: &mut PlacementData,
    ) {
        data.position = data
            .position
            .relative_n(data.rotation.rotate(Direction::SOUTH), -1);
        Self::add_piece(
            pieces, self.manager, "wall_corner", data.position, data.rotation, Mirror::NONE,
        );
        data.position = data
            .position
            .relative_n(data.rotation.rotate(Direction::SOUTH), -7)
            .relative_n(data.rotation.rotate(Direction::WEST), -6);
        data.rotation = data.rotation.get_rotated(Rotation::CLOCKWISE_90);
    }

    fn traverse_inner_turn(
        &self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        data: &mut PlacementData,
    ) {
        data.position = data
            .position
            .relative_n(data.rotation.rotate(Direction::SOUTH), 6)
            .relative_n(data.rotation.rotate(Direction::EAST), 8);
        data.rotation = data.rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90);
    }

    fn add_room_1x1(
        &mut self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        pos: BlockPos,
        rotation: Rotation,
        direction: Option<Direction>,
        floor_rooms: &dyn FloorRoomCollection,
    ) {
        let mut rotation1 = Rotation::NONE;
        let mut string = floor_rooms.get_1x1(&mut *self.random);
        match direction {
            Some(Direction::EAST) => {}
            Some(Direction::NORTH) => {
                rotation1 = rotation1.get_rotated(Rotation::COUNTERCLOCKWISE_90);
            }
            Some(Direction::WEST) => {
                rotation1 = rotation1.get_rotated(Rotation::CLOCKWISE_180);
            }
            Some(Direction::SOUTH) => {
                rotation1 = rotation1.get_rotated(Rotation::CLOCKWISE_90);
            }
            _ => {
                string = floor_rooms.get_1x1_secret(&mut *self.random);
            }
        }

        let zero_position_with_transform = StructureTemplate::get_zero_position_with_transform_full(
            BlockPos::new(1, 0, 0),
            Mirror::NONE,
            rotation1,
            7,
            7,
        );
        rotation1 = rotation1.get_rotated(rotation);
        let zero = zero_position_with_transform.rotate(rotation);
        let block_pos = pos.offset(zero.x, 0, zero.z);
        Self::add_piece(pieces, self.manager, &string, block_pos, rotation1, Mirror::NONE);
    }

    #[allow(clippy::too_many_arguments)]
    fn add_room_1x2(
        &mut self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        pos: BlockPos,
        rotation: Rotation,
        front_direction: Option<Direction>,
        side_direction: Direction,
        floor_rooms: &dyn FloorRoomCollection,
        is_stairs: bool,
    ) {
        let south = rotation.rotate(Direction::SOUTH);
        let east = rotation.rotate(Direction::EAST);
        let west = rotation.rotate(Direction::WEST);
        let north = rotation.rotate(Direction::NORTH);
        // Порт 16 веток Java `addRoom1x2` — сравнения Direction.
        if side_direction == Direction::EAST && front_direction == Some(Direction::SOUTH) {
            let p = pos.relative_n(east, 1);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(pieces, self.manager, &name, p, rotation, Mirror::NONE);
        } else if side_direction == Direction::EAST && front_direction == Some(Direction::NORTH) {
            let p = pos.relative_n(east, 1).relative_n(south, 6);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(pieces, self.manager, &name, p, rotation, Mirror::LEFT_RIGHT);
        } else if side_direction == Direction::WEST && front_direction == Some(Direction::NORTH) {
            let p = pos.relative_n(east, 7).relative_n(south, 6);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::CLOCKWISE_180), Mirror::NONE,
            );
        } else if side_direction == Direction::WEST && front_direction == Some(Direction::SOUTH) {
            let p = pos.relative_n(east, 7);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(pieces, self.manager, &name, p, rotation, Mirror::FRONT_BACK);
        } else if side_direction == Direction::SOUTH && front_direction == Some(Direction::EAST) {
            let p = pos.relative_n(east, 1);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::CLOCKWISE_90), Mirror::LEFT_RIGHT,
            );
        } else if side_direction == Direction::SOUTH && front_direction == Some(Direction::WEST) {
            let p = pos.relative_n(east, 7);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::CLOCKWISE_90), Mirror::NONE,
            );
        } else if side_direction == Direction::NORTH && front_direction == Some(Direction::WEST) {
            let p = pos.relative_n(east, 7).relative_n(south, 6);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::CLOCKWISE_90), Mirror::FRONT_BACK,
            );
        } else if side_direction == Direction::NORTH && front_direction == Some(Direction::EAST) {
            let p = pos.relative_n(east, 1).relative_n(south, 6);
            let name = floor_rooms.get_1x2_side_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90), Mirror::NONE,
            );
        } else if side_direction == Direction::SOUTH && front_direction == Some(Direction::NORTH) {
            let p = pos.relative_n(east, 1).relative_n(north, 8);
            let name = floor_rooms.get_1x2_front_entrance(&mut *self.random, is_stairs);
            Self::add_piece(pieces, self.manager, &name, p, rotation, Mirror::NONE);
        } else if side_direction == Direction::NORTH && front_direction == Some(Direction::SOUTH) {
            let p = pos.relative_n(east, 7).relative_n(south, 14);
            let name = floor_rooms.get_1x2_front_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::CLOCKWISE_180), Mirror::NONE,
            );
        } else if side_direction == Direction::WEST && front_direction == Some(Direction::EAST) {
            let p = pos.relative_n(east, 15);
            let name = floor_rooms.get_1x2_front_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::CLOCKWISE_90), Mirror::NONE,
            );
        } else if side_direction == Direction::EAST && front_direction == Some(Direction::WEST) {
            let p = pos.relative_n(west, 7).relative_n(south, 6);
            let name = floor_rooms.get_1x2_front_entrance(&mut *self.random, is_stairs);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90), Mirror::NONE,
            );
        } else if side_direction == Direction::UP && front_direction == Some(Direction::EAST) {
            let p = pos.relative_n(east, 15);
            let name = floor_rooms.get_1x2_secret(&mut *self.random);
            Self::add_piece(
                pieces, self.manager, &name, p,
                rotation.get_rotated(Rotation::CLOCKWISE_90), Mirror::NONE,
            );
        } else if side_direction == Direction::UP && front_direction == Some(Direction::SOUTH) {
            let p = pos.relative_n(east, 1).relative_n(north, 0);
            let name = floor_rooms.get_1x2_secret(&mut *self.random);
            Self::add_piece(pieces, self.manager, &name, p, rotation, Mirror::NONE);
        }
    }

    fn add_room_2x2(
        &mut self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        pos: BlockPos,
        rotation: Rotation,
        front_direction: Direction,
        side_direction: Direction,
        floor_rooms: &dyn FloorRoomCollection,
    ) {
        let mut i = 0;
        let mut i1 = 0;
        let mut rotation1 = rotation;
        let mut mirror = Mirror::NONE;
        if side_direction == Direction::EAST && front_direction == Direction::SOUTH {
            i = -7;
        } else if side_direction == Direction::EAST && front_direction == Direction::NORTH {
            i = -7;
            i1 = 6;
            mirror = Mirror::LEFT_RIGHT;
        } else if side_direction == Direction::NORTH && front_direction == Direction::EAST {
            i = 1;
            i1 = 14;
            rotation1 = rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90);
        } else if side_direction == Direction::NORTH && front_direction == Direction::WEST {
            i = 7;
            i1 = 14;
            rotation1 = rotation.get_rotated(Rotation::COUNTERCLOCKWISE_90);
            mirror = Mirror::LEFT_RIGHT;
        } else if side_direction == Direction::SOUTH && front_direction == Direction::WEST {
            i = 7;
            i1 = -8;
            rotation1 = rotation.get_rotated(Rotation::CLOCKWISE_90);
        } else if side_direction == Direction::SOUTH && front_direction == Direction::EAST {
            i = 1;
            i1 = -8;
            rotation1 = rotation.get_rotated(Rotation::CLOCKWISE_90);
            mirror = Mirror::LEFT_RIGHT;
        } else if side_direction == Direction::WEST && front_direction == Direction::NORTH {
            i = 15;
            i1 = 6;
            rotation1 = rotation.get_rotated(Rotation::CLOCKWISE_180);
        } else if side_direction == Direction::WEST && front_direction == Direction::SOUTH {
            i = 15;
            mirror = Mirror::FRONT_BACK;
        }

        let east = rotation.rotate(Direction::EAST);
        let south = rotation.rotate(Direction::SOUTH);
        let block_pos = pos.relative_n(east, i).relative_n(south, i1);
        let name = floor_rooms.get_2x2(&mut *self.random);
        Self::add_piece(pieces, self.manager, &name, block_pos, rotation1, mirror);
    }

    fn add_room_2x2_secret(
        &mut self,
        pieces: &mut Vec<Box<dyn StructurePiece>>,
        pos: BlockPos,
        rotation: Rotation,
        floor_rooms: &dyn FloorRoomCollection,
    ) {
        let block_pos = pos.relative_n(rotation.rotate(Direction::EAST), 1);
        let name = floor_rooms.get_2x2_secret(&mut *self.random);
        Self::add_piece(pieces, self.manager, &name, block_pos, rotation, Mirror::NONE);
    }
}

/// `handleDataMarker` — сундуки (ChestWest/East/South/North) и
/// мобы (Mage→evoker, Warrior→vindicator, Group of Allays→1-3 allay).
fn mansion_data_marker_handler(rotation: Rotation) -> DataMarkerHandler {
    Rc::new(move |name, pos, level, _random, box_| {
        if name.starts_with("Chest") {
            let mut state = block_state("chest");
            let facing = match name {
                "ChestWest" => Some(rotation.rotate(Direction::WEST)),
                "ChestEast" => Some(rotation.rotate(Direction::EAST)),
                "ChestSouth" => Some(rotation.rotate(Direction::SOUTH)),
                "ChestNorth" => Some(rotation.rotate(Direction::NORTH)),
                _ => None,
            };
            if let Some(dir) = facing {
                state = with_string_property(state, "facing", dir.get_name());
            }
            create_chest_at(
                level,
                box_,
                _random,
                pos,
                &Identifier::minecraft(WOODLAND_MANSION_LOOT),
                Some(&state),
            );
        } else {
            let types: Vec<&'static str> = match name {
                "Mage" => vec!["evoker"],
                "Warrior" => vec!["vindicator"],
                "Group of Allays" => {
                    let n = level.level_random_next_int(3) + 1;
                    vec!["allay"; n.max(0) as usize]
                }
                _ => return,
            };
            for ty in types {
                if level
                    .spawn_entity_of_type(&Identifier::minecraft(ty), pos)
                    .is_some()
                {
                    level.set_block_state(pos, &BlockState::air());
                }
            }
        }
    })
}

/// `WoodlandMansionPieces.generateMansion`.
pub fn generate_mansion(
    structure_template_manager: &StructureTemplateManager,
    pos: BlockPos,
    rotation: Rotation,
    pieces: &mut Vec<Box<dyn StructurePiece>>,
    random: &mut dyn RandomSource,
) {
    let grid = MansionGrid::new(random);
    let mut placer = MansionPiecePlacer::new(structure_template_manager, random);
    placer.create_mansion(pos, rotation, pieces, &grid);
}

/// `WoodlandMansionStructure` — фабрика `Structure`.
pub fn woodland_mansion_structure(
    id: Identifier,
    settings: StructureSettings,
) -> Structure {
    let structure_type = register_structure_type(Identifier::minecraft("woodland_mansion"));
    let mut structure = Structure::new(id, settings, structure_type);
    structure.find_generation_point = Some(Arc::new(
        |context: &mut GenerationContext| {
            let rotation = Rotation::get_random(&mut context.random);
            let pos = get_lowest_y_in_5by5_box_offset_7_blocks(context, rotation);
            if pos.y < 60 {
                return None;
            }
            Some(GenerationStub::new(pos, move |out, ctx| {
                if let Some(manager) = ctx.template_manager.as_ref() {
                    generate_mansion(
                        manager.as_ref(),
                        pos,
                        rotation,
                        out,
                        &mut ctx.random,
                    );
                }
            }))
        },
    ));
    structure.after_place = Some(Arc::new(
        |level: &dyn LevelReader,
         _structure_manager: &dyn StructureManagerAccess,
         _random: &mut dyn RandomSource,
         bounding_box: &BoundingBox,
         _chunk_pos: ChunkPos,
         pieces: &PiecesContainer| {
            let Some(bb1) = pieces.calculate_bounding_box() else {
                return;
            };
            let min_y = level.min_build_height();
            let min_y1 = bb1.min_y;
            let cobble = block_state("cobblestone");
            for x in bounding_box.min_x..=bounding_box.max_x {
                for z in bounding_box.min_z..=bounding_box.max_z {
                    let mut p = BlockPos::new(x, min_y1, z);
                    if !level.is_empty_block(p)
                        && bb1.is_inside_pos(p)
                        && pieces.is_inside_piece(p)
                    {
                        let mut y = min_y1 - 1;
                        while y > min_y {
                            p = BlockPos::new(x, y, z);
                            let state = level.get_block_state(p);
                            if !level.is_empty_block(p)
                                && !state.map(|s| s.is_liquid_block()).unwrap_or(false)
                            {
                                break;
                            }
                            level.set_block_state(p, &cobble);
                            y -= 1;
                        }
                    }
                }
            }
        },
    ));
    structure
}
