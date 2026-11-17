//! Перенос `net.minecraft.world.level.ChunkPos`.
//! Codec — TODO (network layer).


/// `ChunkPos` — пара int-координат чанка, упакованная в long.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }

    /// `ChunkPos.asLong(x, z)` — статическая упаковка.
    pub fn as_long_static(x: i32, z: i32) -> i64 {
        ((x as i64) & 0xFFFFFFFF) << 32 | ((z as i64) & 0xFFFFFFFF)
    }

    pub fn of(packed: i64) -> Self {
        Self::new((packed >> 32) as i32, packed as i32)
    }

    pub fn get_region_x(x: i32) -> i32 {
        x >> 5
    }

    pub fn get_region_z(z: i32) -> i32 {
        z >> 5
    }

    pub fn get_region_x_pos(&self) -> i32 {
        Self::get_region_x(self.x)
    }

    pub fn get_region_z_pos(&self) -> i32 {
        Self::get_region_z(self.z)
    }

    pub fn get_block_x(&self, section_x: i32, quarter_x: i32) -> i32 {
        (self.x << 8) + (section_x << 4) + quarter_x
    }

    pub fn get_block_z(&self, section_z: i32, quarter_z: i32) -> i32 {
        (self.z << 8) + (section_z << 4) + quarter_z
    }

    pub fn get_block_coord(&self, section_coord: i32, quarter_coord: i32) -> i32 {
        (section_coord << 8) + quarter_coord
    }

    pub fn get_middle_block_x(&self) -> i32 {
        this_block_coord(self.x, 8)
    }

    pub fn get_middle_block_z(&self) -> i32 {
        this_block_coord(self.z, 8)
    }

    pub fn get_min_block_x(&self) -> i32 {
        this_block_coord(self.x, 0)
    }

    pub fn get_min_block_z(&self) -> i32 {
        this_block_coord(self.z, 0)
    }

    pub fn get_max_block_x(&self) -> i32 {
        this_block_coord(self.x, 15)
    }

    pub fn get_max_block_z(&self) -> i32 {
        this_block_coord(self.z, 15)
    }

    pub fn as_long(&self) -> i64 {
        Self::as_long_static(self.x, self.z)
    }

    /// `betweenClosed` по чанкам — возвращает Vec (диапазоны малы).
    pub fn between_closed(min_x: i32, min_z: i32, max_x: i32, max_z: i32) -> Vec<Self> {
        let mut out = Vec::new();
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                out.push(Self::new(x, z));
            }
        }
        out
    }

    pub fn between_closed_vec4(min_chunk_x: i32, min_chunk_z: i32, max_chunk_x: i32, max_chunk_z: i32) -> Vec<Self> {
        Self::between_closed(min_chunk_x, min_chunk_z, max_chunk_x, max_chunk_z)
    }

    /// `rangeClosed(minX..maxX, minZ..maxZ)` — спан в блоках, но как чанки.
    pub fn range_closed(min_chunk_x: i32, min_chunk_z: i32, max_chunk_x: i32, max_chunk_z: i32) -> Vec<Self> {
        Self::between_closed(min_chunk_x, min_chunk_z, max_chunk_x, max_chunk_z)
    }

    /// `squareOutSouthEast`-подобное: 4 чанка от pos (pos, +x, +z, +x+z).
    pub fn square_around(pos: &Self) -> [Self; 4] {
        [*pos, Self::new(pos.x + 1, pos.z), Self::new(pos.x, pos.z + 1), Self::new(pos.x + 1, pos.z + 1)]
    }

    /// `manhattanDistance`
    pub fn manhattan_distance(&self, other: &Self) -> i32 {
        (self.x - other.x).abs() + (self.z - other.z).abs()
    }

    /// `distanceSquared` до block-координат.
    pub fn distance_squared_to_block(&self, x: f64, z: f64) -> f64 {
        let dx = self.get_middle_block_x() as f64 - x;
        let dz = self.get_middle_block_z() as f64 - z;
        dx * dx + dz * dz
    }

    /// `hashCode` совместимый.
    pub fn hash_code(&self) -> i32 {
        (self.x * 31 + self.z) as i32
    }

    /// `toString` — "[x, z]" в Java? ChunkPos toString = "[x, z]" (через Vec2i?) — фактически "[x, z]".
    pub fn to_string(&self) -> String {
        format!("[{}, {}]", self.x, self.z)
    }
}

fn this_block_coord(chunk_coord: i32, offset: i32) -> i32 {
    chunk_coord * 16 + offset
}

impl From<&crate::core::section_pos::SectionPos> for ChunkPos {
    fn from(s: &crate::core::section_pos::SectionPos) -> Self {
        Self::new(s.x, s.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_roundtrip() {
        for (x, z) in [(0, 0), (1, -1), (-30000000, 30000000), (i32::MAX, i32::MIN)] {
            assert_eq!(ChunkPos::of(ChunkPos::as_long_static(x, z)), ChunkPos::new(x, z));
        }
    }

    #[test]
    fn block_coords() {
        let p = ChunkPos::new(1, -2);
        assert_eq!(p.get_min_block_x(), 16);
        assert_eq!(p.get_max_block_x(), 31);
        assert_eq!(p.get_min_block_z(), -32);
        assert_eq!(p.get_max_block_z(), -17);
        assert_eq!(p.get_middle_block_x(), 24);
        assert_eq!(p.get_middle_block_z(), -24);
    }

    #[test]
    fn region() {
        assert_eq!(ChunkPos::get_region_x(-33), -2);
        assert_eq!(ChunkPos::get_region_z(33), 1);
    }

    #[test]
    fn between_closed_count() {
        let v = ChunkPos::between_closed(0, 0, 2, 1);
        assert_eq!(v.len(), 6);
    }
}
