//! Перенос `net.minecraft.world.damagesource` — источники и типы урона.

pub mod damage_effects;
pub mod damage_scaling;
pub mod damage_source;
pub mod damage_type;
pub mod damage_types;
pub mod damage_sources;
pub mod combat_entry;
pub mod combat_rules;
pub mod combat_tracker;
pub mod death_message_type;
pub mod fall_location;

pub use combat_entry::CombatEntry;
pub use combat_rules::CombatRules;
pub use combat_tracker::CombatTracker;
pub use damage_effects::DamageEffects;
pub use fall_location::FallLocation;
pub use damage_scaling::DamageScaling;
pub use damage_source::DamageSource;
pub use damage_type::DamageType;
pub use damage_sources::DamageSources;
pub use death_message_type::DeathMessageType;
