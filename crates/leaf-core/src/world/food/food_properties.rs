//! Перенос `net.minecraft.world.food.FoodProperties`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `FoodProperties` — nutrition, saturation, canAlwaysEat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoodProperties {
    /// `nutrition`.
    pub nutrition: i32,
    /// `saturation`.
    pub saturation: f32,
    /// `canAlwaysEat`.
    pub can_always_eat: bool,
}

impl FoodProperties {
    /// `new`.
    pub fn new(nutrition: i32, saturation: f32, can_always_eat: bool) -> Self {
        Self { nutrition, saturation, can_always_eat }
    }

    /// `DIRECT_STREAM_CODEC` — VAR_INT + FLOAT + BOOL.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, FoodProperties> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &FoodProperties| {
                buf.write_var_int(v.nutrition);
                buf.write_float(v.saturation);
                buf.write_bool(v.can_always_eat);
            },
            |buf: &mut FriendlyByteBuf| {
                let nutrition = buf.read_var_int();
                let saturation = buf.read_float();
                let can_always_eat = buf.read_bool();
                FoodProperties { nutrition, saturation, can_always_eat }
            },
        )
    }
}
