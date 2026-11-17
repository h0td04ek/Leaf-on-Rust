//! `net.minecraft.world.level.GrassColor`.

use super::color_map_color_util;
use std::sync::{Mutex, OnceLock};

static PIXELS: OnceLock<Mutex<Vec<i32>>> = OnceLock::new();

fn pixels() -> std::sync::MutexGuard<'static, Vec<i32>> {
    PIXELS
        .get_or_init(|| Mutex::new(vec![0; 65536]))
        .lock()
        .unwrap()
}

/// `init(grassBuffer)` — вызывается из bootstrap'а worldgen-данных.
pub fn init(grass_buffer: Vec<i32>) {
    *pixels() = grass_buffer;
}

/// `get(temperature, downfall)`.
pub fn get(temperature: f64, downfall: f64) -> i32 {
    let guard = pixels();
    color_map_color_util::get(temperature, downfall, &guard, -65281)
}

/// `getDefaultColor`.
pub fn get_default_color() -> i32 {
    get(0.5, 1.0)
}
