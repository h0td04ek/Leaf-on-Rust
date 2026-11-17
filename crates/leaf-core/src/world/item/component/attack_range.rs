//! Перенос `net.minecraft.world.item.component.AttackRange`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `AttackRange` — 6 float-полей.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AttackRange {
    /// `minRange`.
    pub min_range: f32,
    /// `maxRange`.
    pub max_range: f32,
    /// `minCreativeRange`.
    pub min_creative_range: f32,
    /// `maxCreativeRange`.
    pub max_creative_range: f32,
    /// `hitboxMargin`.
    pub hitbox_margin: f32,
    /// `mobFactor`.
    pub mob_factor: f32,
}

impl AttackRange {
    /// `CODEC_DEFAULT` (Paper defaults).
    pub const DEFAULT: AttackRange = AttackRange {
        min_range: 0.0,
        max_range: 3.0,
        min_creative_range: 0.0,
        max_creative_range: 5.0,
        hitbox_margin: 0.3,
        mob_factor: 1.0,
    };

    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        min_range: f32,
        max_range: f32,
        min_creative_range: f32,
        max_creative_range: f32,
        hitbox_margin: f32,
        mob_factor: f32,
    ) -> Self {
        Self { min_range, max_range, min_creative_range, max_creative_range, hitbox_margin, mob_factor }
    }

    /// `STREAM_CODEC` — 6× FLOAT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, AttackRange> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &AttackRange| {
                buf.write_float(v.min_range);
                buf.write_float(v.max_range);
                buf.write_float(v.min_creative_range);
                buf.write_float(v.max_creative_range);
                buf.write_float(v.hitbox_margin);
                buf.write_float(v.mob_factor);
            },
            |buf: &mut FriendlyByteBuf| {
                let min_range = buf.read_float();
                let max_range = buf.read_float();
                let min_creative_range = buf.read_float();
                let max_creative_range = buf.read_float();
                let hitbox_margin = buf.read_float();
                let mob_factor = buf.read_float();
                AttackRange::new(
                    min_range,
                    max_range,
                    min_creative_range,
                    max_creative_range,
                    hitbox_margin,
                    mob_factor,
                )
            },
        )
    }
}
