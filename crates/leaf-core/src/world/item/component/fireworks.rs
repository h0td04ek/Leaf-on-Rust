//! Перенос `net.minecraft.world.item.component.Fireworks`.

use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::item::component::firework_explosion::FireworkExplosion;

/// `Fireworks` — (flightDuration, explosions ≤256).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fireworks {
    /// `flightDuration`.
    pub flight_duration: i32,
    /// `explosions`.
    pub explosions: Vec<FireworkExplosion>,
}

impl Fireworks {
    /// `MAX_EXPLOSIONS`.
    pub const MAX_EXPLOSIONS: usize = 256;

    /// `new`.
    pub fn new(flight_duration: i32, explosions: Vec<FireworkExplosion>) -> Self {
        Self { flight_duration, explosions }
    }

    /// `STREAM_CODEC` — VAR_INT + list(256, FireworkExplosion).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Fireworks> + Send + Sync + 'static {
        let list_enc = byte_buf_codecs::list_limited(256, FireworkExplosion::stream_codec());
        let list_dec = byte_buf_codecs::list_limited(256, FireworkExplosion::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Fireworks| {
                buf.write_var_int(v.flight_duration);
                list_enc.encode(buf, &v.explosions);
            },
            move |buf: &mut FriendlyByteBuf| {
                let flight_duration = buf.read_var_int();
                let explosions = list_dec.decode(buf);
                Fireworks { flight_duration, explosions }
            },
        )
    }
}
