//! Перенос `net.minecraft.core.Vec3i` — целочисленный 3D-вектор.
//! Codec/StreamCodec и LevelHeightAccessor-зависимые методы — TODO.

use crate::core::direction::{Axis, Direction};
use crate::util::mth;
use std::cmp::Ordering;
use std::fmt;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// `org.dreeam.leaf.util.Vec3iHasher.hash`
pub fn vec3i_hash(x: i32, y: i32, z: i32) -> i32 {
    const INT_PHI: i32 = 0x9E37_79B9u32 as i32;
    let hash = (x.wrapping_mul(31337) ^ y.wrapping_mul(961) ^ z).wrapping_mul(INT_PHI);
    hash ^ (hash >> 16)
}

impl Vec3i {
    pub const ZERO: Self = Self { x: 0, y: 0, z: 0 };

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub const fn get_x(&self) -> i32 {
        self.x
    }

    pub const fn get_y(&self) -> i32 {
        self.y
    }

    pub const fn get_z(&self) -> i32 {
        self.z
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        if dx == 0 && dy == 0 && dz == 0 {
            *self
        } else {
            Self::new(self.x + dx, self.y + dy, self.z + dz)
        }
    }

    pub fn offset_vec(&self, vector: &Self) -> Self {
        self.offset(vector.x, vector.y, vector.z)
    }

    pub fn subtract(&self, vector: &Self) -> Self {
        self.offset(-vector.x, -vector.y, -vector.z)
    }

    pub fn multiply(&self, scalar: i32) -> Self {
        if scalar == 1 {
            *self
        } else if scalar == 0 {
            Self::ZERO
        } else {
            Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
        }
    }

    pub fn multiply_xyz(&self, factor_x: i32, factor_y: i32, factor_z: i32) -> Self {
        Self::new(self.x * factor_x, self.y * factor_y, self.z * factor_z)
    }

    pub fn above(&self) -> Self {
        self.above_n(1)
    }

    pub fn above_n(&self, distance: i32) -> Self {
        self.relative_n(Direction::UP, distance)
    }

    pub fn below(&self) -> Self {
        self.below_n(1)
    }

    pub fn below_n(&self, distance: i32) -> Self {
        self.relative_n(Direction::DOWN, distance)
    }

    pub fn north(&self) -> Self {
        self.north_n(1)
    }

    pub fn north_n(&self, distance: i32) -> Self {
        self.relative_n(Direction::NORTH, distance)
    }

    pub fn south(&self) -> Self {
        self.south_n(1)
    }

    pub fn south_n(&self, distance: i32) -> Self {
        self.relative_n(Direction::SOUTH, distance)
    }

    pub fn west(&self) -> Self {
        self.west_n(1)
    }

    pub fn west_n(&self, distance: i32) -> Self {
        self.relative_n(Direction::WEST, distance)
    }

    pub fn east(&self) -> Self {
        self.east_n(1)
    }

    pub fn east_n(&self, distance: i32) -> Self {
        self.relative_n(Direction::EAST, distance)
    }

    pub fn relative(&self, direction: Direction) -> Self {
        self.relative_n(direction, 1)
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

    pub fn cross(&self, vector: &Self) -> Self {
        Self::new(
            self.y * vector.z - self.z * vector.y,
            self.z * vector.x - self.x * vector.z,
            self.x * vector.y - self.y * vector.x,
        )
    }

    pub fn closer_than(&self, vector: &Self, distance: f64) -> bool {
        self.dist_sqr(vector) < mth::square_f64(distance)
    }

    pub fn dist_sqr(&self, vector: &Self) -> f64 {
        self.dist_to_low_corner_sqr(vector.x as f64, vector.y as f64, vector.z as f64)
    }

    pub fn dist_to_center_sqr(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = self.x as f64 + 0.5 - x;
        let d1 = self.y as f64 + 0.5 - y;
        let d2 = self.z as f64 + 0.5 - z;
        d * d + d1 * d1 + d2 * d2
    }

    pub fn dist_to_low_corner_sqr(&self, x: f64, y: f64, z: f64) -> f64 {
        let d = self.x as f64 - x;
        let d1 = self.y as f64 - y;
        let d2 = self.z as f64 - z;
        d * d + d1 * d1 + d2 * d2
    }

    pub fn dist_man_hattan(&self, vector: &Self) -> i32 {
        ((vector.x - self.x).abs() + (vector.y - self.y).abs() + (vector.z - self.z).abs()) as i32
    }

    pub fn dist_chessboard(&self, vector: &Self) -> i32 {
        (self.x - vector.x)
            .abs()
            .max((self.y - vector.y).abs())
            .max((self.z - vector.z).abs())
    }

    pub fn get(&self, axis: Axis) -> i32 {
        axis.choose(self.x, self.y, self.z)
    }
}

impl Ord for Vec3i {
    /// Java `compareTo`: сначала Y, при равенстве Z, затем X.
    fn cmp(&self, other: &Self) -> Ordering {
        self.y
            .cmp(&other.y)
            .then_with(|| self.z.cmp(&other.z))
            .then_with(|| self.x.cmp(&other.x))
    }
}

impl PartialOrd for Vec3i {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Vec3i {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vec3i{{x={}, y={}, z={}}}", self.x, self.y, self.z)
    }
}
