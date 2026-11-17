//! `MapDecoration` — декорация на карте.

use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::chat::component::Component;
use crate::network::chat::component_json::{component_stream_codec, optional_component_stream_codec};
use crate::world::level::saveddata::maps::map_decoration_type::MapDecorationType;
use crate::core::holder::Holder;

/// `MapDecoration` — `{ type, x, y, rot, name }`.
#[derive(Clone, Debug)]
pub struct MapDecoration {
    /// `type`.
    pub decoration_type: Holder<MapDecorationType>,
    /// `x`.
    pub x: i8,
    /// `y`.
    pub y: i8,
    /// `rot` — `& 15`.
    pub rot: i8,
    /// `name`.
    pub name: Option<Component>,
}

impl MapDecoration {
    /// `new` — `rot & 15`.
    pub fn new(
        decoration_type: Holder<MapDecorationType>,
        x: i8,
        y: i8,
        rot: i8,
        name: Option<Component>,
    ) -> Self {
        Self { decoration_type, x, y, rot: rot & 15, name }
    }

    /// `getSpriteLocation`.
    pub fn get_sprite_location(&self) -> &crate::resources::Identifier {
        // Type-erased — return placeholder
        static DEFAULT: std::sync::OnceLock<crate::resources::Identifier> = std::sync::OnceLock::new();
        DEFAULT.get_or_init(|| crate::resources::Identifier::from_namespace_and_path("minecraft", "map/decorations/white").unwrap())
    }

    /// `renderOnFrame`.
    pub fn render_on_frame(&self) -> bool {
        // Type-erased — default true
        true
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(MapDecorationType.STREAM_CODEC, BYTE, BYTE, BYTE, OPTIONAL_COMPONENT, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MapDecoration> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, d: &MapDecoration| {
                MapDecorationType::stream_codec().encode(buf, &d.decoration_type);
                buf.write_byte(d.x as i32);
                buf.write_byte(d.y as i32);
                buf.write_byte(d.rot as i32);
                match &d.name {
                    Some(c) => {
                        buf.write_bool(true);
                        component_stream_codec().encode(buf, c);
                    },
                    None => { buf.write_bool(false); },
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let decoration_type = MapDecorationType::stream_codec().decode(buf);
                let x = buf.read_byte();
                let y = buf.read_byte();
                let rot = buf.read_byte();
                let has_name = buf.read_bool();
                let name = if has_name {
                    Some(optional_component_stream_codec().decode(buf).unwrap())
                } else {
                    None
                };
                Self::new(decoration_type, x, y, rot, name)
            },
        )
    }
}
