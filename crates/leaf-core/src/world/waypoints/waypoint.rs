//! Перенос `net.minecraft.world.waypoints.Waypoint` — интерфейс +
//! вложенный `Icon` (иконка метки на карте/в мире).

use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `WaypointStyleAssets.ROOT_ID` — реестр `waypoint_style_asset`.
/// На проводе `ResourceKey.streamCodec(ROOT_ID)` пишет только
/// `ResourceLocation` ключа (реестр подразумевается).
pub const WAYPOINT_STYLE_ASSET_ROOT: &str = "waypoint_style_asset";

/// `WaypointStyleAssets.DEFAULT` — `minecraft:default`.
pub fn waypoint_style_assets_default() -> Identifier {
    Identifier::with_default_namespace("default").expect("default waypoint style")
}

/// `Waypoint.Icon` — иконка метки: стиль + опциональный RGB-цвет.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WaypointIcon {
    /// `style` — `ResourceKey<WaypointStyleAsset>` (на проводе — Identifier).
    pub style: Identifier,
    /// `color` — RGB int (0xRRGGBB), optional.
    pub color: Option<i32>,
}

impl Default for WaypointIcon {
    /// `Waypoint.Icon.NULL`/`DEFAULT`-эквивалент.
    fn default() -> Self {
        Self {
            style: waypoint_style_assets_default(),
            color: None,
        }
    }
}

impl WaypointIcon {
    /// `Waypoint.Icon.NULL`.
    pub fn null() -> WaypointIcon {
        WaypointIcon::default()
    }

    /// `hasData`.
    pub fn has_data(&self) -> bool {
        self.style != waypoint_style_assets_default() || self.color.is_some()
    }

    /// `copyFrom`.
    pub fn copy_from(&mut self, icon: &WaypointIcon) {
        self.color = icon.color;
        self.style = icon.style.clone();
    }

    /// `STREAM_CODEC` — ResourceKey(style) + optional RGB_COLOR
    /// (RGB_COLOR пишет 3 байта R,G,B).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, WaypointIcon> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, icon: &WaypointIcon| {
                buf.write_identifier(&icon.style).expect("waypoint style");
                match icon.color {
                    Some(c) => {
                        buf.write_bool(true);
                        buf.write_byte((c >> 16) & 0xFF);
                        buf.write_byte((c >> 8) & 0xFF);
                        buf.write_byte(c & 0xFF);
                    }
                    None => {
                        buf.write_bool(false);
                    }
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let style = buf.read_identifier().expect("waypoint style");
                let color = if buf.read_bool() {
                    let r = buf.read_byte() as i32 & 0xFF;
                    let g = buf.read_byte() as i32 & 0xFF;
                    let b = buf.read_byte() as i32 & 0xFF;
                    Some((r << 16) | (g << 8) | b)
                } else {
                    None
                };
                WaypointIcon { style, color }
            },
        )
    }
}
