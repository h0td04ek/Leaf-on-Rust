//! Перенос `net.minecraft.world.item.crafting.display.RecipeDisplayId`.

use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `RecipeDisplayId` — VarInt index.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RecipeDisplayId {
    /// `index`.
    pub index: i32,
}

impl RecipeDisplayId {
    /// `new`.
    pub fn new(index: i32) -> Self {
        Self { index }
    }

    /// `STREAM_CODEC` — `VAR_INT`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, RecipeDisplayId> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, id: &RecipeDisplayId| {
                buf.write_var_int(id.index);
            },
            |buf: &mut FriendlyByteBuf| RecipeDisplayId::new(buf.read_var_int()),
        )
    }
}
