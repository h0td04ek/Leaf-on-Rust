//! Перенос `net.minecraft.world.item.alchemy.Potion`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::world::effect::mob_effect_instance::MobEffectInstance;

/// `Potion` — (name, effects, requiredFeatures).
#[derive(Clone, Debug, PartialEq)]
pub struct Potion {
    /// `name`.
    pub name: String,
    /// `effects`.
    pub effects: Vec<MobEffectInstance>,
    /// `requiredFeatures` — feature flags.
    pub required_features: crate::world::flag::feature_flag_set::FeatureFlagSet,
}

impl Potion {
    /// `new Potion(name, effects...)`.
    pub fn new(name: impl Into<String>, effects: Vec<MobEffectInstance>) -> Self {
        Self {
            name: name.into(),
            effects,
            required_features: crate::world::flag::feature_flag_set::FeatureFlagSet::all_vanilla(),
        }
    }

    /// `requiredFeatures(flags...)`.
    pub fn required_features(mut self, flags: crate::world::flag::feature_flag_set::FeatureFlagSet) -> Self {
        self.required_features = flags;
        self
    }

    /// `getEffects`.
    pub fn get_effects(&self) -> &[MobEffectInstance] {
        &self.effects
    }

    /// `name`.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// `hasInstantEffects`.
    pub fn has_instant_effects(&self) -> bool {
        self.effects
            .iter()
            .any(|e| e.effect.value().map(|m| m.is_instantaneous()).unwrap_or(false))
    }

    /// `Potion.STREAM_CODEC` — `holderRegistry(POTION)`.
    pub fn stream_codec() -> impl StreamCodec<crate::network::friendly_byte_buf::FriendlyByteBuf, crate::core::holder::Holder<Potion>> + Send + Sync + 'static {
        crate::core::holder::holder_registry_stream_codec::<Potion>()
    }
}
