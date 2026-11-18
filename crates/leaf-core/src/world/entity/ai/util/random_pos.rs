//! Перенос `net.minecraft.world.entity.ai.util.RandomPos`.

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::util::mth;
use crate::world::entity::mob::Mob;
use crate::world::phys::vec3::Vec3;

/// `RANDOM_POS_ATTEMPTS`.
const RANDOM_POS_ATTEMPTS: i32 = 10;

/// `generateRandomDirection(random, horizontalDistance,
/// verticalDistance)` — случайное смещение в кубе.
pub fn generate_random_direction(
    random: &mut dyn crate::random::RandomSource,
    horizontal_distance: i32,
    vertical_distance: i32,
) -> BlockPos {
    let x = random.next_int_bound(2 * horizontal_distance + 1) - horizontal_distance;
    let y = random.next_int_bound(2 * vertical_distance + 1) - vertical_distance;
    let z = random.next_int_bound(2 * horizontal_distance + 1) - horizontal_distance;
    BlockPos::new(x, y, z)
}

/// `generateRandomDirectionWithinRadians(random, minDistance,
/// maxDistance, yRange, y, x, z, maxAngleDelta)`.
#[allow(clippy::too_many_arguments)]
pub fn generate_random_direction_within_radians(
    random: &mut dyn crate::random::RandomSource,
    min_distance: f64,
    max_distance: f64,
    y_range: i32,
    y: i32,
    x: f64,
    z: f64,
    max_angle_delta: f64,
) -> Option<BlockPos> {
    let angle = mth::atan2(z, x) - std::f64::consts::FRAC_PI_2;
    let new_angle = angle + (2.0 * random.next_float() as f64 - 1.0) * max_angle_delta;
    let dist = mth::lerp_f64(random.next_double().sqrt(), min_distance, max_distance) * std::f64::consts::SQRT_2;
    let dx = -dist * new_angle.sin();
    let dz = dist * new_angle.cos();
    if dx.abs() > max_distance || dz.abs() > max_distance {
        return None;
    }
    let yy = random.next_int_bound(2 * y_range + 1) - y_range + y;
    Some(BlockPos::containing(dx, yy as f64, dz))
}

/// `moveUpOutOfSolid(pos, maxY, predicate)`.
pub fn move_up_out_of_solid(
    pos: BlockPos,
    max_y: i32,
    predicate: impl Fn(BlockPos) -> bool,
) -> BlockPos {
    if !predicate(pos) {
        return pos;
    }
    let mut p = pos.offset(0, 1, 0);
    while p.y <= max_y && predicate(p) {
        p = p.offset(0, 1, 0);
    }
    p
}

/// `moveUpToAboveSolid(pos, aboveSolidAmount, maxY, predicate)`.
pub fn move_up_to_above_solid(
    pos: BlockPos,
    above_solid_amount: i32,
    max_y: i32,
    predicate: impl Fn(BlockPos) -> bool,
) -> BlockPos {
    assert!(above_solid_amount >= 0);
    if !predicate(pos) {
        return pos;
    }
    let mut p = pos.offset(0, 1, 0);
    while p.y <= max_y && predicate(p) {
        p = p.offset(0, 1, 0);
    }
    let y = p.y;
    while p.y <= max_y && p.y - y < above_solid_amount {
        p = p.offset(0, 1, 0);
        if predicate(p) {
            p = p.offset(0, -1, 0);
            break;
        }
    }
    p
}

/// `generateRandomPos(mob, posSupplier)` — 10 попыток,
/// выбор с максимальным walkTargetValue.
pub fn generate_random_pos(
    mob: &Mob,
    mut pos_supplier: impl FnMut() -> Option<BlockPos>,
) -> Option<Vec3> {
    let mut best: Option<BlockPos> = None;
    let mut best_value = f64::NEG_INFINITY;
    for _ in 0..RANDOM_POS_ATTEMPTS {
        if let Some(bp) = pos_supplier() {
            let v = mob.get_walk_target_value(bp);
            if v > best_value {
                best_value = v;
                best = Some(bp);
            }
        }
    }
    best.map(|bp| bp.get_bottom_center())
}

/// `generateRandomPosTowardDirection(mob, range, random, pos)` —
/// смещение с учётом home-ограничения.
pub fn generate_random_pos_toward_direction(
    mob: &Mob,
    range: f64,
    random: &mut dyn crate::random::RandomSource,
    pos: BlockPos,
) -> BlockPos {
    let mut x = pos.x as f64;
    let mut z = pos.z as f64;
    if mob.has_home() && range > 1.0 {
        let home = mob.home_position;
        if mob.get_x() > home.x as f64 {
            x -= random.next_double() * range / 2.0;
        } else {
            x += random.next_double() * range / 2.0;
        }
        if mob.get_z() > home.z as f64 {
            z -= random.next_double() * range / 2.0;
        } else {
            z += random.next_double() * range / 2.0;
        }
    }
    BlockPos::containing(x + mob.get_x(), pos.y as f64 + mob.get_y(), z + mob.get_z())
}
