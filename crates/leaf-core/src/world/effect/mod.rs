//! Перенос `net.minecraft.world.effect`.

pub mod mob_effect;
pub use mob_effect::MobEffect;

pub mod mob_effect_instance;
pub use mob_effect_instance::{MobEffectDetails, MobEffectInstance};

pub mod mob_effects;
