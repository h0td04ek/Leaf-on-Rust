//! Перенос `net.minecraft.core.SectionPos`.
//! Codec/StreamCodec/streams — см. аналоги-итераторы.

use crate::core::block_pos::{BlockPos, BetweenClosed};
use crate::core::direction::Direction;
use crate::core::vec3i::Vec3i;
use crate::util::mth;
use crate::world::level::chunk_pos::ChunkPos;
use crate::world::phys::Vec3;

pub const SECTION_BITS: i32 = 4;
pub const SECTION_SIZE: i32 = 16;
pub const SECTION_MASK: i32 = 15;
pub const SECTION_HALF_SIZE: i32 = 8;
pub const SECTION_MAX_INDEX: i32 = 15;

const PACKED_X_MASK: i64 = 4194303;
const PACKED_Y_MASK: i64 = 1048575;
const PACKED_Z_MASK: i64 = 4194303;
const Z_OFFSET: i32 = 20;
const X_OFFSET: i32 = 42;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SectionPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl SectionPos {
    pub const fn of_i(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn of(pos: &BlockPos) -> Self {
        Self::of_i(pos.x >> 4, pos.y >> 4, pos.z >> 4)
    }

    pub fn of_chunk(chunk_pos: &ChunkPos, y: i32) -> Self {
        Self::of_i(chunk_pos.x, y, chunk_pos.z)
    }

    /// `of(Vec3)` / `of(Position)`
    pub fn of_vec3(position: &Vec3) -> Self {
        Self::of_i(
            Self::block_to_section_coord_f64(position.x),
            Self::block_to_section_coord_f64(position.y),
            Self::block_to_section_coord_f64(position.z),
        )
    }

    pub fn of_packed(packed: i64) -> Self {
        Self::of_i(
            (packed >> 42) as i32,
            ((packed << 44) >> 44) as i32,
            ((packed << 22) >> 42) as i32,
        )
    }

    // ---------------------------------------------------------- packing

    /// Paper-оптимизированный `offset(packed, Direction)`.
    pub fn offset_packed(packed: i64, direction: Direction) -> i64 {
        Self::offset_packed_xyz(packed, direction.get_step_x(), direction.get_step_y(), direction.get_step_z())
    }

    /// Paper: `getAdjacentFromBlockPos` — packed section от block-координат.
    pub fn get_adjacent_from_block_pos(x: i32, y: i32, z: i32, direction: Direction) -> i64 {
        let sx = ((x >> 4) + direction.get_step_x()) as i64 & PACKED_X_MASK;
        let sy = ((y >> 4) + direction.get_step_y()) as i64 & PACKED_Y_MASK;
        let sz = ((z >> 4) + direction.get_step_z()) as i64 & PACKED_Z_MASK;
        (sx << X_OFFSET) | sy | (sz << Z_OFFSET)
    }

    /// Paper: `getAdjacentFromSectionPos`.
    pub fn get_adjacent_from_section_pos(x: i32, y: i32, z: i32, direction: Direction) -> i64 {
        let sx = (x + direction.get_step_x()) as i64 & PACKED_X_MASK;
        let sy = (y + direction.get_step_y()) as i64 & PACKED_Y_MASK;
        let sz = (z + direction.get_step_z()) as i64 & PACKED_Z_MASK;
        (sx << X_OFFSET) | sy | (sz << Z_OFFSET)
    }

    pub fn offset_packed_xyz(packed: i64, dx: i32, dy: i32, dz: i32) -> i64 {
        let sx = (((packed >> 42) as i32) + dx) as i64 & PACKED_X_MASK;
        let sy = (((packed << 44) >> 44) as i32 + dy) as i64 & PACKED_Y_MASK;
        let sz = (((packed << 22) >> 42) as i32 + dz) as i64 & PACKED_Z_MASK;
        (sx << X_OFFSET) | sy | (sz << Z_OFFSET)
    }

    // ---------------------------------------------------------- coord conv

    pub fn pos_to_section_coord(pos: f64) -> i32 {
        Self::block_to_section_coord(mth::floor(pos))
    }

    pub fn block_to_section_coord(block_coord: i32) -> i32 {
        block_coord >> SECTION_BITS
    }

    pub fn block_to_section_coord_f64(block_coord: f64) -> i32 {
        mth::floor(block_coord) >> SECTION_BITS
    }

    pub fn section_relative(block_coord: i32) -> i32 {
        block_coord & SECTION_MASK
    }

    /// `sectionRelativePos` — x<<8 | z<<4 | y.
    pub fn section_relative_pos(pos: &BlockPos) -> i16 {
        (((pos.x & 15) << 8) | ((pos.z & 15) << 4) | (pos.y & 15)) as i16
    }

    pub fn section_relative_x(v: i16) -> i32 {
        ((v as u16) >> 8) as i32 & 15
    }

    pub fn section_relative_y(v: i16) -> i32 {
        ((v as u16) >> 0) as i32 & 15
    }

    pub fn section_relative_z(v: i16) -> i32 {
        ((v as u16) >> 4) as i32 & 15
    }

    pub fn section_to_block_coord(section_coord: i32) -> i32 {
        section_coord << SECTION_BITS
    }

    pub fn section_to_block_coord_offset(section_coord: i32, offset: i32) -> i32 {
        Self::section_to_block_coord(section_coord) + offset
    }

    // ---------------------------------------------------------- packed fns

    pub fn x_packed(packed: i64) -> i32 {
        (packed << 0 >> X_OFFSET) as i32
    }

    pub fn y_packed(packed: i64) -> i32 {
        (packed << 44 >> 44) as i32
    }

    pub fn z_packed(packed: i64) -> i32 {
        (packed << Z_OFFSET >> X_OFFSET) as i32
    }

    /// `blockToSection(blockPos packed)` — из BlockPos-packed long.
    pub fn block_to_section(packed_block_pos: i64) -> i64 {
        let sx = ((packed_block_pos >> 42) as i64) & PACKED_X_MASK;
        let sy = ((packed_block_pos << 52) >> 56) as i64 & PACKED_Y_MASK;
        let sz = ((packed_block_pos << 26) >> 42) as i64 & PACKED_Z_MASK;
        (sx << X_OFFSET) | sy | (sz << Z_OFFSET)
    }

    pub fn get_zero_node(x: i32, z: i32) -> i64 {
        Self::get_zero_node_i(Self::as_long_static(x, 0, z))
    }

    pub fn get_zero_node_i(pos: i64) -> i64 {
        pos & !0xFFFFF // -1048576
    }

    /// `sectionToChunk` — ChunkPos.asLong(x, z).
    pub fn section_to_chunk(pos: i64) -> i64 {
        ChunkPos::as_long_static(Self::x_packed(pos), Self::z_packed(pos))
    }

    // ---------------------------------------------------------- instance

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }

    pub fn get_z(&self) -> i32 {
        self.z
    }

    pub fn min_block_x(&self) -> i32 {
        self.x << SECTION_BITS
    }

    pub fn min_block_y(&self) -> i32 {
        self.y << SECTION_BITS
    }

    pub fn min_block_z(&self) -> i32 {
        self.z << SECTION_BITS
    }

    pub fn max_block_x(&self) -> i32 {
        Self::section_to_block_coord_offset(self.x, SECTION_MASK)
    }

    pub fn max_block_y(&self) -> i32 {
        Self::section_to_block_coord_offset(self.y, SECTION_MASK)
    }

    pub fn max_block_z(&self) -> i32 {
        Self::section_to_block_coord_offset(self.z, SECTION_MASK)
    }

    pub fn relative_to_block_x(&self, x: i16) -> i32 {
        self.min_block_x() + Self::section_relative_x(x)
    }

    pub fn relative_to_block_y(&self, y: i16) -> i32 {
        self.min_block_y() + Self::section_relative_y(y)
    }

    pub fn relative_to_block_z(&self, z: i16) -> i32 {
        self.min_block_z() + Self::section_relative_z(z)
    }

    pub fn relative_to_block_pos(&self, pos: i16) -> BlockPos {
        BlockPos::new(
            self.relative_to_block_x(pos),
            self.relative_to_block_y(pos),
            self.relative_to_block_z(pos),
        )
    }

    pub fn origin(&self) -> BlockPos {
        BlockPos::new(
            Self::section_to_block_coord(self.x),
            Self::section_to_block_coord(self.y),
            Self::section_to_block_coord(self.z),
        )
    }

    pub fn center(&self) -> BlockPos {
        self.origin().offset(8, 8, 8)
    }

    pub fn chunk(&self) -> ChunkPos {
        ChunkPos::new(self.x, self.z)
    }

    pub fn as_long_static(x: i32, y: i32, z: i32) -> i64 {
        (((x as i64) & PACKED_X_MASK) << X_OFFSET)
            | (((y as i64) & PACKED_Y_MASK))
            | (((z as i64) & PACKED_Z_MASK) << Z_OFFSET)
    }

    /// Paper: `blockPosAsSectionLong`
    pub fn block_pos_as_section_long(x: i32, y: i32, z: i32) -> i64 {
        ((((x >> 4) as i64) & PACKED_X_MASK) << X_OFFSET)
            | ((((y >> 4) as i64) & PACKED_Y_MASK))
            | ((((z >> 4) as i64) & PACKED_Z_MASK) << Z_OFFSET)
    }

    pub fn as_long(&self) -> i64 {
        Self::as_long_static(self.x, self.y, self.z)
    }

    pub fn as_long_from_block_pos(pos: &BlockPos) -> i64 {
        Self::as_long_static(
            Self::block_to_section_coord(pos.x),
            Self::block_to_section_coord(pos.y),
            Self::block_to_section_coord(pos.z),
        )
    }

    pub fn offset(&self, dx: i32, dy: i32, dz: i32) -> Self {
        if dx == 0 && dy == 0 && dz == 0 {
            *self
        } else {
            Self::of_i(self.x + dx, self.y + dy, self.z + dz)
        }
    }

    /// `blocksInside` — итератор BlockPos по 16³.
    pub fn blocks_inside(&self) -> BetweenClosed {
        BlockPos::between_closed_coords(
            self.min_block_x(),
            self.min_block_y(),
            self.min_block_z(),
            self.max_block_x(),
            self.max_block_y(),
            self.max_block_z(),
        )
    }

    /// `cube` — куб SectionPos радиуса radius.
    pub fn cube(center: &Self, radius: i32) -> BetweenClosedSections {
        Self::between_closed_coords(
            center.x - radius,
            center.y - radius,
            center.z - radius,
            center.x + radius,
            center.y + radius,
            center.z + radius,
        )
    }

    /// `aroundChunk`
    pub fn around_chunk(chunk_pos: &ChunkPos, radius: i32, min_y: i32, max_y: i32) -> BetweenClosedSections {
        Self::between_closed_coords(
            chunk_pos.x - radius,
            min_y,
            chunk_pos.z - radius,
            chunk_pos.x + radius,
            max_y,
            chunk_pos.z + radius,
        )
    }

    /// Аналог `betweenClosedStream(x1,y1,z1,x2,y2,z2)`.
    pub fn between_closed_coords(x1: i32, y1: i32, z1: i32, x2: i32, y2: i32, z2: i32) -> BetweenClosedSections {
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let min_z = z1.min(z2);
        let dx = x1.max(x2) - min_x + 1;
        let dy = y1.max(y2) - min_y + 1;
        let dz = z1.max(z2) - min_z + 1;
        BetweenClosedSections {
            min: Self::of_i(min_x, min_y, min_z),
            dims: (dx, dy, dz),
            index: 0,
            total: dx as i64 * dy as i64 * dz as i64,
        }
    }

    /// `aroundAndAtBlockPos(x,y,z)` — все секции, пересекаемые блоком ±1.
    pub fn around_and_at_block_pos(x: i32, y: i32, z: i32, consumer: &mut dyn FnMut(i64)) {
        let sx1 = Self::block_to_section_coord(x - 1);
        let sx2 = Self::block_to_section_coord(x + 1);
        let sy1 = Self::block_to_section_coord(y - 1);
        let sy2 = Self::block_to_section_coord(y + 1);
        let sz1 = Self::block_to_section_coord(z - 1);
        let sz2 = Self::block_to_section_coord(z + 1);
        if sx1 == sx2 && sy1 == sy2 && sz1 == sz2 {
            consumer(Self::as_long_static(sx1, sy1, sz1));
        } else {
            for i in sx1..=sx2 {
                for i1 in sy1..=sy2 {
                    for i2 in sz1..=sz2 {
                        consumer(Self::as_long_static(i, i1, i2));
                    }
                }
            }
        }
    }

    pub fn around_and_at_block_pos_packed(packed_block_pos: i64, consumer: &mut dyn FnMut(i64)) {
        Self::around_and_at_block_pos(
            BlockPos::get_x_packed(packed_block_pos),
            BlockPos::get_y_packed(packed_block_pos),
            BlockPos::get_z_packed(packed_block_pos),
            consumer,
        )
    }

    pub fn around_and_at_block_pos_vec(pos: &BlockPos, consumer: &mut dyn FnMut(i64)) {
        Self::around_and_at_block_pos(pos.x, pos.y, pos.z, consumer)
    }

    /// `toString` — Vec3i формат "x, y, z".
    pub fn to_string(&self) -> String {
        format!("{}, {}, {}", self.x, self.y, self.z)
    }
}

impl From<&Vec3i> for SectionPos {
    fn from(v: &Vec3i) -> Self {
        Self::of_i(
            Self::block_to_section_coord(v.x),
            Self::block_to_section_coord(v.y),
            Self::block_to_section_coord(v.z),
        )
    }
}

/// Аналог `betweenClosedStream` — итератор SectionPos.
pub struct BetweenClosedSections {
    min: SectionPos,
    dims: (i32, i32, i32),
    index: i64,
    total: i64,
}

impl BetweenClosedSections {
}

impl Iterator for BetweenClosedSections {
    type Item = SectionPos;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.total {
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
        Some(SectionPos::of_i(
            self.min.x + i4 as i32,
            self.min.y + i6 as i32,
            self.min.z + i7 as i32,
        ))
    }
}
