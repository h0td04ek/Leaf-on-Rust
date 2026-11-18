//! Перенос `net.minecraft.world.level.storage.loot.providers.score`.

pub mod scoreboard_name_provider;

pub use scoreboard_name_provider::{
    ContextScoreboardNameProvider, FixedScoreboardNameProvider, ScoreHolder,
    ScoreboardNameProvider,
};
