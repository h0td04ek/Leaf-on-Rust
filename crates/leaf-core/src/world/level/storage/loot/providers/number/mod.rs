//! Перенос `net.minecraft.world.level.storage.loot.providers.number`.

pub mod number_provider;
pub mod providers;

pub use number_provider::NumberProvider;
pub use providers::*;
