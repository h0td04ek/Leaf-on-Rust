//! Перенос `net.minecraft.util.Unit` и мелких enum-обёрток.

/// `Unit.INSTANCE` — синглтон "ничего" для кодеков/коллекций.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Unit;

pub const INSTANCE: Unit = Unit;

/// `Brightness` — упакованная яркость (block<<4 | sky<<20).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Brightness {
    pub block: i32,
    pub sky: i32,
}

impl Brightness {
    pub const FULL_BRIGHT: Brightness = Brightness { block: 15, sky: 15 };

    pub const fn new(block: i32, sky: i32) -> Self {
        Self { block, sky }
    }

    pub fn pack(&self) -> i32 {
        Self::pack_i(self.block, self.sky)
    }

    pub const fn pack_i(block: i32, sky: i32) -> i32 {
        (block << 4) | (sky << 20)
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
