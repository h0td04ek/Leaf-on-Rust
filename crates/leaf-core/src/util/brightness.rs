//! Перенос `net.minecraft.util.Brightness` (record).

/// `Brightness(block, sky)` — упакованная яркость блока/неба (0..15).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Brightness {
    pub block: i32,
    pub sky: i32,
}

impl Brightness {
    pub const FULL_BRIGHT: Self = Self { block: 15, sky: 15 };

    pub fn new(block: i32, sky: i32) -> Self {
        Self { block, sky }
    }

    /// `pack(block, sky)` — block << 4 | sky << 20.
    pub fn pack_levels(block: i32, sky: i32) -> i32 {
        (block << 4) | (sky << 20)
    }

    pub fn pack(&self) -> i32 {
        Self::pack_levels(self.block, self.sky)
    }

    pub fn block(packed: i32) -> i32 {
        (packed >> 4) & 0xFFFF
    }

    pub fn sky(packed: i32) -> i32 {
        (packed >> 20) & 0xFFFF
    }

    pub fn unpack(packed_brightness: i32) -> Self {
        Self::new(Self::block(packed_brightness), Self::sky(packed_brightness))
    }
}
