//! Перенос `net.minecraft.world.effect.MobEffectInstance` —
//! активный эффект на сущности.

use crate::core::holder::Holder;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::world::effect::mob_effect::MobEffect;

/// `MobEffectInstance.Details` — amplifier/duration/флаги +
/// рекурсивный hiddenEffect.
#[derive(Clone, Debug, PartialEq)]
pub struct MobEffectDetails {
    /// `amplifier`.
    pub amplifier: i32,
    /// `duration`.
    pub duration: i32,
    /// `ambient`.
    pub ambient: bool,
    /// `showParticles`.
    pub show_particles: bool,
    /// `showIcon`.
    pub show_icon: bool,
    /// `hiddenEffect` (optional, рекурсивный).
    pub hidden_effect: Option<Box<MobEffectDetails>>,
}

impl MobEffectDetails {
    /// `new`.
    pub fn new(
        amplifier: i32,
        duration: i32,
        ambient: bool,
        show_particles: bool,
        show_icon: bool,
        hidden_effect: Option<Box<MobEffectDetails>>,
    ) -> Self {
        Self { amplifier, duration, ambient, show_particles, show_icon, hidden_effect }
    }

    fn encode(&self, buf: &mut FriendlyByteBuf, depth: usize) {
        assert!(depth < 64, "mob effect details recursion too deep");
        buf.write_var_int(self.amplifier);
        buf.write_var_int(self.duration);
        buf.write_bool(self.ambient);
        buf.write_bool(self.show_particles);
        buf.write_bool(self.show_icon);
        match &self.hidden_effect {
            Some(h) => {
                buf.write_bool(true);
                h.encode(buf, depth + 1);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }

    fn decode(buf: &mut FriendlyByteBuf, depth: usize) -> Self {
        assert!(depth < 64, "mob effect details recursion too deep");
        let amplifier = buf.read_var_int();
        let duration = buf.read_var_int();
        let ambient = buf.read_bool();
        let show_particles = buf.read_bool();
        let show_icon = buf.read_bool();
        let hidden_effect = if buf.read_bool() {
            Some(Box::new(Self::decode(buf, depth + 1)))
        } else {
            None
        };
        Self { amplifier, duration, ambient, show_particles, show_icon, hidden_effect }
    }

    /// `STREAM_CODEC` (`StreamCodec.recursive` + increaseDepth —
    /// depth-учёт реализован счётчиком аргумента).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MobEffectDetails> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &MobEffectDetails| v.encode(buf, 0),
            |buf: &mut FriendlyByteBuf| MobEffectDetails::decode(buf, 0),
        )
    }
}

/// `MobEffectInstance` — (effect holder, details).
#[derive(Clone, Debug, PartialEq)]
pub struct MobEffectInstance {
    /// `effect` — holder эффекта.
    pub effect: Holder<MobEffect>,
    /// `details`.
    pub details: MobEffectDetails,
}

impl MobEffectInstance {
    /// `new`.
    pub fn new(effect: Holder<MobEffect>, details: MobEffectDetails) -> Self {
        Self { effect, details }
    }

    /// `getAmplifier`, `getDuration` и т.п. — делегаты.
    pub fn amplifier(&self) -> i32 {
        self.details.amplifier
    }
    pub fn duration(&self) -> i32 {
        self.details.duration
    }

    /// `STREAM_CODEC` — MobEffect holder + Details.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, MobEffectInstance> + Send + Sync + 'static {
        let effect_codec = crate::core::holder::holder_registry_stream_codec::<MobEffect>();
        let details_codec = MobEffectDetails::stream_codec();
        let effect_codec2 = crate::core::holder::holder_registry_stream_codec::<MobEffect>();
        let details_codec2 = MobEffectDetails::stream_codec();
        sync_fn(
            move |buf: &mut FriendlyByteBuf, v: &MobEffectInstance| {
                effect_codec.encode(buf, &v.effect);
                details_codec.encode(buf, &v.details);
            },
            move |buf: &mut FriendlyByteBuf| {
                let effect = effect_codec2.decode(buf);
                let details = details_codec2.decode(buf);
                MobEffectInstance::new(effect, details)
            },
        )
    }
}
