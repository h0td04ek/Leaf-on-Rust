//! Перенос `net.minecraft.core.Direction` (+ Axis, AxisDirection, Plane).
//! Codec/StreamCodec и Entity-зависимые варианты (`orderedByNearest`,
//! `getFacingAxis`, `rotate(Matrix4fc)`) — TODO отдельной итерацией.

use crate::core::vec3i::Vec3i;
use crate::joml::{Quaternionf, Vector3f};
use crate::random::RandomSource;
use crate::util::mth;
use crate::world::phys::Vec3;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    pub const VALUES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

    pub fn name(&self) -> &'static str {
        match self {
            Axis::X => "x",
            Axis::Y => "y",
            Axis::Z => "z",
        }
    }

    /// `StringRepresentable.fromEnum`-совместимый поиск по имени.
    pub fn by_name(name: &str) -> Option<Axis> {
        Axis::VALUES.iter().copied().find(|a| a.name() == name)
    }

    pub fn is_vertical(&self) -> bool {
        *self == Axis::Y
    }

    pub fn is_horizontal(&self) -> bool {
        matches!(self, Axis::X | Axis::Z)
    }

    pub fn get_positive(&self) -> Direction {
        match self {
            Axis::X => Direction::EAST,
            Axis::Y => Direction::UP,
            Axis::Z => Direction::SOUTH,
        }
    }

    pub fn get_negative(&self) -> Direction {
        match self {
            Axis::X => Direction::WEST,
            Axis::Y => Direction::DOWN,
            Axis::Z => Direction::NORTH,
        }
    }

    pub fn get_directions(&self) -> [Direction; 2] {
        [self.get_positive(), self.get_negative()]
    }

    pub fn get_plane(&self) -> Plane {
        match self {
            Axis::X | Axis::Z => Plane::HORIZONTAL,
            Axis::Y => Plane::VERTICAL,
        }
    }

    pub fn choose(&self, x: i32, y: i32, z: i32) -> i32 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    pub fn choose_f64(&self, x: f64, y: f64, z: f64) -> f64 {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    pub fn choose_bool(&self, x: bool, y: bool, z: bool) -> bool {
        match self {
            Axis::X => x,
            Axis::Y => y,
            Axis::Z => z,
        }
    }

    pub fn get_random(random: &mut dyn RandomSource) -> Axis {
        Axis::VALUES[random.next_int_bound(Axis::VALUES.len() as i32) as usize]
    }
}

impl std::fmt::Display for Axis {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AxisDirection {
    POSITIVE,
    NEGATIVE,
}

impl AxisDirection {
    pub fn get_step(&self) -> i32 {
        match self {
            AxisDirection::POSITIVE => 1,
            AxisDirection::NEGATIVE => -1,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AxisDirection::POSITIVE => "Towards positive",
            AxisDirection::NEGATIVE => "Towards negative",
        }
    }

    pub fn opposite(&self) -> Self {
        match self {
            AxisDirection::POSITIVE => AxisDirection::NEGATIVE,
            AxisDirection::NEGATIVE => AxisDirection::POSITIVE,
        }
    }
}

impl std::fmt::Display for AxisDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Plane {
    HORIZONTAL,
    VERTICAL,
}

impl Plane {
    pub const FACES_HORIZONTAL: [Direction; 4] = [
        Direction::NORTH,
        Direction::EAST,
        Direction::SOUTH,
        Direction::WEST,
    ];
    pub const AXES_HORIZONTAL: [Axis; 2] = [Axis::X, Axis::Z];
    pub const FACES_VERTICAL: [Direction; 2] = [Direction::UP, Direction::DOWN];
    pub const AXES_VERTICAL: [Axis; 1] = [Axis::Y];

    pub fn faces(&self) -> &'static [Direction] {
        match self {
            Plane::HORIZONTAL => &Self::FACES_HORIZONTAL,
            Plane::VERTICAL => &Self::FACES_VERTICAL,
        }
    }

    pub fn axes(&self) -> &'static [Axis] {
        match self {
            Plane::HORIZONTAL => &Self::AXES_HORIZONTAL,
            Plane::VERTICAL => &Self::AXES_VERTICAL,
        }
    }

    pub fn length(&self) -> usize {
        self.faces().len()
    }

    pub fn get_random_direction(&self, random: &mut dyn RandomSource) -> Direction {
        let faces = self.faces();
        faces[random.next_int_bound(faces.len() as i32) as usize]
    }

    pub fn get_random_axis(&self, random: &mut dyn RandomSource) -> Axis {
        let axes = self.axes();
        axes[random.next_int_bound(axes.len() as i32) as usize]
    }

    pub fn shuffled_copy(&self, random: &mut dyn RandomSource) -> Vec<Direction> {
        let mut list = self.faces().to_vec();
        crate::util::shuffle(&mut list, random);
        list
    }

    pub fn test(&self, direction: Option<Direction>) -> bool {
        direction.is_some_and(|d| d.get_axis().get_plane() == *self)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction {
    DOWN,
    UP,
    NORTH,
    SOUTH,
    WEST,
    EAST,
}

impl Direction {
    /// Порядок объявления в Java enum — важен для data3d/data2d.
    pub const VALUES: [Direction; 6] = [
        Direction::DOWN,
        Direction::UP,
        Direction::NORTH,
        Direction::SOUTH,
        Direction::WEST,
        Direction::EAST,
    ];

    // (data3d, oppositeIndex, data2d, name, axisDirection, axis, normal)
    const META: [(i32, usize, i32, &'static str, AxisDirection, Axis, Vec3i); 6] = [
        (0, 1, -1, "down", AxisDirection::NEGATIVE, Axis::Y, Vec3i::new(0, -1, 0)),
        (1, 0, -1, "up", AxisDirection::POSITIVE, Axis::Y, Vec3i::new(0, 1, 0)),
        (2, 3, 2, "north", AxisDirection::NEGATIVE, Axis::Z, Vec3i::new(0, 0, -1)),
        (3, 2, 0, "south", AxisDirection::POSITIVE, Axis::Z, Vec3i::new(0, 0, 1)),
        (4, 5, 1, "west", AxisDirection::NEGATIVE, Axis::X, Vec3i::new(-1, 0, 0)),
        (5, 4, 3, "east", AxisDirection::POSITIVE, Axis::X, Vec3i::new(1, 0, 0)),
    ];

    const BY_2D_DATA: [Direction; 4] = [Direction::SOUTH, Direction::WEST, Direction::NORTH, Direction::EAST];

    fn idx(&self) -> usize {
        *self as usize
    }

    pub fn get_3d_data_value(&self) -> i32 {
        Self::META[self.idx()].0
    }

    pub fn get_2d_data_value(&self) -> i32 {
        Self::META[self.idx()].2
    }

    pub fn get_name(&self) -> &'static str {
        Self::META[self.idx()].3
    }

    pub fn get_axis_direction(&self) -> AxisDirection {
        Self::META[self.idx()].4
    }

    pub fn get_axis(&self) -> Axis {
        Self::META[self.idx()].5
    }

    pub fn get_unit_vec3i(&self) -> Vec3i {
        Self::META[self.idx()].6
    }

    pub fn get_unit_vec3(&self) -> Vec3 {
        let n = self.get_unit_vec3i();
        Vec3::new(n.x as f64, n.y as f64, n.z as f64)
    }

    pub fn get_unit_vec3f(&self) -> Vector3f {
        let n = self.get_unit_vec3i();
        Vector3f::new(n.x as f32, n.y as f32, n.z as f32)
    }

    pub fn get_step_x(&self) -> i32 {
        self.get_unit_vec3i().x
    }

    pub fn get_step_y(&self) -> i32 {
        self.get_unit_vec3i().y
    }

    pub fn get_step_z(&self) -> i32 {
        self.get_unit_vec3i().z
    }

    pub fn step(&self) -> Vector3f {
        self.get_unit_vec3f()
    }

    pub fn get_opposite(&self) -> Direction {
        Self::VALUES[Self::META[self.idx()].1]
    }

    pub fn get_clock_wise(&self) -> Direction {
        match self {
            Direction::NORTH => Direction::EAST,
            Direction::SOUTH => Direction::WEST,
            Direction::WEST => Direction::NORTH,
            Direction::EAST => Direction::SOUTH,
            _ => panic!("Unable to get Y-rotated facing of {:?}", self),
        }
    }

    fn clock_wise_x(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::SOUTH,
            Direction::UP => Direction::NORTH,
            Direction::NORTH => Direction::DOWN,
            Direction::SOUTH => Direction::UP,
            _ => panic!("Unable to get X-rotated facing of {:?}", self),
        }
    }

    fn counter_clock_wise_x(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::NORTH,
            Direction::UP => Direction::SOUTH,
            Direction::NORTH => Direction::UP,
            Direction::SOUTH => Direction::DOWN,
            _ => panic!("Unable to get X-rotated facing of {:?}", self),
        }
    }

    fn clock_wise_z(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::WEST,
            Direction::UP => Direction::EAST,
            Direction::WEST => Direction::UP,
            Direction::EAST => Direction::DOWN,
            _ => panic!("Unable to get Z-rotated facing of {:?}", self),
        }
    }

    fn counter_clock_wise_z(&self) -> Direction {
        match self {
            Direction::DOWN => Direction::EAST,
            Direction::UP => Direction::WEST,
            Direction::WEST => Direction::DOWN,
            Direction::EAST => Direction::UP,
            _ => panic!("Unable to get Z-rotated facing of {:?}", self),
        }
    }

    pub fn get_counter_clock_wise(&self) -> Direction {
        match self {
            Direction::NORTH => Direction::WEST,
            Direction::SOUTH => Direction::EAST,
            Direction::WEST => Direction::SOUTH,
            Direction::EAST => Direction::NORTH,
            _ => panic!("Unable to get CCW facing of {:?}", self),
        }
    }

    pub fn get_clock_wise_axis(&self, axis: Axis) -> Direction {
        match axis {
            Axis::X => {
                if *self != Direction::WEST && *self != Direction::EAST {
                    self.clock_wise_x()
                } else {
                    *self
                }
            }
            Axis::Y => {
                if *self != Direction::UP && *self != Direction::DOWN {
                    self.get_clock_wise()
                } else {
                    *self
                }
            }
            Axis::Z => {
                if *self != Direction::NORTH && *self != Direction::SOUTH {
                    self.clock_wise_z()
                } else {
                    *self
                }
            }
        }
    }

    pub fn get_counter_clock_wise_axis(&self, axis: Axis) -> Direction {
        match axis {
            Axis::X => {
                if *self != Direction::WEST && *self != Direction::EAST {
                    self.counter_clock_wise_x()
                } else {
                    *self
                }
            }
            Axis::Y => {
                if *self != Direction::UP && *self != Direction::DOWN {
                    self.get_counter_clock_wise()
                } else {
                    *self
                }
            }
            Axis::Z => {
                if *self != Direction::NORTH && *self != Direction::SOUTH {
                    self.counter_clock_wise_z()
                } else {
                    *self
                }
            }
        }
    }

    /// `StringRepresentable.getSerializedName`
    pub fn get_serialized_name(&self) -> &'static str {
        self.get_name()
    }

    /// `CODEC.byName` — поиск по имени без учёта регистра? В Java EnumCodec
    /// сравнивает getSerializedName точно; byName возвращает Option.
    pub fn by_name(name: &str) -> Option<Direction> {
        Self::VALUES.iter().copied().find(|d| d.get_name() == name)
    }

    pub fn from_3d_data_value(index: i32) -> Direction {
        let len = Self::VALUES.len() as i32;
        Self::VALUES[mth::abs_i32(index % len) as usize]
    }

    pub fn from_2d_data_value(horizontal_index: i32) -> Direction {
        let len = Self::BY_2D_DATA.len() as i32;
        Self::BY_2D_DATA[mth::abs_i32(horizontal_index % len) as usize]
    }

    pub fn from_y_rot(y_rot: f64) -> Direction {
        Self::from_2d_data_value(mth::floor(y_rot / 90.0 + 0.5) & 3)
    }

    pub fn from_axis_and_direction(axis: Axis, direction: AxisDirection) -> Direction {
        match axis {
            Axis::X => {
                if direction == AxisDirection::POSITIVE { Direction::EAST } else { Direction::WEST }
            }
            Axis::Y => {
                if direction == AxisDirection::POSITIVE { Direction::UP } else { Direction::DOWN }
            }
            Axis::Z => {
                if direction == AxisDirection::POSITIVE { Direction::SOUTH } else { Direction::NORTH }
            }
        }
    }

    /// `getYRot` — только горизонтальные направления.
    pub fn get_y_rot_static(direction: Direction) -> f32 {
        match direction {
            Direction::NORTH => 180.0,
            Direction::SOUTH => 0.0,
            Direction::WEST => 90.0,
            Direction::EAST => -90.0,
            _ => panic!("No y-Rot for vertical axis: {:?}", direction),
        }
    }

    /// `toYRot`
    pub fn to_y_rot(&self) -> i32 {
        (self.get_2d_data_value() & 3) * 90
    }

    pub fn get_random(random: &mut dyn RandomSource) -> Direction {
        Self::VALUES[random.next_int_bound(Self::VALUES.len() as i32) as usize]
    }

    pub fn get_approximate_nearest(x: f32, y: f32, z: f32) -> Direction {
        let mut direction = Direction::NORTH;
        let mut f = f32::MIN;
        for d in Self::VALUES {
            let n = d.get_unit_vec3i();
            let f1 = x * n.x as f32 + y * n.y as f32 + z * n.z as f32;
            if f1 > f {
                f = f1;
                direction = d;
            }
        }
        direction
    }

    pub fn get_approximate_nearest_vec(&self_vec: &Vec3) -> Direction {
        Self::get_approximate_nearest(self_vec.x as f32, self_vec.y as f32, self_vec.z as f32)
    }

    pub fn get_nearest(x: i32, y: i32, z: i32, default_value: Option<Direction>) -> Option<Direction> {
        let abs = x.abs();
        let abs1 = y.abs();
        let abs2 = z.abs();
        if abs > abs2 && abs > abs1 {
            Some(if x < 0 { Direction::WEST } else { Direction::EAST })
        } else if abs2 > abs && abs2 > abs1 {
            Some(if z < 0 { Direction::NORTH } else { Direction::SOUTH })
        } else if abs1 > abs && abs1 > abs2 {
            Some(if y < 0 { Direction::DOWN } else { Direction::UP })
        } else {
            default_value
        }
    }

    pub fn get_nearest_vec3i(vector: &Vec3i, default_value: Option<Direction>) -> Option<Direction> {
        Self::get_nearest(vector.x, vector.y, vector.z, default_value)
    }

    /// `axisStepOrder` — порядок обхода осей для skew-векторов.
    pub fn axis_step_order(vector: &Vec3) -> [Axis; 3] {
        if vector.x.abs() < vector.z.abs() {
            [Axis::Y, Axis::Z, Axis::X]
        } else {
            [Axis::Y, Axis::X, Axis::Z]
        }
    }

    /// `isFacingAngle`
    pub fn is_facing_angle(&self, angle: f32) -> bool {
        let f = angle as f64 * (std::f64::consts::PI / 180.0);
        let f1 = -mth::sin(f);
        let cos = mth::cos(f);
        let n = self.get_unit_vec3i();
        (n.x as f32) * f1 + (n.z as f32) * cos > 0.0
    }

    /// `getRotation` — кватернион ориентации направления (Paper-кэш).
    pub fn get_rotation(&self) -> Quaternionf {
        let mut q = Quaternionf::default();
        match self {
            Direction::DOWN => {
                q.rotation_x(std::f32::consts::PI);
            }
            Direction::UP => {}
            Direction::NORTH => {
                q.rotation_xyz(std::f32::consts::FRAC_PI_2, 0.0, std::f32::consts::PI);
            }
            Direction::SOUTH => {
                q.rotation_x(std::f32::consts::FRAC_PI_2);
            }
            Direction::WEST => {
                q.rotation_xyz(std::f32::consts::FRAC_PI_2, 0.0, std::f32::consts::FRAC_PI_2);
            }
            Direction::EAST => {
                q.rotation_xyz(std::f32::consts::FRAC_PI_2, 0.0, -std::f32::consts::FRAC_PI_2);
            }
        }
        q
    }

    pub fn get(&self, axis_direction: AxisDirection, axis: Axis) -> Direction {
        for d in Self::VALUES {
            if d.get_axis_direction() == axis_direction && d.get_axis() == axis {
                return d;
            }
        }
        panic!("No such direction: {:?} {:?}", axis_direction, axis);
    }

    /// `allShuffled` — все направления в случайном порядке.
    pub fn all_shuffled(random: &mut dyn RandomSource) -> Vec<Direction> {
        let mut list: Vec<Direction> = Self::VALUES.to_vec();
        crate::util::shuffle(&mut list, random);
        list
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.get_name())
    }
}

/// `net.minecraft.core.Direction8` — 8 горизонтальных направлений
/// (4 основных + 4 диагональных).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Direction8 {
    /// `NORTH`.
    North,
    /// `NORTH_EAST`.
    NorthEast,
    /// `EAST`.
    East,
    /// `SOUTH_EAST`.
    SouthEast,
    /// `SOUTH`.
    South,
    /// `SOUTH_WEST`.
    SouthWest,
    /// `WEST`.
    West,
    /// `NORTH_WEST`.
    NorthWest,
}

impl Direction8 {
    /// Все значения в порядке объявления.
    pub const VALUES: [Direction8; 8] = [
        Direction8::North,
        Direction8::NorthEast,
        Direction8::East,
        Direction8::SouthEast,
        Direction8::South,
        Direction8::SouthWest,
        Direction8::West,
        Direction8::NorthWest,
    ];

    fn directions(&self) -> &'static [Direction] {
        match self {
            Direction8::North => &[Direction::NORTH],
            Direction8::NorthEast => &[Direction::NORTH, Direction::EAST],
            Direction8::East => &[Direction::EAST],
            Direction8::SouthEast => &[Direction::SOUTH, Direction::EAST],
            Direction8::South => &[Direction::SOUTH],
            Direction8::SouthWest => &[Direction::SOUTH, Direction::WEST],
            Direction8::West => &[Direction::WEST],
            Direction8::NorthWest => &[Direction::NORTH, Direction::WEST],
        }
    }

    /// `getDirections`.
    pub fn get_directions(&self) -> &'static [Direction] {
        self.directions()
    }

    /// `getStepX` — сумма stepX составляющих направлений.
    pub fn get_step_x(&self) -> i32 {
        self.directions().iter().map(|d| d.get_step_x()).sum()
    }

    /// `getStepZ`.
    pub fn get_step_z(&self) -> i32 {
        self.directions().iter().map(|d| d.get_step_z()).sum()
    }
}
