//! Перенос `net.minecraft.world.phys.AABB`.
//! BlockPos/BoundingBox/BlockHitResult-зависимые члены — TODO
//! (перенесём вместе с core.BlockPos / world.phys.BlockHitResult).

use crate::core::direction::Direction;
use crate::joml::Vector3f;
use crate::util::mth;
use crate::world::phys::Vec3;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl Aabb {
    /// `new AABB(x1..z2)` — Java присваивает напрямую, без сортировки.
    pub fn new(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self {
            min_x: x1,
            min_y: y1,
            min_z: z1,
            max_x: x2,
            max_y: y2,
            max_z: z2,
        }
    }

    /// Вариант с гарантированной сортировкой min/max (удобство, в Java нет).
    pub fn sorted(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self::new(
            x1.min(x2),
            y1.min(y2),
            z1.min(z2),
            x1.max(x2),
            y1.max(y2),
            z1.max(z2),
        )
    }

    pub fn from_corners_unnormalized(x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64) -> Self {
        Self::new(x1, y1, z1, x2, y2, z2)
    }

    pub fn from_vec3(start: &Vec3, end: &Vec3) -> Self {
        Self::from_corners_unnormalized(start.x, start.y, start.z, end.x, end.y, end.z)
    }

    pub fn unit_cube_from_lower_corner(vector: &Vec3) -> Self {
        Self::from_corners_unnormalized(
            vector.x,
            vector.y,
            vector.z,
            vector.x + 1.0,
            vector.y + 1.0,
            vector.z + 1.0,
        )
    }

    /// `setMinX` и прочие — возвращают изменённую копию.
    pub fn set_min_x(&self, min_x: f64) -> Self {
        Self::from_corners_unnormalized(min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z)
    }

    pub fn set_min_y(&self, min_y: f64) -> Self {
        Self::from_corners_unnormalized(self.min_x, min_y, self.min_z, self.max_x, self.max_y, self.max_z)
    }

    pub fn set_min_z(&self, min_z: f64) -> Self {
        Self::from_corners_unnormalized(self.min_x, self.min_y, min_z, self.max_x, self.max_y, self.max_z)
    }

    pub fn set_max_x(&self, max_x: f64) -> Self {
        Self::from_corners_unnormalized(self.min_x, self.min_y, self.min_z, max_x, self.max_y, self.max_z)
    }

    pub fn set_max_y(&self, max_y: f64) -> Self {
        Self::from_corners_unnormalized(self.min_x, self.min_y, self.min_z, self.max_x, max_y, self.max_z)
    }

    pub fn set_max_z(&self, max_z: f64) -> Self {
        Self::from_corners_unnormalized(self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, max_z)
    }

    /// `min(Direction.Axis)` / `max(Direction.Axis)`
    pub fn min_axis(&self, axis: crate::core::direction::Axis) -> f64 {
        axis.choose_f64(self.min_x, self.min_y, self.min_z)
    }

    pub fn max_axis(&self, axis: crate::core::direction::Axis) -> f64 {
        axis.choose_f64(self.max_x, self.max_y, self.max_z)
    }

    /// Java `equals` — `Double.compare` по всем шести координатам
    /// (NaN == NaN, -0.0 != 0.0) → `f64::total_cmp`.
    pub fn java_equals(&self, other: &Self) -> bool {
        self.min_x.total_cmp(&other.min_x) == std::cmp::Ordering::Equal
            && self.min_y.total_cmp(&other.min_y) == std::cmp::Ordering::Equal
            && self.min_z.total_cmp(&other.min_z) == std::cmp::Ordering::Equal
            && self.max_x.total_cmp(&other.max_x) == std::cmp::Ordering::Equal
            && self.max_y.total_cmp(&other.max_y) == std::cmp::Ordering::Equal
            && self.max_z.total_cmp(&other.max_z) == std::cmp::Ordering::Equal
    }

    pub fn contract(&self, x: f64, y: f64, z: f64) -> Self {
        let (mut d, mut d1, mut d2) = (self.min_x, self.min_y, self.min_z);
        let (mut d3, mut d4, mut d5) = (self.max_x, self.max_y, self.max_z);
        if x < 0.0 {
            d -= x;
        } else if x > 0.0 {
            d3 -= x;
        }
        if y < 0.0 {
            d1 -= y;
        } else if y > 0.0 {
            d4 -= y;
        }
        if z < 0.0 {
            d2 -= z;
        } else if z > 0.0 {
            d5 -= z;
        }
        Self::from_corners_unnormalized(d, d1, d2, d3, d4, d5)
    }

    pub fn expand_towards_vec(&self, vector: &Vec3) -> Self {
        self.expand_towards(vector.x, vector.y, vector.z)
    }

    pub fn expand_towards(&self, x: f64, y: f64, z: f64) -> Self {
        let (mut d, mut d1, mut d2) = (self.min_x, self.min_y, self.min_z);
        let (mut d3, mut d4, mut d5) = (self.max_x, self.max_y, self.max_z);
        if x < 0.0 {
            d += x;
        } else if x > 0.0 {
            d3 += x;
        }
        if y < 0.0 {
            d1 += y;
        } else if y > 0.0 {
            d4 += y;
        }
        if z < 0.0 {
            d2 += z;
        } else if z > 0.0 {
            d5 += z;
        }
        Self::from_corners_unnormalized(d, d1, d2, d3, d4, d5)
    }

    pub fn inflate(&self, x: f64, y: f64, z: f64) -> Self {
        Self::from_corners_unnormalized(
            self.min_x - x,
            self.min_y - y,
            self.min_z - z,
            self.max_x + x,
            self.max_y + y,
            self.max_z + z,
        )
    }

    pub fn inflate_all(&self, value: f64) -> Self {
        self.inflate(value, value, value)
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self::from_corners_unnormalized(
            self.min_x.max(other.min_x),
            self.min_y.max(other.min_y),
            self.min_z.max(other.min_z),
            self.max_x.min(other.max_x),
            self.max_y.min(other.max_y),
            self.max_z.min(other.max_z),
        )
    }

    pub fn minmax(&self, other: &Self) -> Self {
        Self::from_corners_unnormalized(
            self.min_x.min(other.min_x),
            self.min_y.min(other.min_y),
            self.min_z.min(other.min_z),
            self.max_x.max(other.max_x),
            self.max_y.max(other.max_y),
            self.max_z.max(other.max_z),
        )
    }

    pub fn move_xyz(&self, x: f64, y: f64, z: f64) -> Self {
        Self::from_corners_unnormalized(
            self.min_x + x,
            self.min_y + y,
            self.min_z + z,
            self.max_x + x,
            self.max_y + y,
            self.max_z + z,
        )
    }

    pub fn move_vec(&self, vec: &Vec3) -> Self {
        self.move_xyz(vec.x, vec.y, vec.z)
    }

    pub fn move_vec3f(&self, vec: &Vector3f) -> Self {
        self.move_xyz(vec.x as f64, vec.y as f64, vec.z as f64)
    }

    pub fn intersects(&self, other: &Self) -> bool {
        self.intersects_coords(
            other.min_x, other.min_y, other.min_z, other.max_x, other.max_y, other.max_z,
        )
    }

    pub fn intersects_coords(
        &self,
        x1: f64,
        y1: f64,
        z1: f64,
        x2: f64,
        y2: f64,
        z2: f64,
    ) -> bool {
        self.min_x < x2 && self.max_x > x1 && self.min_y < y2 && self.max_y > y1 && self.min_z < z2
            && self.max_z > z1
    }

    /// `intersects(Vec3 min, Vec3 max)`
    pub fn intersects_vec_range(&self, min: &Vec3, max: &Vec3) -> bool {
        self.intersects_coords(
            min.x.min(max.x),
            min.y.min(max.y),
            min.z.min(max.z),
            min.x.max(max.x),
            min.y.max(max.y),
            min.z.max(max.z),
        )
    }

    /// `intersects(BlockPos)` — TODO: core::BlockPos

    pub fn contains_vec(&self, vec: &Vec3) -> bool {
        self.contains(vec.x, vec.y, vec.z)
    }

    pub fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        x >= self.min_x && x < self.max_x && y >= self.min_y && y < self.max_y && z >= self.min_z
            && z < self.max_z
    }

    pub fn get_size(&self) -> f64 {
        (self.get_xsize() + self.get_ysize() + self.get_zsize()) / 3.0
    }

    pub fn get_xsize(&self) -> f64 {
        self.max_x - self.min_x
    }

    pub fn get_ysize(&self) -> f64 {
        self.max_y - self.min_y
    }

    pub fn get_zsize(&self) -> f64 {
        self.max_z - self.min_z
    }

    pub fn deflate(&self, x: f64, y: f64, z: f64) -> Self {
        self.inflate(-x, -y, -z)
    }

    pub fn deflate_all(&self, value: f64) -> Self {
        self.inflate(-value, -value, -value)
    }

    pub fn clip(&self, from: &Vec3, to: &Vec3) -> Option<Vec3> {
        Self::clip_coords(
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z, from, to,
        )
    }

    pub fn clip_coords(
        min_x: f64,
        min_y: f64,
        min_z: f64,
        max_x: f64,
        max_y: f64,
        max_z: f64,
        from: &Vec3,
        to: &Vec3,
    ) -> Option<Vec3> {
        let mut min_distance = [1.0f64];
        let d = to.x - from.x;
        let d1 = to.y - from.y;
        let d2 = to.z - from.z;
        let direction = get_direction_coords(
            min_x, min_y, min_z, max_x, max_y, max_z, from, &mut min_distance, None, d, d1, d2,
        );
        if direction.is_none() {
            return None;
        }
        let d3 = min_distance[0];
        Some(from.add(d3 * d, d3 * d1, d3 * d2))
    }

    /// Paper-оптимизированный `collidedAlongVector(Vec3, AABB)` (Leaf).
    pub fn collided_along_vector(&self, vector: &Vec3, aabb: &Self) -> bool {
        let center = self.get_center();
        let vec3 = center.add(vector.x, vector.y, vector.z);
        let aabb1 = aabb.inflate(
            self.get_xsize() * 0.5 - 1.0e-7,
            self.get_ysize() * 0.5 - 1.0e-7,
            self.get_zsize() * 0.5 - 1.0e-7,
        );
        aabb1.contains_vec(&vec3) || aabb1.contains_vec(&center) || aabb1.clip(&center, &vec3).is_some()
    }

    /// `collidedAlongVector(Vec3, List<AABB>)` — Leaf-вариант.
    pub fn collided_along_vector_list(&self, vector: &Vec3, boxes: &[Self]) -> bool {
        let center = self.get_center();
        let vec3 = center.add(vector.x, vector.y, vector.z);
        for aabb in boxes {
            let aabb1 = aabb.inflate(
                self.get_xsize() * 0.5 - 1.0e-7,
                self.get_ysize() * 0.5 - 1.0e-7,
                self.get_zsize() * 0.5 - 1.0e-7,
            );
            if aabb1.contains_vec(&vec3) || aabb1.contains_vec(&center) {
                return true;
            }
            if aabb1.clip(&center, &vec3).is_some() {
                return true;
            }
        }
        false
    }

    pub fn distance_to_sqr_vec(&self, vec: &Vec3) -> f64 {
        let max = (self.min_x - vec.x).max(vec.x - self.max_x).max(0.0);
        let max1 = (self.min_y - vec.y).max(vec.y - self.max_y).max(0.0);
        let max2 = (self.min_z - vec.z).max(vec.z - self.max_z).max(0.0);
        mth::length_squared_3d(max, max1, max2)
    }

    pub fn distance_to_sqr_aabb(&self, box_: &Self) -> f64 {
        let max = (self.min_x - box_.max_x).max(box_.min_x - self.max_x).max(0.0);
        let max1 = (self.min_y - box_.max_y).max(box_.min_y - self.max_y).max(0.0);
        let max2 = (self.min_z - box_.max_z).max(box_.min_z - self.max_z).max(0.0);
        mth::length_squared_3d(max, max1, max2)
    }

    pub fn has_nan(&self) -> bool {
        self.min_x.is_nan()
            || self.min_y.is_nan()
            || self.min_z.is_nan()
            || self.max_x.is_nan()
            || self.max_y.is_nan()
            || self.max_z.is_nan()
    }

    pub fn get_center(&self) -> Vec3 {
        Vec3::new(
            mth::lerp_f64(0.5, self.min_x, self.max_x),
            mth::lerp_f64(0.5, self.min_y, self.max_y),
            mth::lerp_f64(0.5, self.min_z, self.max_z),
        )
    }

    pub fn get_bottom_center(&self) -> Vec3 {
        Vec3::new(
            mth::lerp_f64(0.5, self.min_x, self.max_x),
            self.min_y,
            mth::lerp_f64(0.5, self.min_z, self.max_z),
        )
    }

    pub fn get_min_position(&self) -> Vec3 {
        Vec3::new(self.min_x, self.min_y, self.min_z)
    }

    pub fn get_max_position(&self) -> Vec3 {
        Vec3::new(self.max_x, self.max_y, self.max_z)
    }

    pub fn of_size(center: &Vec3, x_size: f64, y_size: f64, z_size: f64) -> Self {
        Self::from_corners_unnormalized(
            center.x - x_size / 2.0,
            center.y - y_size / 2.0,
            center.z - z_size / 2.0,
            center.x + x_size / 2.0,
            center.y + y_size / 2.0,
            center.z + z_size / 2.0,
        )
    }

    /// Purpur/Tuinity `offsetY`
    pub fn offset_y(&self, dy: f64) -> Self {
        Self::from_corners_unnormalized(self.min_x, self.min_y + dy, self.min_z, self.max_x, self.max_y + dy, self.max_z)
    }
}

/// Paper-оптимизированная `getDirection` (public в Paper).
#[allow(clippy::too_many_arguments)]
pub fn get_direction(
    aabb: &Aabb,
    start: &Vec3,
    min_distance: &mut [f64; 1],
    facing: Option<Direction>,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
) -> Option<Direction> {
    get_direction_coords(
        aabb.min_x,
        aabb.min_y,
        aabb.min_z,
        aabb.max_x,
        aabb.max_y,
        aabb.max_z,
        start,
        min_distance,
        facing,
        delta_x,
        delta_y,
        delta_z,
    )
}

#[allow(clippy::too_many_arguments)]
fn get_direction_coords(
    min_x: f64,
    min_y: f64,
    min_z: f64,
    max_x: f64,
    max_y: f64,
    max_z: f64,
    start: &Vec3,
    min_distance: &mut [f64; 1],
    mut facing: Option<Direction>,
    delta_x: f64,
    delta_y: f64,
    delta_z: f64,
) -> Option<Direction> {
    if delta_x > 1.0e-7 {
        facing = clip_point(
            min_distance, facing, delta_x, delta_y, delta_z, min_x, min_y, max_y, min_z, max_z,
            Direction::WEST, start.x, start.y, start.z,
        );
    } else if delta_x < -1.0e-7 {
        facing = clip_point(
            min_distance, facing, delta_x, delta_y, delta_z, max_x, min_y, max_y, min_z, max_z,
            Direction::EAST, start.x, start.y, start.z,
        );
    }

    if delta_y > 1.0e-7 {
        facing = clip_point(
            min_distance, facing, delta_y, delta_z, delta_x, min_y, min_z, max_z, min_x, max_x,
            Direction::DOWN, start.y, start.z, start.x,
        );
    } else if delta_y < -1.0e-7 {
        facing = clip_point(
            min_distance, facing, delta_y, delta_z, delta_x, max_y, min_z, max_z, min_x, max_x,
            Direction::UP, start.y, start.z, start.x,
        );
    }

    if delta_z > 1.0e-7 {
        facing = clip_point(
            min_distance, facing, delta_z, delta_x, delta_y, min_z, min_x, max_x, min_y, max_y,
            Direction::NORTH, start.z, start.x, start.y,
        );
    } else if delta_z < -1.0e-7 {
        facing = clip_point(
            min_distance, facing, delta_z, delta_x, delta_y, max_z, min_x, max_x, min_y, max_y,
            Direction::SOUTH, start.z, start.x, start.y,
        );
    }

    facing
}

#[allow(clippy::too_many_arguments)]
fn clip_point(
    min_distance: &mut [f64; 1],
    prev_direction: Option<Direction>,
    distance_side: f64,
    distance_other_a: f64,
    distance_other_b: f64,
    min_side: f64,
    min_other_a: f64,
    max_other_a: f64,
    min_other_b: f64,
    max_other_b: f64,
    hit_side: Direction,
    start_side: f64,
    start_other_a: f64,
    start_other_b: f64,
) -> Option<Direction> {
    let d = (min_side - start_side) / distance_side;
    let d1 = start_other_a + d * distance_other_a;
    let d2 = start_other_b + d * distance_other_b;
    if 0.0 < d
        && d < min_distance[0]
        && min_other_a - 1.0e-7 < d1
        && d1 < max_other_a + 1.0e-7
        && min_other_b - 1.0e-7 < d2
        && d2 < max_other_b + 1.0e-7
    {
        min_distance[0] = d;
        Some(hit_side)
    } else {
        prev_direction
    }
}

impl std::fmt::Display for Aabb {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AABB[{}, {}, {}] -> [{}, {}, {}]",
            self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z
        )
    }
}
