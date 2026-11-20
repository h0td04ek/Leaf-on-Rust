//! Перенос `PosRuleTest` + наследников (`PosAlwaysTrueTest`,
//! `LinearPosTest`, `AxisAlignedLinearPosTest`).

use crate::core::block_pos::BlockPos;
use crate::core::direction::{Axis, AxisDirection, Direction};
use crate::random::RandomSource;
use crate::util::mth;

/// `PosRuleTest` — предикат на позиции (localPos/relativePos/
/// structurePos).
pub trait PosRuleTest: Send + Sync {
    /// `test(localPos, relativePos, structurePos, random)`.
    fn test(
        &self,
        local_pos: BlockPos,
        relative_pos: BlockPos,
        structure_pos: BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool;
}

/// `PosAlwaysTrueTest`.
pub struct PosAlwaysTrueTest;

impl PosRuleTest for PosAlwaysTrueTest {
    fn test(
        &self,
        _local_pos: BlockPos,
        _relative_pos: BlockPos,
        _structure_pos: BlockPos,
        _random: &mut dyn RandomSource,
    ) -> bool {
        true
    }
}

/// `LinearPosTest` — линейная вероятность по manhattan-дистанции.
pub struct LinearPosTest {
    /// `minChance`.
    pub min_chance: f32,
    /// `maxChance`.
    pub max_chance: f32,
    /// `minDist`.
    pub min_dist: i32,
    /// `maxDist`.
    pub max_dist: i32,
}

impl LinearPosTest {
    /// `new(minChance, maxChance, minDist, maxDist)` —
    /// `minDist >= maxDist` → panic (как в Java).
    pub fn new(min_chance: f32, max_chance: f32, min_dist: i32, max_dist: i32) -> Self {
        assert!(min_dist < max_dist, "Invalid range: [{}, {}]", min_dist, max_dist);
        Self {
            min_chance,
            max_chance,
            min_dist,
            max_dist,
        }
    }
}

impl PosRuleTest for LinearPosTest {
    fn test(
        &self,
        _local_pos: BlockPos,
        relative_pos: BlockPos,
        structure_pos: BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool {
        let dist = relative_pos.dist_manhattan(structure_pos) as f32;
        let random_float = random.next_float();
        random_float
            <= mth::clamped_lerp_f32(
                mth::inverse_lerp_f32(dist, self.min_dist as f32, self.max_dist as f32) as f32,
                self.min_chance,
                self.max_chance,
            )
    }
}

/// `AxisAlignedLinearPosTest` — линейная вероятность по
/// дистанции вдоль оси.
pub struct AxisAlignedLinearPosTest {
    /// `minChance`.
    pub min_chance: f32,
    /// `maxChance`.
    pub max_chance: f32,
    /// `minDist`.
    pub min_dist: i32,
    /// `maxDist`.
    pub max_dist: i32,
    /// `axis`.
    pub axis: Axis,
}

impl AxisAlignedLinearPosTest {
    /// `new(minChance, maxChance, minDist, maxDist, axis)`.
    pub fn new(
        min_chance: f32,
        max_chance: f32,
        min_dist: i32,
        max_dist: i32,
        axis: Axis,
    ) -> Self {
        assert!(min_dist < max_dist, "Invalid range: [{}, {}]", min_dist, max_dist);
        Self {
            min_chance,
            max_chance,
            min_dist,
            max_dist,
            axis,
        }
    }
}

impl PosRuleTest for AxisAlignedLinearPosTest {
    fn test(
        &self,
        _local_pos: BlockPos,
        relative_pos: BlockPos,
        structure_pos: BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool {
        let direction = Direction::UP.get(AxisDirection::POSITIVE, self.axis);
        let step = direction.get_unit_vec3i();
        let dist = ((relative_pos.x - structure_pos.x) * step.x).abs()
            + ((relative_pos.y - structure_pos.y) * step.y).abs()
            + ((relative_pos.z - structure_pos.z) * step.z).abs();
        let random_float = random.next_float();
        random_float
            <= mth::clamped_lerp_f32(
                mth::inverse_lerp_f32(dist as f32, self.min_dist as f32, self.max_dist as f32) as f32,
                self.min_chance,
                self.max_chance,
            )
    }
}
