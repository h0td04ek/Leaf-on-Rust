//! `net.minecraft.world.level.ColorMapColorUtil` —
//! интерполяция по цветовой карте 256×256.

/// `get(x, y, pixels, defaultValue)` — x=temperature, y=downfall.
pub fn get(x: f64, y: f64, pixels: &[i32], default_value: i32) -> i32 {
    let y = y * x;
    let i = ((1.0 - x) * 255.0) as i32;
    let i1 = ((1.0 - y) * 255.0) as i32;
    let i2 = (i1 << 8) | i;
    if i2 >= pixels.len() as i32 || i2 < 0 {
        default_value
    } else {
        pixels[i2 as usize]
    }
}
