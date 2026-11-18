//! Перенос `net.minecraft.world.damagesource` — источники и типы урона.

pub mod damage_effects;
pub mod damage_scaling;
pub mod damage_source;
pub mod damage_type;
pub mod damage_types;
pub mod damage_sources;
pub mod death_message_type;

pub use damage_effects::DamageEffects;
pub use damage_scaling::DamageScaling;
pub use damage_source::DamageSource;
pub use damage_type::DamageType;
pub use damage_sources::DamageSources;
pub use death_message_type::DeathMessageType;
