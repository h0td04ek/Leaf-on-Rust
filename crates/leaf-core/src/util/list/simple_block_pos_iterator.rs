//! `org.dreeam.leaf.util.list.SimpleBlockPosIterator` —
//! Y-major обход AABB (оптимизированный `betweenClosed`).

use crate::core::block_pos::BlockPos;
use crate::util::mth;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;

/// `SimpleBlockPosIterator`.
#[derive(Clone)]
pub struct SimpleBlockPosIterator {
    start_x: i32,
    start_y: i32,
    start_z: i32,
    end_x: i32,
    end_y: i32,
    end_z: i32,
    pos: Option<BlockPos>,
}

impl SimpleBlockPosIterator {
    /// `iterable(bounds)` — собирает в `Vec` (Java возвращает
    /// ленивый `Iterable`; для горячих путей достаточно Vec —
    /// при необходимости заменить на истинный итератор без
    /// изменения call-site'ов, использующих `iterable`/`collect`).
    pub fn iterable(bounds: &Aabb) -> Vec<BlockPos> {
        Self::new(bounds).collect()
    }

    /// `traverseBoundsInDirection(direction, bounds, maxDistance)`.
    pub fn traverse_bounds_in_direction(
        direction: &Vec3,
        bounds: &Aabb,
        max_distance: f64,
    ) -> Vec<BlockPos> {
        let scaled_distance = (max_distance / direction.length()).min(1.0);
        let previous_bounds = bounds.move_vec(&direction.scale(-1.0));
        let bounds_to_search = previous_bounds.expand_towards_vec(&direction.scale(scaled_distance));
        Self::iterable(&bounds_to_search)
    }

    /// `new(bounds)`.
    pub fn new(bounds: &Aabb) -> Self {
        Self {
            start_x: mth::floor(bounds.min_x),
            start_y: mth::floor(bounds.min_y),
            start_z: mth::floor(bounds.min_z),
            end_x: mth::floor(bounds.max_x),
            end_y: mth::floor(bounds.max_y),
            end_z: mth::floor(bounds.max_z),
            pos: None,
        }
    }
}

impl Iterator for SimpleBlockPosIterator {
    type Item = BlockPos;

    fn next(&mut self) -> Option<BlockPos> {
        match self.pos {
            None => {
                let p = BlockPos::new(self.start_x, self.start_y, self.start_z);
                self.pos = Some(p);
                Some(p)
            }
            Some(p) => {
                let (mut x, mut y, mut z) = (p.get_x(), p.get_y(), p.get_z());
                if y < self.end_y {
                    y += 1;
                } else if x < self.end_x {
                    x += 1;
                    y = self.start_y;
                } else if z < self.end_z {
                    z += 1;
                    x = self.start_x;
                    y = self.start_y;
                } else {
                    return None;
                }
                let next = BlockPos::new(x, y, z);
                self.pos = Some(next);
                Some(next)
            }
        }
    }
}
