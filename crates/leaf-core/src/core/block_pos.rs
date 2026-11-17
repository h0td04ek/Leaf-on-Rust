//! Перенос `net.minecraft.core.BlockPos` (+ MutableBlockPos и итераторы).
//! Codec/StreamCodec — TODO. `Position`-трейт и Entity-конструктор — TODO.

use crate::core::direction::{Axis, Direction};
use crate::core::vec3i::Vec3i;
use crate::random::RandomSource;
use crate::util::mth;
use crate::world::level::block::Rotation;
use crate::world::phys::{Aabb, Vec3};
use std::collections::VecDeque;

pub const ZERO: BlockPos = BlockPos { x: 0, y: 0, z: 0 };

pub const PACKED_HORIZONTAL_LENGTH: i32 = 26;
pub const PACKED_Y_LENGTH: i32 = 12;
const PACKED_X_MASK: i64 = 67108863;
const PACKED_Y_MASK: i64 = 4095;
const PACKED_Z_MASK: i64 = 67108863;
const Y_OFFSET: i32 = 0;
const Z_OFFSET: i32 = 12;
const X_OFFSET: i32 = 38;
pub const MAX_HORIZONTAL_COORDINATE: i32 = 33554431;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    // ---------------------------------------------------------- packed long

    /// Paper-оптимизированный `getAdjacent`
    pub fn get_adjacent(base_x: i32, base_y: i32, base_z: i32, direction: Direction) -> i64 {
        Self::as_long_static(
            base_x + direction.get_step_x(),
            base_y + direction.get_step_y(),
            base_z + direction.get_step_z(),
        )
    }

    pub fn offset_packed(pos: i64, direction: Direction) -> i64 {
        Self::offset_packed_xyz(pos, direction.get_step_x(), direction.get_step_y(), direction.get_step_z())
    }

    /// Paper-упрощённый unpack+repack.
    pub fn offset_packed_xyz(pos: i64, dx: i32, dy: i32, dz: i32) -> i64 {
        Self::as_long_static(
            ((pos >> 38) as i32) + dx,
            (((pos << 52) >> 52) as i32) + dy,
            (((pos << 26) >> 38) as i32) + dz,
        )
    }

    pub fn get_x_packed(packed_pos: i64) -> i32 {
        (packed_pos >> 38) as i32
    }

    pub fn get_y_packed(packed_pos: i64) -> i32 {
        ((packed_pos << 52) >> 52) as i32
    }

    pub fn get_z_packed(packed_pos: i64) -> i32 {
        ((packed_pos << 26) >> 38) as i32
    }

    pub fn of(packed_pos: i64) -> Self {
        Self::new(
            Self::get_x_packed(packed_pos),
            Self::get_y_packed(packed_pos),
            Self::get_z_packed(packed_pos),
        )
    }

    pub fn containing(x: f64, y: f64, z: f64) -> Self {
        Self::new(mth::floor(x), mth::floor(y), mth::floor(z))
    }

    pub fn containing_vec3(position: &Vec3) -> Self {
        Self::containing(position.x, position.y, position.z)
    }

    pub fn min_pos(pos1: &Self, pos2: &Self) -> Self {
        Self::new(
            pos1.x.min(pos2.x),
            pos1.y.min(pos2.y),
            pos1.z.min(pos2.z),
        )
    }

    pub fn max_pos(pos1: &Self, pos2: &Self) -> Self {
        Self::new(
            pos1.x.max(pos2.x),
            pos1.y.max(pos2.y),
            pos1.z.max(pos2.z),
        )
    }

    pub fn as_long(&self) -> i64 {
        Self::as_long_static(self.x, self.y, self.z)
    }

    /// `BlockPos.asLong(x, y, z)` — статическая упаковка.
    pub fn as_long_static(x: i32, y: i32, z: i32) -> i64 {
        ((x as i64 & PACKED_X_MASK) << X_OFFSET)
            | (((y as i64) & PACKED_Y_MASK) << Y_OFFSET)
            | ((z as i64 & PACKED_Z_MASK) << Z_OFFSET)
    }

    pub fn get_flat_index(packed_pos: i64) -> i64 {
        packed_pos & -16
    }

    // ---------------------------------------------------------- offset/move

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        if dx == 0 && dy == 0 && dz == 0 {
            *self
        } else {
            Self::new(self.x + dx, self.y + dy, self.z + dz)
        }
    }

    pub fn offset_vec3i(&self, vector: &Vec3i) -> Self {
        self.offset(vector.x, vector.y, vector.z)
    }

    pub fn subtract(&self, vector: &Vec3i) -> Self {
        self.offset(-vector.x, -vector.y, -vector.z)
    }

    pub fn multiply(&self, scalar: i32) -> Self {
        if scalar == 1 {
            *self
        } else if scalar == 0 {
            ZERO
        } else {
            Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
        }
    }

    pub fn get_center(&self) -> Vec3 {
        crate::world::phys::vec3::at_center_of(&self.to_vec3i())
    }

    pub fn get_bottom_center(&self) -> Vec3 {
        crate::world::phys::vec3::at_bottom_center_of(&self.to_vec3i())
    }

    pub fn above(&self) -> Self {
        Self::new(self.x, self.y + 1, self.z)
    }

    pub fn above_n(&self, distance: i32) -> Self {
        if distance == 0 {
            *self
        } else {
            Self::new(self.x, self.y + distance, self.z)
        }
    }

    pub fn below(&self) -> Self {
        Self::new(self.x, self.y - 1, self.z)
    }

    pub fn below_n(&self, distance: i32) -> Self {
        if distance == 0 {
            *self
        } else {
            Self::new(self.x, self.y - distance, self.z)
        }
    }

    pub fn north(&self) -> Self {
        Self::new(self.x, self.y, self.z - 1)
    }

    pub fn north_n(&self, distance: i32) -> Self {
        if distance == 0 {
            *self
        } else {
            Self::new(self.x, self.y, self.z - distance)
        }
    }

    pub fn south(&self) -> Self {
        Self::new(self.x, self.y, self.z + 1)
    }

    pub fn south_n(&self, distance: i32) -> Self {
        if distance == 0 {
            *self
        } else {
            Self::new(self.x, self.y, self.z + distance)
        }
    }

    pub fn west(&self) -> Self {
        Self::new(self.x - 1, self.y, self.z)
    }

    pub fn west_n(&self, distance: i32) -> Self {
        if distance == 0 {
            *self
        } else {
            Self::new(self.x - distance, self.y, self.z)
        }
    }

    pub fn east(&self) -> Self {
        Self::new(self.x + 1, self.y, self.z)
    }

    pub fn east_n(&self, distance: i32) -> Self {
        if distance == 0 {
            *self
        } else {
            Self::new(self.x + distance, self.y, self.z)
        }
    }

    /// Paper-оптимизированный `relative(Direction)`.
    pub fn relative(&self, direction: Direction) -> Self {
        match direction {
            Direction::UP => Self::new(self.x, self.y + 1, self.z),
            Direction::DOWN => Self::new(self.x, self.y - 1, self.z),
            Direction::NORTH => Self::new(self.x, self.y, self.z - 1),
            Direction::SOUTH => Self::new(self.x, self.y, self.z + 1),
            Direction::WEST => Self::new(self.x - 1, self.y, self.z),
            Direction::EAST => Self::new(self.x + 1, self.y, self.z),
        }
    }

    pub fn relative_n(&self, direction: Direction, distance: i32) -> Self {
        if distance == 0 {
            *self
        } else {
            Self::new(
                self.x + direction.get_step_x() * distance,
                self.y + direction.get_step_y() * distance,
                self.z + direction.get_step_z() * distance,
            )
        }
    }

    pub fn relative_axis(&self, axis: Axis, amount: i32) -> Self {
        if amount == 0 {
            *self
        } else {
            let i = if axis == Axis::X { amount } else { 0 };
            let i1 = if axis == Axis::Y { amount } else { 0 };
            let i2 = if axis == Axis::Z { amount } else { 0 };
            Self::new(self.x + i, self.y + i1, self.z + i2)
        }
    }

    pub fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::CLOCKWISE_90 => Self::new(-self.z, self.y, self.x),
            Rotation::CLOCKWISE_180 => Self::new(-self.x, self.y, -self.z),
            Rotation::COUNTERCLOCKWISE_90 => Self::new(self.z, self.y, -self.x),
            Rotation::NONE => *self,
        }
    }

    pub fn cross(&self, vector: &Vec3i) -> Self {
        Self::new(
            self.y * vector.z - self.z * vector.y,
            self.z * vector.x - self.x * vector.z,
            self.x * vector.y - self.y * vector.x,
        )
    }

    pub fn at_y(&self, y: i32) -> Self {
        Self::new(self.x, y, self.z)
    }

    pub fn immutable(&self) -> Self {
        *self
    }

    pub fn mutable(&self) -> MutableBlockPos {
        MutableBlockPos::new(self.x, self.y, self.z)
    }

    pub fn clamp_location_within(&self, pos: &Vec3) -> Vec3 {
        Vec3::new(
            mth::clamp_f64(pos.x, (self.x as f64) + 1.0e-5, self.x as f64 + 1.0 - 1.0e-5),
            mth::clamp_f64(pos.y, (self.y as f64) + 1.0e-5, self.y as f64 + 1.0 - 1.0e-5),
            mth::clamp_f64(pos.z, (self.z as f64) + 1.0e-5, self.z as f64 + 1.0 - 1.0e-5),
        )
    }

    pub fn to_vec3i(&self) -> Vec3i {
        Vec3i::new(self.x, self.y, self.z)
    }

    // ---------------------------------------------------------- итераторы

    /// `randomInCube`
    pub fn random_in_cube(
        random: &mut dyn RandomSource,
        amount: i32,
        center: &Self,
        radius: i32,
    ) -> Vec<Self> {
        Self::random_between_closed_vec(
            random,
            amount,
            center.x - radius,
            center.y - radius,
            center.z - radius,
            center.x + radius,
            center.y + radius,
            center.z + radius,
        )
    }

    /// `squareOutSouthEast` — [pos, south, east, south.east]
    pub fn square_out_south_east(pos: &Self) -> [Self; 4] {
        [*pos, pos.south(), pos.east(), pos.south().east()]
    }

    /// `randomBetweenClosed` — собираем в Vec (amount ограничен вызывающим).
    /// Для ленивого варианта используйте `RandomBetweenClosed`.
    pub fn random_between_closed_vec(
        random: &mut dyn RandomSource,
        amount: i32,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Vec<Self> {
        RandomBetweenClosed::new_owned(random.fork(), amount, min_x, min_y, min_z, max_x, max_y, max_z)
            .collect()
    }

    pub fn random_between_closed<'a>(
        random: &'a mut dyn RandomSource,
        amount: i32,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> RandomBetweenClosed<'a> {
        RandomBetweenClosed {
            random: Some(random),
            owned: None,
            counter: amount,
            min: Self::new(min_x, min_y, min_z),
            dims: (max_x - min_x + 1, max_y - min_y + 1, max_z - min_z + 1),
        }
    }

    /// `withinManhattan` — итератор "ромбовидных" оболочек.
    /// JettPack-кэш `IterateOutwardsCache` не портирован — итерируем напрямую.
    pub fn within_manhattan(pos: &Self, x_size: i32, y_size: i32, z_size: i32) -> WithinManhattan {
        WithinManhattan::new(*pos, x_size, y_size, z_size)
    }

    pub fn find_closest_match(
        pos: &Self,
        width: i32,
        height: i32,
        pos_filter: &mut dyn FnMut(&Self) -> bool,
    ) -> Option<Self> {
        for block_pos in Self::within_manhattan(pos, width, height, width) {
            if pos_filter(&block_pos) {
                return Some(block_pos);
            }
        }
        None
    }

    /// `betweenClosed(AABB)`
    pub fn between_closed_aabb(box_: &Aabb) -> BetweenClosed {
        let block_pos = Self::containing(box_.min_x, box_.min_y, box_.min_z);
        let block_pos1 = Self::containing(box_.max_x, box_.max_y, box_.max_z);
        Self::between_closed(&block_pos, &block_pos1)
    }

    pub fn between_closed(pos1: &Self, pos2: &Self) -> BetweenClosed {
        Self::between_closed_coords(
            pos1.x.min(pos2.x),
            pos1.y.min(pos2.y),
            pos1.z.min(pos2.z),
            pos1.x.max(pos2.x),
            pos1.y.max(pos2.y),
            pos1.z.max(pos2.z),
        )
    }

    /// `betweenClosed(minX..maxZ)` — итератор.
    pub fn between_closed_coords(
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> BetweenClosed {
        BetweenClosed {
            min: Self::new(min_x, min_y, min_z),
            dims: (max_x - min_x + 1, max_y - min_y + 1, max_z - min_z + 1),
            index: 0,
            total: (max_x - min_x + 1) as i64 * (max_y - min_y + 1) as i64 * (max_z - min_z + 1) as i64,
        }
    }

    /// `betweenCornersInDirection(AABB, Vec3)`
    pub fn between_corners_in_direction_aabb(aabb: &Aabb, direction: &Vec3) -> BetweenCornersInDirection {
        let min_position = aabb.get_min_position();
        let max_position = aabb.get_max_position();
        Self::between_corners_in_direction_coords(
            mth::floor(min_position.x),
            mth::floor(min_position.y),
            mth::floor(min_position.z),
            mth::floor(max_position.x),
            mth::floor(max_position.y),
            mth::floor(max_position.z),
            direction,
        )
    }

    pub fn between_corners_in_direction(pos1: &Self, pos2: &Self, direction: &Vec3) -> BetweenCornersInDirection {
        Self::between_corners_in_direction_coords(
            pos1.x, pos1.y, pos1.z, pos2.x, pos2.y, pos2.z, direction,
        )
    }

    pub fn between_corners_in_direction_coords(
        x1: i32,
        y1: i32,
        z1: i32,
        x2: i32,
        y2: i32,
        z2: i32,
        direction: &Vec3,
    ) -> BetweenCornersInDirection {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let min_z = z1.min(z2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);
        let max_z = z1.max(z2);
        let i = max_x - min_x;
        let i1 = max_y - min_y;
        let i2 = max_z - min_z;
        let i3 = if direction.x >= 0.0 { min_x } else { max_x };
        let i4 = if direction.y >= 0.0 { min_y } else { max_y };
        let i5 = if direction.z >= 0.0 { min_z } else { max_z };
        let list = Self::axis_step_order_vec3(direction);
        let axis = list[0];
        let axis1 = list[1];
        let axis2 = list[2];
        let direction1 = if axis.choose_f64(direction.x, direction.y, direction.z) >= 0.0 {
            axis.get_positive()
        } else {
            axis.get_negative()
        };
        let direction2 = if axis1.choose_f64(direction.x, direction.y, direction.z) >= 0.0 {
            axis1.get_positive()
        } else {
            axis1.get_negative()
        };
        let direction3 = if axis2.choose_f64(direction.x, direction.y, direction.z) >= 0.0 {
            axis2.get_positive()
        } else {
            axis2.get_negative()
        };
        let i6 = axis.choose(i, i1, i2);
        let i7 = axis1.choose(i, i1, i2);
        let i8 = axis2.choose(i, i1, i2);
        BetweenCornersInDirection {
            cursor_x: i3,
            cursor_y: i4,
            cursor_z: i5,
            first_index: 0,
            second_index: 0,
            third_index: 0,
            end: false,
            i6,
            i7,
            i8,
            first_dir: (direction1.get_step_x(), direction1.get_step_y(), direction1.get_step_z()),
            second_dir: (direction2.get_step_x(), direction2.get_step_y(), direction2.get_step_z()),
            third_dir: (direction3.get_step_x(), direction3.get_step_y(), direction3.get_step_z()),
        }
    }

    fn axis_step_order_vec3(vector: &Vec3) -> [Axis; 3] {
        Direction::axis_step_order(vector)
    }

    /// `spiralAround`
    pub fn spiral_around(
        center: &Self,
        size: i32,
        rotation_direction: Direction,
        expansion_direction: Direction,
    ) -> SpiralAround {
        assert!(
            rotation_direction.get_axis() != expansion_direction.get_axis(),
            "The two directions cannot be on the same axis"
        );
        SpiralAround::new(*center, size, rotation_direction, expansion_direction)
    }

    /// `breadthFirstTraversal` — BFS по соседям.
    /// `children_getter` принимает позицию и callback для каждого ребёнка;
    /// `action` возвращает TraversalNodeStatus.
    pub fn breadth_first_traversal(
        start_pos: &Self,
        radius: i32,
        max_blocks: i32,
        mut children_getter: impl FnMut(&Self, &mut dyn FnMut(Self)),
        mut action: impl FnMut(&Self) -> TraversalNodeStatus,
    ) -> i32 {
        let mut queue: VecDeque<(Self, i32)> = VecDeque::new();
        let mut set = std::collections::HashSet::new();
        queue.push_back((*start_pos, 0));
        let mut i = 0;

        while let Some((block_pos, right)) = queue.pop_front() {
            let packed = block_pos.as_long();
            if set.insert(packed) {
                let status = action(&block_pos);
                if status != TraversalNodeStatus::SKIP {
                    if status == TraversalNodeStatus::STOP {
                        break;
                    }
                    i += 1;
                    if i >= max_blocks {
                        return i;
                    }
                    if right < radius {
                        children_getter(&block_pos, &mut |pos| queue.push_back((pos, right + 1)));
                    }
                }
            }
        }
        i
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraversalNodeStatus {
    ACCEPT,
    SKIP,
    STOP,
}

// ================================================================ MutableBlockPos

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct MutableBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl MutableBlockPos {
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_floor(x: f64, y: f64, z: f64) -> Self {
        Self::new(mth::floor(x), mth::floor(y), mth::floor(z))
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    /// `set(x,y,z)` — возвращает self для чейнинга.
    pub fn set(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.x = x;
        self.y = y;
        self.z = z;
        self
    }

    pub fn set_floor(&mut self, x: f64, y: f64, z: f64) -> &mut Self {
        self.set(mth::floor(x), mth::floor(y), mth::floor(z))
    }

    pub fn set_vec3i(&mut self, pos: &Vec3i) -> &mut Self {
        self.set(pos.x, pos.y, pos.z)
    }

    pub fn set_packed(&mut self, packed_pos: i64) -> &mut Self {
        self.set(BlockPos::get_x_packed(packed_pos), BlockPos::get_y_packed(packed_pos), BlockPos::get_z_packed(packed_pos))
    }

    /// `set(AxisCycle, ...)` — TODO: AxisCycle

    pub fn set_with_offset_vec3i(&mut self, pos: &Vec3i, direction: Direction) -> &mut Self {
        self.set(
            pos.x + direction.get_step_x(),
            pos.y + direction.get_step_y(),
            pos.z + direction.get_step_z(),
        )
    }

    pub fn set_with_offset(&mut self, pos: &Vec3i, offset_x: i32, offset_y: i32, offset_z: i32) -> &mut Self {
        self.set(pos.x + offset_x, pos.y + offset_y, pos.z + offset_z)
    }

    pub fn set_with_offset_vec(&mut self, pos: &Vec3i, offset: &Vec3i) -> &mut Self {
        self.set(pos.x + offset.x, pos.y + offset.y, pos.z + offset.z)
    }

    pub fn move_dir(&mut self, direction: Direction) -> &mut Self {
        self.move_dir_n(direction, 1)
    }

    pub fn move_dir_n(&mut self, direction: Direction, n: i32) -> &mut Self {
        self.set(
            self.x + direction.get_step_x() * n,
            self.y + direction.get_step_y() * n,
            self.z + direction.get_step_z() * n,
        )
    }

    pub fn r#move(&mut self, x: i32, y: i32, z: i32) -> &mut Self {
        self.set(self.x + x, self.y + y, self.z + z)
    }

    pub fn move_vec3i(&mut self, offset: &Vec3i) -> &mut Self {
        self.set(self.x + offset.x, self.y + offset.y, self.z + offset.z)
    }

    pub fn clamp_axis(&mut self, axis: Axis, min: i32, max: i32) -> &mut Self {
        match axis {
            Axis::X => self.set(mth::clamp_i32(self.x, min, max), self.y, self.z),
            Axis::Y => self.set(self.x, mth::clamp_i32(self.y, min, max), self.z),
            Axis::Z => self.set(self.x, self.y, mth::clamp_i32(self.z, min, max)),
        }
    }

    pub fn set_x(&mut self, x: i32) -> &mut Self {
        self.x = x;
        self
    }

    pub fn set_y(&mut self, y: i32) -> &mut Self {
        self.y = y;
        self
    }

    pub fn set_z(&mut self, z: i32) -> &mut Self {
        self.z = z;
        self
    }

    pub fn immutable(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }
}

// ================================================================ итераторы

/// `randomBetweenClosed` — ленивый итератор (владеет RNG через fork).
pub struct RandomBetweenClosed<'a> {
    random: Option<&'a mut dyn RandomSource>,
    owned: Option<Box<dyn RandomSource>>,
    counter: i32,
    min: BlockPos,
    dims: (i32, i32, i32),
}

impl<'a> RandomBetweenClosed<'a> {
    fn new_owned(
        random: Box<dyn RandomSource>,
        amount: i32,
        min_x: i32,
        min_y: i32,
        min_z: i32,
        max_x: i32,
        max_y: i32,
        max_z: i32,
    ) -> Self {
        Self {
            random: None,
            owned: Some(random),
            counter: amount,
            min: BlockPos::new(min_x, min_y, min_z),
            dims: (max_x - min_x + 1, max_y - min_y + 1, max_z - min_z + 1),
        }
    }

    fn rng(&mut self) -> &mut dyn RandomSource {
        if let Some(r) = self.random.as_deref_mut() {
            r
        } else {
            self.owned.as_mut().unwrap().as_mut()
        }
    }
}

impl<'a> Iterator for RandomBetweenClosed<'a> {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.counter <= 0 {
            return None;
        }
        let (i, i1, i2) = self.dims;
        let pos = BlockPos::new(
            self.min.x + self.rng().next_int_bound(i),
            self.min.y + self.rng().next_int_bound(i1),
            self.min.z + self.rng().next_int_bound(i2),
        );
        self.counter -= 1;
        Some(pos)
    }
}

/// `betweenClosed(minX..maxZ)` — итератор по индексу.
pub struct BetweenClosed {
    min: BlockPos,
    dims: (i32, i32, i32),
    index: i64,
    total: i64,
}

impl Iterator for BetweenClosed {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index as i64 == self.total {
            return None;
        }
        let i = self.dims.0 as i64;
        let i1 = self.dims.1 as i64;
        let index = self.index;
        self.index += 1;
        let i4 = index % i;
        let i5 = index / i;
        let i6 = i5 % i1;
        let i7 = i5 / i1;
        Some(BlockPos::new(
            self.min.x + i4 as i32,
            self.min.y + i6 as i32,
            self.min.z + i7 as i32,
        ))
    }
}

/// `withinManhattan` — оболочки Манхэттенского расстояния.
/// Порт computeNext 1:1, включая zMirror-трюк.
pub struct WithinManhattan {
    x1: i32,
    y1: i32,
    z: i32,
    x_size: i32,
    y_size: i32,
    z_size: i32,
    i: i32, // xSize + ySize + zSize
    cursor: MutableBlockPos,
    current_depth: i32,
    max_x: i32,
    max_y: i32,
    x: i32,
    y: i32,
    z_mirror: bool,
}

impl WithinManhattan {
    fn new(pos: BlockPos, x_size: i32, y_size: i32, z_size: i32) -> Self {
        Self {
            x1: pos.x,
            y1: pos.y,
            z: pos.z,
            x_size,
            y_size,
            z_size,
            i: x_size + y_size + z_size,
            cursor: MutableBlockPos::new(0, 0, 0),
            current_depth: 0,
            max_x: 0,
            max_y: 0,
            x: 0,
            y: 0,
            z_mirror: false,
        }
    }
}

impl Iterator for WithinManhattan {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.z_mirror {
            self.z_mirror = false;
            let z = self.z;
            let cz = self.cursor.z;
            self.cursor.set_z(z - (cz - z));
            return Some(self.cursor.immutable());
        }
        loop {
            if self.y > self.max_y {
                self.x += 1;
                if self.x > self.max_x {
                    self.current_depth += 1;
                    if self.current_depth > self.i {
                        return None;
                    }
                    self.max_x = self.x_size.min(self.current_depth);
                    self.x = -self.max_x;
                }
                self.max_y = self.y_size.min(self.current_depth - self.x.abs());
                self.y = -self.max_y;
            }

            let i1 = self.x;
            let i2 = self.y;
            let i3 = self.current_depth - i1.abs() - i2.abs();
            if i3 <= self.z_size {
                self.z_mirror = i3 != 0;
                self.cursor.set(self.x1 + i1, self.y1 + i2, self.z + i3);
                // Java for-loop инкрементит y после каждой итерации,
                // включая ту, что нашла позицию.
                self.y += 1;
                return Some(self.cursor.immutable());
            }
            self.y += 1;
        }
    }
}

/// `spiralAround` — итератор по спирали.
pub struct SpiralAround {
    directions: [Direction; 4],
    cursor: MutableBlockPos,
    legs: i32,
    leg: i32,
    leg_size: i32,
    leg_index: i32,
    last_x: i32,
    last_y: i32,
    last_z: i32,
}

impl SpiralAround {
    fn new(center: BlockPos, size: i32, rotation_direction: Direction, expansion_direction: Direction) -> Self {
        let directions = [
            rotation_direction,
            expansion_direction,
            rotation_direction.get_opposite(),
            expansion_direction.get_opposite(),
        ];
        let mut cursor = center.mutable();
        cursor.move_dir(expansion_direction);
        Self {
            directions,
            last_x: cursor.x,
            last_y: cursor.y,
            last_z: cursor.z,
            cursor,
            legs: 4 * size,
            leg: -1,
            leg_size: 0,
            leg_index: 0,
        }
    }
}

impl Iterator for SpiralAround {
    type Item = MutableBlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        let dir = self.directions[((self.leg + 4) % 4) as usize];
        self.cursor
            .set(self.last_x, self.last_y, self.last_z)
            .move_dir(dir);
        self.last_x = self.cursor.x;
        self.last_y = self.cursor.y;
        self.last_z = self.cursor.z;
        if self.leg_index >= self.leg_size {
            if self.leg >= self.legs {
                return None;
            }
            self.leg += 1;
            self.leg_index = 0;
            self.leg_size = self.leg / 2 + 1;
        }
        self.leg_index += 1;
        Some(self.cursor)
    }
}

/// `betweenCornersInDirection` — итератор по 3 вложенным циклам.
pub struct BetweenCornersInDirection {
    cursor_x: i32,
    cursor_y: i32,
    cursor_z: i32,
    first_index: i32,
    second_index: i32,
    third_index: i32,
    end: bool,
    i6: i32,
    i7: i32,
    i8: i32,
    first_dir: (i32, i32, i32),
    second_dir: (i32, i32, i32),
    third_dir: (i32, i32, i32),
}

impl Iterator for BetweenCornersInDirection {
    type Item = BlockPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }
        let pos = BlockPos::new(
            self.cursor_x
                + self.first_dir.0 * self.first_index
                + self.second_dir.0 * self.second_index
                + self.third_dir.0 * self.third_index,
            self.cursor_y
                + self.first_dir.1 * self.first_index
                + self.second_dir.1 * self.second_index
                + self.third_dir.1 * self.third_index,
            self.cursor_z
                + self.first_dir.2 * self.first_index
                + self.second_dir.2 * self.second_index
                + self.third_dir.2 * self.third_index,
        );
        if self.third_index < self.i8 {
            self.third_index += 1;
        } else if self.second_index < self.i7 {
            self.second_index += 1;
            self.third_index = 0;
        } else if self.first_index < self.i6 {
            self.first_index += 1;
            self.third_index = 0;
            self.second_index = 0;
        } else {
            self.end = true;
        }
        Some(pos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::random::single::SingleThreadedRandomSource;

    #[test]
    fn pack_unpack_roundtrip() {
        for (x, y, z) in [(0, 0, 0), (1, 64, -1), (-100, -60, 200), (33554431, 2047, -33554432), (123456, -2048, 789)] {
            let p = BlockPos::new(x, y, z);
            let q = BlockPos::of(p.as_long());
            assert_eq!(p, q, "roundtrip {p:?}");
        }
    }

    #[test]
    fn packed_field_extraction() {
        let p = BlockPos::new(5, 300, -7);
        let packed = p.as_long();
        assert_eq!(BlockPos::get_x_packed(packed), 5);
        assert_eq!(BlockPos::get_y_packed(packed), 300);
        assert_eq!(BlockPos::get_z_packed(packed), -7);
        assert_eq!(BlockPos::of(packed), p);
    }

    #[test]
    fn offset_packed_xyz_matches() {
        let p = BlockPos::new(10, 20, -30);
        let moved = BlockPos::offset_packed_xyz(p.as_long(), 1, -1, 2);
        assert_eq!(BlockPos::of(moved), BlockPos::new(11, 19, -28));
    }

    #[test]
    fn relative_directions() {
        let p = BlockPos::new(0, 64, 0);
        assert_eq!(p.relative(Direction::UP), BlockPos::new(0, 65, 0));
        assert_eq!(p.relative(Direction::DOWN), BlockPos::new(0, 63, 0));
        assert_eq!(p.relative(Direction::NORTH), BlockPos::new(0, 64, -1));
        assert_eq!(p.relative(Direction::SOUTH), BlockPos::new(0, 64, 1));
        assert_eq!(p.relative(Direction::WEST), BlockPos::new(-1, 64, 0));
        assert_eq!(p.relative(Direction::EAST), BlockPos::new(1, 64, 0));
        assert_eq!(p.relative_n(Direction::EAST, 5), BlockPos::new(5, 64, 0));
    }

    #[test]
    fn rotate_positions() {
        let p = BlockPos::new(3, 7, -2);
        assert_eq!(p.rotate(Rotation::NONE), p);
        assert_eq!(p.rotate(Rotation::CLOCKWISE_90), BlockPos::new(2, 7, 3));
        assert_eq!(p.rotate(Rotation::CLOCKWISE_180), BlockPos::new(-3, 7, 2));
        assert_eq!(p.rotate(Rotation::COUNTERCLOCKWISE_90), BlockPos::new(-2, 7, -3));
    }

    #[test]
    fn between_closed_count_and_order() {
        let min = BlockPos::new(0, 0, 0);
        let max = BlockPos::new(1, 1, 0);
        let v: Vec<BlockPos> = BlockPos::between_closed(&min, &max).collect();
        assert_eq!(v.len(), 4);
        // порядок Java: z внешний, потом y, потом x
        assert_eq!(v[0], BlockPos::new(0, 0, 0));
        assert_eq!(v[1], BlockPos::new(1, 0, 0));
        assert_eq!(v[2], BlockPos::new(0, 1, 0));
        assert_eq!(v[3], BlockPos::new(1, 1, 0));
    }

    #[test]
    fn random_between_closed_bounds() {
        let mut rng = SingleThreadedRandomSource::new(42);
        let v: Vec<BlockPos> = BlockPos::random_between_closed_vec(
            &mut rng, 100, 0, 0, 0, 10, 10, 10,
        );
        assert_eq!(v.len(), 100);
        for p in &v {
            assert!((0..=10).contains(&p.x) && (0..=10).contains(&p.y) && (0..=10).contains(&p.z));
        }
    }

    #[test]
    fn within_manhattan_center_first() {
        let center = BlockPos::new(0, 0, 0);
        let mut it = BlockPos::within_manhattan(&center, 1, 1, 1);
        assert_eq!(it.next(), Some(BlockPos::new(0, 0, 0)));
        // withinManhattan — обход box-оболочек по глубине Манхэттена,
        // для (1,1,1) покрывает весь куб 3x3x3 = 27 позиций
        let mut count = 1;
        let mut seen = std::collections::HashSet::new();
        seen.insert(BlockPos::new(0, 0, 0));
        while let Some(p) = it.next() {
            assert!(seen.insert(p), "дубликат {p:?}");
            count += 1;
        }
        assert_eq!(count, 27);
    }

    #[test]
    fn spiral_around_count() {
        let center = BlockPos::new(0, 0, 0);
        // legs = 4*size; суммарное число позиций — детерминированное,
        // сверено с Java-итератором (size=2 → 25).
        let v: Vec<MutableBlockPos> =
            BlockPos::spiral_around(&center, 2, Direction::NORTH, Direction::EAST).collect();
        assert_eq!(v.len(), 25);
        // первая позиция — сам центр (computeNext шагает от lastX/Y/Z
        // в направлении directions[3] = expansion.opposite)
        assert_eq!(v[0].immutable(), BlockPos::new(0, 0, 0));
    }

    #[test]
    fn mutable_block_pos_ops() {
        let mut p = MutableBlockPos::new(1, 2, 3);
        p.r#move(1, 0, -1).move_dir(Direction::UP);
        assert_eq!(p.immutable(), BlockPos::new(2, 3, 2));
        p.set_packed(BlockPos::new(-5, 100, 7).as_long());
        assert_eq!(p.immutable(), BlockPos::new(-5, 100, 7));
        p.clamp_axis(Axis::X, -10, 10);
        assert_eq!(p.x, -5);
        p.clamp_axis(Axis::X, -3, 10);
        assert_eq!(p.x, -3);
    }

    #[test]
    fn square_out_south_east() {
        let p = BlockPos::new(1, 2, 3);
        assert_eq!(
            BlockPos::square_out_south_east(&p),
            [
                BlockPos::new(1, 2, 3),
                BlockPos::new(1, 2, 4),
                BlockPos::new(2, 2, 3),
                BlockPos::new(2, 2, 4),
            ]
        );
    }
}
