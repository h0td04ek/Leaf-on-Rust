//! Перенос `net.minecraft.world.level.levelgen.structure.BoundingBox`.

use crate::core::block_pos::BlockPos;
use crate::core::direction::{Axis, Direction};
use crate::core::section_pos::SectionPos;
use crate::world::level::chunk_pos::ChunkPos;
use crate::core::vec3i::Vec3i;

/// `BoundingBox` — AABB из целых координат (границы включительно).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoundingBox {
    /// `minX`.
    pub min_x: i32,
    /// `minY`.
    pub min_y: i32,
    /// `minZ`.
    pub min_z: i32,
    /// `maxX`.
    pub max_x: i32,
    /// `maxY`.
    pub max_y: i32,
    /// `maxZ`.
    pub max_z: i32,
}

impl BoundingBox {
    /// `new(minX, minY, minZ, maxX, maxY, maxZ)` — нормализует
    /// (min/max переставляются при инверсии).
    pub fn new(mut min_x: i32, mut min_y: i32, mut min_z: i32, mut max_x: i32, mut max_y: i32, mut max_z: i32) -> Self {
        if min_x > max_x {
            std::mem::swap(&mut min_x, &mut max_x);
        }
        if min_y > max_y {
            std::mem::swap(&mut min_y, &mut max_y);
        }
        if min_z > max_z {
            std::mem::swap(&mut min_z, &mut max_z);
        }
        Self { min_x, min_y, min_z, max_x, max_y, max_z }
    }

    /// `BoundingBox(BlockPos)` — точечный бокс (0×0×0 → min=max-1?
    /// Java: `new BoundingBox(pos)` = fromCorners(pos,pos) →
    /// min=max).
    pub fn of_pos(pos: BlockPos) -> Self {
        Self::new(pos.x, pos.y, pos.z, pos.x, pos.y, pos.z)
    }

    /// `fromCorners(first, second)`.
    pub fn from_corners(first: Vec3i, second: Vec3i) -> Self {
        Self::new(first.x, first.y, first.z, second.x, second.y, second.z)
    }

    /// `infinite()` — полный диапазон.
    pub fn infinite() -> Self {
        Self::new(i32::MIN, i32::MIN, i32::MIN, i32::MAX, i32::MAX, i32::MAX)
    }

    /// `orientBox(structureMinX, structureMinY, structureMinZ,
    /// xMin, yMin, zMin, xMax, yMax, zMax, facing)` — бокс куска
    /// от offset-координат (используется NetherFortress и др.).
    /// `xMax/yMax/zMax` — это max-координаты минус 1 внутри
    /// (Java: `min + max - 1`).
    #[allow(clippy::too_many_arguments)]
    pub fn orient_box_offsets(
        structure_min_x: i32,
        structure_min_y: i32,
        structure_min_z: i32,
        x_min: i32,
        y_min: i32,
        z_min: i32,
        x_max: i32,
        y_max: i32,
        z_max: i32,
        facing: Direction,
    ) -> Self {
        match facing {
            Direction::NORTH => Self::new(
                structure_min_x + x_min,
                structure_min_y + y_min,
                structure_min_z - z_max + 1 + z_min,
                structure_min_x + x_max - 1 + x_min,
                structure_min_y + y_max - 1 + y_min,
                structure_min_z + z_min,
            ),
            Direction::WEST => Self::new(
                structure_min_x - z_max + 1 + z_min,
                structure_min_y + y_min,
                structure_min_z + x_min,
                structure_min_x + z_min,
                structure_min_y + y_max - 1 + y_min,
                structure_min_z + x_max - 1 + x_min,
            ),
            Direction::EAST => Self::new(
                structure_min_x + z_min,
                structure_min_y + y_min,
                structure_min_z + x_min,
                structure_min_x + z_max - 1 + z_min,
                structure_min_y + y_max - 1 + y_min,
                structure_min_z + x_max - 1 + x_min,
            ),
            _ => Self::new(
                structure_min_x + x_min,
                structure_min_y + y_min,
                structure_min_z + z_min,
                structure_min_x + x_max - 1 + x_min,
                structure_min_y + y_max - 1 + y_min,
                structure_min_z + z_max - 1 + z_min,
            ),
        }
    }

    /// `orientBox(x, y, z, direction, width, depth, height)` —
    /// бокс для постройки заданного направления.
    pub fn orient_box(
        x: i32,
        y: i32,
        z: i32,
        direction: Direction,
        width: i32,
        depth: i32,
        height: i32,
    ) -> Self {
        match direction.get_axis() {
            Axis::Z => Self::new(
                x - depth / 2,
                y,
                z - width / 2,
                x + depth / 2 + (depth & 1),
                y + height - 1,
                z + width / 2 + (width & 1),
            ),
            _ => Self::new(
                x - width / 2,
                y,
                z - depth / 2,
                x + width / 2 + (width & 1),
                y + height - 1,
                z + depth / 2 + (depth & 1),
            ),
        }
    }

    /// `intersectingChunks()` — чанки, пересекающие бокс.
    pub fn intersecting_chunks(&self) -> Vec<ChunkPos> {
        let mut out = Vec::new();
        let min_cx = self.min_x >> 4;
        let max_cx = self.max_x >> 4;
        let min_cz = self.min_z >> 4;
        let max_cz = self.max_z >> 4;
        for cx in min_cx..=max_cx {
            for cz in min_cz..=max_cz {
                out.push(ChunkPos::of(crate::world::level::chunk_pos::ChunkPos::as_long_static(cx, cz)));
            }
        }
        out
    }

    /// `intersects(BoundingBox)`.
    pub fn intersects(&self, box_: &BoundingBox) -> bool {
        self.min_x <= box_.max_x
            && self.max_x >= box_.min_x
            && self.min_y <= box_.max_y
            && self.max_y >= box_.min_y
            && self.min_z <= box_.max_z
            && self.max_z >= box_.min_z
    }

    /// `intersects(minX, minZ, maxX, maxZ)` — 2D-вариант
    /// (X/Z-пересечение на полной высоте).
    pub fn intersects_2d(&self, min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> bool {
        self.min_x <= max_x && self.max_x >= min_x && self.min_z <= max_z && self.max_z >= min_z
    }

    /// `intersects(ChunkPos)`-стиль: `intersects(minBlockX,
    /// minBlockZ, maxBlockX, maxBlockZ)`.
    pub fn intersects_chunk(&self, chunk_pos: ChunkPos) -> bool {
        self.intersects_2d(
            chunk_pos.get_min_block_x(),
            chunk_pos.get_min_block_z(),
            chunk_pos.get_min_block_x() + 15,
            chunk_pos.get_min_block_z() + 15,
        )
    }

    /// `encapsulatingPositions(positions)` — бокс по точкам.
    pub fn encapsulating_positions(positions: &[BlockPos]) -> Option<BoundingBox> {
        if positions.is_empty() {
            return None;
        }
        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut min_z = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;
        let mut max_z = i32::MIN;
        for p in positions {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            min_z = min_z.min(p.z);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
            max_z = max_z.max(p.z);
        }
        Some(Self::new(min_x, min_y, min_z, max_x, max_y, max_z))
    }

    /// `encapsulatingBoxes(boxes)` — объединяющий бокс.
    pub fn encapsulating_boxes<'a>(boxes: impl Iterator<Item = &'a BoundingBox>) -> Option<BoundingBox> {
        let mut iter = boxes.peekable();
        let first = iter.next()?;
        let mut out = *first;
        for b in iter {
            out = out.encapsulate(b);
        }
        Some(out)
    }

    /// `encapsulate(BoundingBox)` — расширить до включения box.
    pub fn encapsulate(&self, box_: &BoundingBox) -> BoundingBox {
        Self::new(
            self.min_x.min(box_.min_x),
            self.min_y.min(box_.min_y),
            self.min_z.min(box_.min_z),
            self.max_x.max(box_.max_x),
            self.max_y.max(box_.max_y),
            self.max_z.max(box_.max_z),
        )
    }

    /// `encapsulating(box1, box2)` — статический вариант.
    pub fn encapsulating(box1: &BoundingBox, box2: &BoundingBox) -> BoundingBox {
        box1.encapsulate(box2)
    }

    /// `encapsulate(BlockPos)`.
    pub fn encapsulate_pos(&self, pos: BlockPos) -> BoundingBox {
        self.encapsulate(&BoundingBox::of_pos(pos))
    }

    /// `move(x, y, z)` — сдвиг (mutating, как Java).
    pub fn move_(&mut self, x: i32, y: i32, z: i32) {
        self.min_x += x;
        self.min_y += y;
        self.min_z += z;
        self.max_x += x;
        self.max_y += y;
        self.max_z += z;
    }

    /// `move(Vec3i)`.
    pub fn move_vec(&mut self, v: Vec3i) {
        self.move_(v.x, v.y, v.z);
    }

    /// `moved(x, y, z)` — сдвинутая копия.
    pub fn moved(&self, x: i32, y: i32, z: i32) -> BoundingBox {
        let mut b = *self;
        b.move_(x, y, z);
        b
    }

    /// `inflatedBy(value)`.
    pub fn inflated_by(&self, value: i32) -> BoundingBox {
        self.inflated_by_xyz(value, value, value)
    }

    /// `inflatedBy(x, y, z)`.
    pub fn inflated_by_xyz(&self, x: i32, y: i32, z: i32) -> BoundingBox {
        Self::new(
            self.min_x - x,
            self.min_y - y,
            self.min_z - z,
            self.max_x + x,
            self.max_y + y,
            self.max_z + z,
        )
    }

    /// `isInside(Vec3i)`.
    pub fn is_inside_vec(&self, v: Vec3i) -> bool {
        self.is_inside(v.x, v.y, v.z)
    }

    /// `isInside(x, y, z)`.
    pub fn is_inside(&self, x: i32, y: i32, z: i32) -> bool {
        x >= self.min_x && x <= self.max_x && y >= self.min_y && y <= self.max_y && z >= self.min_z && z <= self.max_z
    }

    /// `isInside(BlockPos)`.
    pub fn is_inside_pos(&self, pos: BlockPos) -> bool {
        self.is_inside(pos.x, pos.y, pos.z)
    }

    /// `getLength()` — `(max-min)` по осям.
    pub fn get_length(&self) -> Vec3i {
        Vec3i::new(self.max_x - self.min_x, self.max_y - self.min_y, self.max_z - self.min_z)
    }

    /// `getXSpan()`/`getYSpan()`/`getZSpan()`.
    pub fn get_x_span(&self) -> i32 {
        self.max_x - self.min_x
    }
    /// `getYSpan`.
    pub fn get_y_span(&self) -> i32 {
        self.max_y - self.min_y
    }
    /// `getZSpan`.
    pub fn get_z_span(&self) -> i32 {
        self.max_z - self.min_z
    }

    /// `getCenter()` — центр (floor-деление).
    pub fn get_center(&self) -> BlockPos {
        BlockPos::new(
            (self.min_x + self.max_x + 1) / 2,
            (self.min_y + self.max_y + 1) / 2,
            (self.min_z + self.max_z + 1) / 2,
        )
    }

    /// `forAllCorners(consumer)`.
    pub fn for_all_corners(&self, mut f: impl FnMut(BlockPos)) {
        for x in [self.min_x, self.max_x] {
            for y in [self.min_y, self.max_y] {
                for z in [self.min_z, self.max_z] {
                    f(BlockPos::new(x, y, z));
                }
            }
        }
    }

    /// `getMinX`-стиль аксессоры (Java minX() и т.п.).
    /// `minX()`.
    pub fn min_x_(&self) -> i32 {
        self.min_x
    }
    /// `minY()`.
    pub fn min_y_(&self) -> i32 {
        self.min_y
    }
    /// `minZ()`.
    pub fn min_z_(&self) -> i32 {
        self.min_z
    }
    /// `maxX()`.
    pub fn max_x_(&self) -> i32 {
        self.max_x
    }
    /// `maxY()`.
    pub fn max_y_(&self) -> i32 {
        self.max_y
    }
    /// `maxZ()`.
    pub fn max_z_(&self) -> i32 {
        self.max_z
    }
}
