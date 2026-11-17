//! `net.minecraft.world.level.DryFoliageColor`.

use super::color_map_color_util;
use std::sync::{Mutex, OnceLock};

/// `FOLIAGE_DRY_DEFAULT`.
pub const FOLIAGE_DRY_DEFAULT: i32 = -10732494;

static PIXELS: OnceLock<Mutex<Vec<i32>>> = OnceLock::new();

fn pixels() -> std::sync::MutexGuard<'static, Vec<i32>> {
    PIXELS
        .get_or_init(|| Mutex::new(vec![0; 65536]))
        .lock()
        .unwrap()
}

/// `init(pixels)`.
pub fn init(buffer: Vec<i32>) {
    *pixels() = buffer;
}

/// `get(x, y)`.
pub fn get(x: f64, y: f64) -> i32 {
    let guard = pixels();
    color_map_color_util::get(x, y, &guard, FOLIAGE_DRY_DEFAULT)
}
