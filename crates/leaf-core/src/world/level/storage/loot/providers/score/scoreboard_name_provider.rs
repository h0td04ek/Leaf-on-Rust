//! Перенос `net.minecraft.world.level.storage.loot.providers.score`
//! + `ScoreHolder` (`world.scores`).

use crate::world::entity::entity::Entity;
use crate::world::level::storage::loot::loot_context::{EntityTarget, LootContext};
use crate::world::level::storage::loot::parameters::loot_context_params;
use crate::resources::identifier::Identifier;

/// `ScoreHolder` — имя или entity для scoreboard-операций.
#[derive(Clone)]
pub enum ScoreHolder {
    /// `forNameOnly(name)`.
    Name(String),
    /// entity — `getScoreboardName()`.
    Entity(Entity),
}

impl ScoreHolder {
    /// `forNameOnly`.
    pub fn for_name_only(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    /// `getScoreboardName()` — имя в таблице счёта.
    pub fn scoreboard_name(&self) -> String {
        match self {
            ScoreHolder::Name(n) => n.clone(),
            ScoreHolder::Entity(e) => e.get_scoreboard_name(),
        }
    }
}

/// `ScoreboardNameProvider` — источник `ScoreHolder`.
#[derive(Clone, Debug)]
pub enum ScoreboardNameProvider {
    /// `fixed`.
    Fixed(FixedScoreboardNameProvider),
    /// `context`.
    Context(ContextScoreboardNameProvider),
}

fn key_for(target: EntityTarget) -> &'static crate::util::context::context_key::ContextKey<Entity> {
    match target {
        EntityTarget::This => loot_context_params::this_entity(),
        EntityTarget::Attacker => loot_context_params::attacking_entity(),
        EntityTarget::DirectAttacker => loot_context_params::direct_attacking_entity(),
        EntityTarget::AttackingPlayer => loot_context_params::last_damage_player(),
        EntityTarget::TargetEntity => loot_context_params::target_entity(),
        EntityTarget::InteractingEntity => loot_context_params::interacting_entity(),
    }
}

impl ScoreboardNameProvider {
    /// `getScoreHolder(context)`.
    pub fn get_score_holder(&self, ctx: &mut LootContext) -> Option<ScoreHolder> {
        match self {
            ScoreboardNameProvider::Fixed(f) => Some(ScoreHolder::for_name_only(f.name.clone())),
            ScoreboardNameProvider::Context(c) => {
                let key = key_for(c.target);
                ctx.get_optional_parameter(key).cloned().map(ScoreHolder::Entity)
            }
        }
    }

    /// `getReferencedContextParams()`.
    pub fn get_referenced_context_params(&self) -> Vec<Identifier> {
        match self {
            ScoreboardNameProvider::Fixed(_) => Vec::new(),
            ScoreboardNameProvider::Context(c) => {
                vec![key_for(c.target).name().clone()]
            }
        }
    }
}

/// `FixedScoreboardNameProvider`.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedScoreboardNameProvider {
    /// `name`.
    pub name: String,
}

/// `ContextScoreboardNameProvider`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ContextScoreboardNameProvider {
    /// `target`.
    pub target: EntityTarget,
}
