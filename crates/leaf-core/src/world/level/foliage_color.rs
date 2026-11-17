//! `net.minecraft.world.level.FoliageColor`.

use super::color_map_color_util;
use std::sync::{Mutex, OnceLock};

/// `FOLIAGE_EVERGREEN`.
pub const FOLIAGE_EVERGREEN: i32 = -10380959;
/// `FOLIAGE_BIRCH`.
pub const FOLIAGE_BIRCH: i32 = -8345771;
/// `FOLIAGE_DEFAULT`.
pub const FOLIAGE_DEFAULT: i32 = -12012264;
/// `FOLIAGE_MANGROVE`.
pub const FOLIAGE_MANGROVE: i32 = -7158200;

static PIXELS: OnceLock<Mutex<Vec<i32>>> = OnceLock::new();

fn pixels() -> std::sync::MutexGuard<'static, Vec<i32>> {
    PIXELS
        .get_or_init(|| Mutex::new(vec![0; 65536]))
        .lock()
        .unwrap()
}

/// `init(foliageBuffer)`.
pub fn init(foliage_buffer: Vec<i32>) {
    *pixels() = foliage_buffer;
}

/// `get(temperature, downfall)`.
pub fn get(temperature: f64, downfall: f64) -> i32 {
    let guard = pixels();
    color_map_color_util::get(temperature, downfall, &guard, FOLIAGE_DEFAULT)
}
