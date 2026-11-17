//! Перенос `net.minecraft.world.item.component.UseCooldown`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::identifier::Identifier;

/// `UseCooldown` — cooldown после использования предмета.
#[derive(Clone, Debug, PartialEq)]
pub struct UseCooldown {
    /// `seconds`.
    pub seconds: f32,
    /// `cooldownGroup` (optional).
    pub cooldown_group: Option<Identifier>,
}

impl UseCooldown {
    /// `UseCooldown(seconds)`.
    pub fn new(seconds: f32) -> Self {
        Self { seconds, cooldown_group: None }
    }

    /// `ticks`.
    pub fn ticks(&self) -> i32 {
        (self.seconds * 20.0) as i32
    }

    /// `STREAM_CODEC` — FLOAT + optional IDENTIFIER.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, UseCooldown> + Send + Sync + 'static {
        let group_codec = std::sync::Arc::new(byte_buf_codecs::optional(byte_buf_codecs::IDENTIFIER));
                let group_codec2 = group_codec.clone();
sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &UseCooldown| {
                buf.write_float(v.seconds);
                group_codec.encode(buf, &v.cooldown_group);
            },
            move |buf: &mut FriendlyByteBuf| UseCooldown {
                seconds: buf.read_float(),
                cooldown_group: group_codec2.decode(buf),
            },
        )
    }
}
