//! `Mth` — математические хелперы, нужные random-модулю.
//! Полный порт Mth.java — отдельной итерацией.

/// `Mth.getSeed(x, y, z)` — seed для позиционных RNG.
pub fn get_seed(x: i32, y: i32, z: i32) -> i64 {
    let mut l = (x as i64)
        .wrapping_mul(3129871)
        ^ (z as i64).wrapping_mul(116129781)
        ^ (y as i64);
    l = l.wrapping_mul(l).wrapping_mul(42317861).wrapping_add(l.wrapping_mul(11));
    l >> 16
}

pub fn square(v: f64) -> f64 {
    v * v
}
