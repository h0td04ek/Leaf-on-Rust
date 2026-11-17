//! `MapItemSavedData.MapPatch` — патч цветов карты.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `MapPatch` — `{ startX, startY, width, height, mapColors }`.
#[derive(Clone, Debug)]
pub struct MapPatch {
    /// `startX`.
    pub start_x: i32,
    /// `startY`.
    pub start_y: i32,
    /// `width`.
    pub width: i32,
    /// `height`.
    pub height: i32,
    /// `mapColors`.
    pub map_colors: Vec<u8>,
}

impl MapPatch {
    /// `new`.
    pub fn new(start_x: i32, start_y: i32, width: i32, height: i32, map_colors: Vec<u8>) -> Self {
        Self { start_x, start_y, width, height, map_colors }
    }

    /// `STREAM_CODEC` — `StreamCodec.of(MapPatch::write, MapPatch::read)` для `Optional<MapPatch>`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Option<MapPatch>> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, patch: &Option<MapPatch>| {
                match patch {
                    Some(p) => {
                        buf.write_byte(p.width);
                        buf.write_byte(p.height);
                        buf.write_byte(p.start_x);
                        buf.write_byte(p.start_y);
                        buf.write_byte_array(&p.map_colors);
                    },
                    None => { buf.write_byte(0); },
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let width = buf.read_unsigned_byte();
                if width > 0 {
                    let height = buf.read_unsigned_byte();
                    let start_x = buf.read_unsigned_byte();
                    let start_y = buf.read_unsigned_byte();
                    let map_colors = buf.read_byte_array().unwrap_or_default();
                    Some(Self::new(start_x, start_y, width, height, map_colors))
                } else {
                    None
                }
            },
        )
    }

    /// `applyToMap` — применить патч к `MapItemSavedData`.
    pub fn apply_to_map(&self, saved_data: &mut dyn FnMut(i32, i32, u8)) {
        for i in 0..self.width {
            for j in 0..self.height {
                let idx = (i + j * self.width) as usize;
                if idx < self.map_colors.len() {
                    saved_data(self.start_x + i, self.start_y + j, self.map_colors[idx]);
                }
            }
        }
    }
}
