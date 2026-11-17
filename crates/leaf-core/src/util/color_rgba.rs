//! Перенос `net.minecraft.util.ColorRGBA` (record).
//! DFU-кодек (`CODEC`) — позже.

/// `ColorRGBA(rgba)` — цвет в формате 0xRRGGBBAA.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ColorRGBA {
    pub rgba: i32,
}

impl ColorRGBA {
    pub fn new(rgba: i32) -> Self {
        Self { rgba }
    }
}

impl std::fmt::Display for ColorRGBA {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:08x}", self.rgba as u32)
    }
}
