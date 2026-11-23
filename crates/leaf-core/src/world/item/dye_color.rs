//! Перенос `net.minecraft.world.item.DyeColor` — 16 красителей.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `DyeColor` — порядок = `id`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DyeColor {
    /// `WHITE`.
    WHITE,
    /// `ORANGE`.
    ORANGE,
    /// `MAGENTA`.
    MAGENTA,
    /// `LIGHT_BLUE`.
    LIGHT_BLUE,
    /// `YELLOW`.
    YELLOW,
    /// `LIME`.
    LIME,
    /// `PINK`.
    PINK,
    /// `GRAY`.
    GRAY,
    /// `LIGHT_GRAY`.
    LIGHT_GRAY,
    /// `CYAN`.
    CYAN,
    /// `PURPLE`.
    PURPLE,
    /// `BLUE`.
    BLUE,
    /// `BROWN`.
    BROWN,
    /// `GREEN`.
    GREEN,
    /// `RED`.
    RED,
    /// `BLACK`.
    BLACK,
}

impl DyeColor {
    /// `values()`.
    pub const VALUES: [DyeColor; 16] = [
        DyeColor::WHITE, DyeColor::ORANGE, DyeColor::MAGENTA, DyeColor::LIGHT_BLUE,
        DyeColor::YELLOW, DyeColor::LIME, DyeColor::PINK, DyeColor::GRAY,
        DyeColor::LIGHT_GRAY, DyeColor::CYAN, DyeColor::PURPLE, DyeColor::BLUE,
        DyeColor::BROWN, DyeColor::GREEN, DyeColor::RED, DyeColor::BLACK,
    ];

    /// `id`.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `ByIdMap.continuous(..., ZERO)` — вне диапазона → `WHITE`.
    pub fn by_id(id: i32) -> DyeColor {
        if (0..16).contains(&id) {
            Self::VALUES[id as usize]
        } else {
            Self::VALUES[0]
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            DyeColor::WHITE => "white",
            DyeColor::ORANGE => "orange",
            DyeColor::MAGENTA => "magenta",
            DyeColor::LIGHT_BLUE => "light_blue",
            DyeColor::YELLOW => "yellow",
            DyeColor::LIME => "lime",
            DyeColor::PINK => "pink",
            DyeColor::GRAY => "gray",
            DyeColor::LIGHT_GRAY => "light_gray",
            DyeColor::CYAN => "cyan",
            DyeColor::PURPLE => "purple",
            DyeColor::BLUE => "blue",
            DyeColor::BROWN => "brown",
            DyeColor::GREEN => "green",
            DyeColor::RED => "red",
            DyeColor::BLACK => "black",
        }
    }

    /// `CODEC.byName`.
    pub fn by_name(name: &str) -> Option<DyeColor> {
        Self::VALUES.iter().copied().find(|d| d.get_serialized_name() == name)
    }

    /// `byFireworkColor` — `fireworkColor` = 0xFF000000 | (rgb & 0xFFFFFF)?
    /// В Java: fireworkColor = 0xFF000000 | (rgb <<? ) — см. конструктор:
    /// `byFireworkColor`.
    pub fn by_firework_color(color: i32) -> Option<DyeColor> {
        Self::VALUES.iter().copied().find(|d| d.firework_color() == color)
    }

    /// `textColor`.
    pub fn text_color(&self) -> i32 {
        match self {
            DyeColor::WHITE => 16777215,
            DyeColor::ORANGE => 16738335,
            DyeColor::MAGENTA => 16711935,
            DyeColor::LIGHT_BLUE => 10141901,
            DyeColor::YELLOW => 16776960,
            DyeColor::LIME => 12582656,
            DyeColor::PINK => 16738740,
            DyeColor::GRAY => 8421504,
            DyeColor::LIGHT_GRAY => 13882323,
            DyeColor::CYAN => 65535,
            DyeColor::PURPLE => 10494192,
            DyeColor::BLUE => 255,
            DyeColor::BROWN => 9127187,
            DyeColor::GREEN => 65280,
            DyeColor::RED => 16711680,
            DyeColor::BLACK => 0,
        }
    }

    /// `fireworkColor`.
    pub fn firework_color(&self) -> i32 {
        match self {
            DyeColor::WHITE => 15790320,
            DyeColor::ORANGE => 15435844,
            DyeColor::MAGENTA => 12801229,
            DyeColor::LIGHT_BLUE => 6719955,
            DyeColor::YELLOW => 14602026,
            DyeColor::LIME => 4312372,
            DyeColor::PINK => 14188952,
            DyeColor::GRAY => 4408131,
            DyeColor::LIGHT_GRAY => 11250603,
            DyeColor::CYAN => 2651799,
            DyeColor::PURPLE => 8073150,
            DyeColor::BLUE => 2437522,
            DyeColor::BROWN => 5320730,
            DyeColor::GREEN => 3887386,
            DyeColor::RED => 11743532,
            DyeColor::BLACK => 1973019,
        }
    }

    /// `col` — ARGB (3-й аргумент таблицы).
    pub fn col(&self) -> i32 {
        match self {
            DyeColor::WHITE => 16383998,
            DyeColor::ORANGE => 16351261,
            DyeColor::MAGENTA => 13061821,
            DyeColor::LIGHT_BLUE => 3847130,
            DyeColor::YELLOW => 16701501,
            DyeColor::LIME => 8439583,
            DyeColor::PINK => 15961002,
            DyeColor::GRAY => 4673362,
            DyeColor::LIGHT_GRAY => 10329495,
            DyeColor::CYAN => 1481884,
            DyeColor::PURPLE => 8991416,
            DyeColor::BLUE => 3949738,
            DyeColor::BROWN => 8606770,
            DyeColor::GREEN => 6192150,
            DyeColor::RED => 11546150,
            DyeColor::BLACK => 1908001,
        }
    }

    /// `STREAM_CODEC` — `ByteBufCodecs.idMapper(BY_ID, getId)` (VarInt id).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DyeColor> {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &DyeColor| {
                buf.write_var_int(v.get_id());
            },
            |buf: &mut FriendlyByteBuf| DyeColor::by_id(buf.read_var_int()),
        )
    }
}
