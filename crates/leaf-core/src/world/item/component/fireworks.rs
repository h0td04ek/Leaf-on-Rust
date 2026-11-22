//! Перенос `net.minecraft.world.item.component.Fireworks`.

use crate::nbt::{CompoundTag, Tag};
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

    pub(crate) fn to_nbt(&self) -> Result<Tag, String> {
        let flight = u8::try_from(self.flight_duration).map_err(|_| "invalid firework flight duration")?;
        if self.explosions.len() > Self::MAX_EXPLOSIONS {
            return Err("too many firework explosions".into());
        }
        let mut tag = CompoundTag::new();
        if flight != 0 {
            tag.put_byte("flight_duration", flight as i8);
        }
        if !self.explosions.is_empty() {
            tag.put("explosions", Tag::List(self.explosions.iter().map(FireworkExplosion::to_nbt).collect()));
        }
        Ok(Tag::Compound(tag))
    }

    pub(crate) fn from_nbt(tag: &Tag) -> Result<Self, String> {
        let tag = tag.as_compound().ok_or("expected a fireworks compound")?;
        let flight_duration = match tag.get("flight_duration") {
            None => 0,
            Some(Tag::Byte(value)) => i32::from(*value as u8),
            Some(value) => {
                let flight = value.as_long().and_then(|value| u8::try_from(value).ok())
                    .ok_or("invalid firework flight duration")?;
                i32::from(flight)
            }
        };
        let explosions = match tag.get("explosions") {
            None => Vec::new(),
            Some(Tag::List(values)) if values.len() <= Self::MAX_EXPLOSIONS => {
                values.iter().map(FireworkExplosion::from_nbt).collect::<Result<_, _>>()?
            }
            Some(_) => return Err("invalid firework explosions".into()),
        };
        Ok(Self { flight_duration, explosions })
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
