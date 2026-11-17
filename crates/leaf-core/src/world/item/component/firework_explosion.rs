//! Перенос `net.minecraft.world.item.component.FireworkExplosion`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `FireworkExplosion.Shape` — id = wire.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FireworkExplosionShape {
    /// `SMALL_BALL`.
    SMALL_BALL,
    /// `LARGE_BALL`.
    LARGE_BALL,
    /// `STAR`.
    STAR,
    /// `CREEPER`.
    CREEPER,
    /// `BURST`.
    BURST,
}

impl FireworkExplosionShape {
    /// `getId`.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `BY_ID` — `OutOfBoundsStrategy.ZERO`.
    pub fn by_id(id: i32) -> FireworkExplosionShape {
        match id {
            0 => FireworkExplosionShape::SMALL_BALL,
            1 => FireworkExplosionShape::LARGE_BALL,
            2 => FireworkExplosionShape::STAR,
            3 => FireworkExplosionShape::CREEPER,
            4 => FireworkExplosionShape::BURST,
            _ => FireworkExplosionShape::SMALL_BALL,
        }
    }

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            FireworkExplosionShape::SMALL_BALL => "small_ball",
            FireworkExplosionShape::LARGE_BALL => "large_ball",
            FireworkExplosionShape::STAR => "star",
            FireworkExplosionShape::CREEPER => "creeper",
            FireworkExplosionShape::BURST => "burst",
        }
    }

    /// `Shape.STREAM_CODEC` — idMapper.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, FireworkExplosionShape> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &FireworkExplosionShape| {
                buf.write_var_int(v.get_id());
            },
            |buf: &mut FriendlyByteBuf| FireworkExplosionShape::by_id(buf.read_var_int()),
        )
    }
}

/// `IntList` — список int (fastutil IntList).
pub type IntList = Vec<i32>;

/// `FireworkExplosion` — (shape, colors, fadeColors, hasTrail,
/// hasTwinkle).
#[derive(Clone, Debug, PartialEq)]
pub struct FireworkExplosion {
    /// `shape`.
    pub shape: FireworkExplosionShape,
    /// `colors`.
    pub colors: IntList,
    /// `fadeColors`.
    pub fade_colors: IntList,
    /// `hasTrail`.
    pub has_trail: bool,
    /// `hasTwinkle`.
    pub has_twinkle: bool,
}

impl FireworkExplosion {
    /// `DEFAULT`.
    pub fn default_explosion() -> Self {
        Self {
            shape: FireworkExplosionShape::SMALL_BALL,
            colors: Vec::new(),
            fade_colors: Vec::new(),
            has_trail: false,
            has_twinkle: false,
        }
    }

    /// `COLOR_LIST_STREAM_CODEC` — list(INT).
    fn color_list_codec() -> impl StreamCodec<FriendlyByteBuf, Vec<i32>> + Send + Sync + 'static {
        byte_buf_codecs::list(byte_buf_codecs::INT)
    }

    /// `STREAM_CODEC` — Shape + colors + fadeColors + trail + twinkle.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, FireworkExplosion> + Send + Sync + 'static {
        let colors_enc = Self::color_list_codec();
        let colors_dec = Self::color_list_codec();
        let fade_enc = Self::color_list_codec();
        let fade_dec = Self::color_list_codec();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &FireworkExplosion| {
                FireworkExplosionShape::stream_codec().encode(buf, &v.shape);
                colors_enc.encode(buf, &v.colors);
                fade_enc.encode(buf, &v.fade_colors);
                buf.write_bool(v.has_trail);
                buf.write_bool(v.has_twinkle);
            },
            move |buf: &mut FriendlyByteBuf| {
                let shape = FireworkExplosionShape::stream_codec().decode(buf);
                let colors = colors_dec.decode(buf);
                let fade_colors = fade_dec.decode(buf);
                let has_trail = buf.read_bool();
                let has_twinkle = buf.read_bool();
                FireworkExplosion { shape, colors, fade_colors, has_trail, has_twinkle }
            },
        )
    }
}
