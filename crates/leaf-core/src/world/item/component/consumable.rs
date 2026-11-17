//! Перенос `net.minecraft.world.item.component.Consumable`.

use crate::core::holder::Holder;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::sounds::sound_event::SoundEvent;
use crate::world::item::consume_effects::ConsumeEffect;
use crate::world::item::item_use_animation::ItemUseAnimation;

/// `Consumable` — consumeSeconds, animation, sound, particles,
/// onConsumeEffects.
#[derive(Clone, Debug, PartialEq)]
pub struct Consumable {
    /// `consumeSeconds`.
    pub consume_seconds: f32,
    /// `animation`.
    pub animation: ItemUseAnimation,
    /// `sound`.
    pub sound: Holder<SoundEvent>,
    /// `hasConsumeParticles`.
    pub has_consume_particles: bool,
    /// `onConsumeEffects`.
    pub on_consume_effects: Vec<ConsumeEffect>,
}

impl Consumable {
    /// `new`.
    pub fn new(
        consume_seconds: f32,
        animation: ItemUseAnimation,
        sound: Holder<SoundEvent>,
        has_consume_particles: bool,
        on_consume_effects: Vec<ConsumeEffect>,
    ) -> Self {
        Self { consume_seconds, animation, sound, has_consume_particles, on_consume_effects }
    }

    /// `STREAM_CODEC` — FLOAT + ItemUseAnimation + SoundEvent holder +
    /// BOOL + list(ConsumeEffect).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, Consumable> + Send + Sync + 'static {
        let effects_enc = byte_buf_codecs::list(ConsumeEffect::stream_codec());
        let effects_dec = byte_buf_codecs::list(ConsumeEffect::stream_codec());
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &Consumable| {
                buf.write_float(v.consume_seconds);
                ItemUseAnimation::stream_codec().encode(buf, &v.animation);
                SoundEvent::stream_codec().encode(buf, &v.sound);
                buf.write_bool(v.has_consume_particles);
                effects_enc.encode(buf, &v.on_consume_effects);
            },
            move |buf: &mut FriendlyByteBuf| {
                let consume_seconds = buf.read_float();
                let animation = ItemUseAnimation::stream_codec().decode(buf);
                let sound = SoundEvent::stream_codec().decode(buf);
                let has_consume_particles = buf.read_bool();
                let on_consume_effects = effects_dec.decode(buf);
                Consumable {
                    consume_seconds,
                    animation,
                    sound,
                    has_consume_particles,
                    on_consume_effects,
                }
            },
        )
    }
}
