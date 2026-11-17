//! Перенос `net.minecraft.world.phys.HitResult` + `BlockHitResult`.
//! `EntityHitResult` — после порта Entity (см. world::entity).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::world::phys::Vec3;

/// `HitResult.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HitResultType {
    MISS,
    BLOCK,
    ENTITY,
}

/// `HitResult` — базовый результат raycast.
#[derive(Clone, Debug)]
pub struct HitResult {
    pub location: Vec3,
}

impl HitResult {
    pub fn new(location: Vec3) -> Self {
        Self { location }
    }

    /// `distanceTo(Entity)` — TODO после порта Entity.
    pub fn distance_to_xyz(&self, x: f64, y: f64, z: f64) -> f64 {
        let dx = self.location.x - x;
        let dy = self.location.y - y;
        let dz = self.location.z - z;
        dx * dx + dy * dy + dz * dz
    }
}

/// `BlockHitResult`.
#[derive(Clone, Debug)]
pub struct BlockHitResult {
    pub location: Vec3,
    pub direction: Direction,
    pub block_pos: BlockPos,
    pub miss: bool,
    pub inside: bool,
    pub world_border_hit: bool,
}

impl BlockHitResult {
    pub fn miss(location: Vec3, direction: Direction, block_pos: BlockPos) -> Self {
        Self::new_miss(true, location, direction, block_pos, false, false)
    }

    pub fn new(location: Vec3, direction: Direction, block_pos: BlockPos, inside: bool) -> Self {
        Self::new_miss(false, location, direction, block_pos, inside, false)
    }

    pub fn new_border(location: Vec3, direction: Direction, block_pos: BlockPos, inside: bool, world_border_hit: bool) -> Self {
        Self::new_miss(false, location, direction, block_pos, inside, world_border_hit)
    }

    fn new_miss(miss: bool, location: Vec3, direction: Direction, block_pos: BlockPos, inside: bool, world_border_hit: bool) -> Self {
        Self { location, direction, block_pos, miss, inside, world_border_hit }
    }

    pub fn with_direction(&self, new_face: Direction) -> Self {
        Self::new_miss(self.miss, self.location, new_face, self.block_pos, self.inside, self.world_border_hit)
    }

    pub fn with_position(&self, pos: BlockPos) -> Self {
        Self::new_miss(self.miss, self.location, self.direction, pos, self.inside, self.world_border_hit)
    }

    pub fn hit_border(&self) -> Self {
        Self::new_miss(self.miss, self.location, self.direction, self.block_pos, self.inside, true)
    }

    /// `getType`
    pub fn hit_type(&self) -> HitResultType {
        if self.miss { HitResultType::MISS } else { HitResultType::BLOCK }
    }
}
