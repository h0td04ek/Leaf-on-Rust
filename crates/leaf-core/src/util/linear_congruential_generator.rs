//! Перенос `net.minecraft.util.LinearCongruentialGenerator`.

const MULTIPLIER: i64 = 6364136223846793005;
const INCREMENT: i64 = 1442695040888963407;

/// `LinearCongruentialGenerator.next(left, right)`.
pub fn next(left: i64, right: i64) -> i64 {
    let left = left.wrapping_mul(left.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT));
    left.wrapping_add(right)
}
