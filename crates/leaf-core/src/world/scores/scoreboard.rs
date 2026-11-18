//! Перенос `net.minecraft.world.scores.Scoreboard` (+ Score,
//! Objective, ScoreHolder, PlayerScoreEntry).

use crate::network::chat::component::Component;
use crate::world::entity::entity::Entity;
use crate::world::scores::display_slot::DisplaySlot;
use crate::world::scores::objective_criteria::{ObjectiveCriteria, RenderType as ObjectiveRenderType};
use crate::world::scores::team::PlayerTeam;
use std::collections::{HashMap, HashSet};

/// `ScoreHolder` — имя или entity для scoreboard-операций.
#[derive(Clone)]
pub enum ScoreHolder {
    /// `forNameOnly(name)`.
    Name(String),
    /// entity — `getScoreboardName()`.
    Entity(Entity),
}

pub const WILDCARD_NAME: &str = "*";

impl ScoreHolder {
    /// `forNameOnly`.
    pub fn for_name_only(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    /// `fromGameProfile` — имя профиля.
    pub fn from_name(name: impl Into<String>) -> Self {
        Self::Name(name.into())
    }

    /// `getScoreboardName()`.
    pub fn scoreboard_name(&self) -> String {
        match self {
            ScoreHolder::Name(n) => n.clone(),
            ScoreHolder::Entity(e) => e.get_scoreboard_name(),
        }
    }

    /// `getDisplayName()` — `None` для entity, literal для имени.
    pub fn display_name(&self) -> Option<Component> {
        match self {
            ScoreHolder::Name(n) => Some(Component::literal(n.clone())),
            ScoreHolder::Entity(_) => None,
        }
    }
}

/// `ReadOnlyScoreInfo`.
pub trait ReadOnlyScoreInfo {
    /// `value()`.
    fn value(&self) -> i32;
    /// `isLocked()`.
    fn is_locked(&self) -> bool;
}

/// `Score` — значение счёта (value/locked/display/numberFormat).
#[derive(Clone, Debug, Default)]
pub struct Score {
    /// `value`.
    pub value: i32,
    /// `locked`.
    pub locked: bool,
    /// `display`.
    pub display: Option<Component>,
    /// `numberFormat`.
    pub number_format: Option<std::sync::Arc<dyn crate::world::scores::number_format::NumberFormat>>,
}

impl ReadOnlyScoreInfo for Score {
    fn value(&self) -> i32 { self.value }
    fn is_locked(&self) -> bool { self.locked }
}

/// `PlayerScoreEntry`.
#[derive(Clone, Debug)]
pub struct PlayerScoreEntry {
    /// `owner`.
    pub owner: String,
    /// `value`.
    pub value: i32,
    /// `display`.
    pub display: Option<Component>,
    /// `numberFormatOverride`.
    pub number_format_override: Option<std::sync::Arc<dyn crate::world::scores::number_format::NumberFormat>>,
}

impl PlayerScoreEntry {
    /// `isHidden()`.
    pub fn is_hidden(&self) -> bool {
        self.owner.starts_with('#')
    }
    /// `ownerName()`.
    pub fn owner_name(&self) -> Component {
        self.display.clone().unwrap_or_else(|| Component::literal(self.owner.clone()))
    }
}

/// `Objective` — цель счёта.
#[derive(Clone, Debug)]
pub struct Objective {
    /// `name`.
    pub name: String,
    /// `displayName`.
    pub display_name: Component,
    /// `renderType`.
    pub render_type: ObjectiveRenderType,
    /// `criteria`.
    pub criteria: ObjectiveCriteria,
}


/// Ключ счёта: `(scoreboard_name, objective_name)`.
pub type ScoreKey = (String, String);

/// `Scoreboard` — таблица счёта уровня.
#[derive(Default)]
pub struct Scoreboard {
    /// `objectives` — name -> Objective.
    objectives: HashMap<String, Objective>,
    /// `playerScores` — (holder, objective) -> Score.
    player_scores: HashMap<ScoreKey, Score>,
    /// `displayObjectives`.
    display_objectives: HashMap<DisplaySlot, String>,
    /// `playerTeams` — name -> Team.
    player_teams: HashMap<String, PlayerTeam>,
    /// отслеживаемые игроки для `entityRemoved`.
    tracked: HashSet<String>,
}

impl Scoreboard {
    /// `getObjective(name)`.
    pub fn get_objective(&self, name: &str) -> Option<&Objective> {
        self.objectives.get(name)
    }

    /// `addObjective(name, criteria, displayName, renderType)`.
    pub fn add_objective(
        &mut self,
        name: impl Into<String>,
        criteria: ObjectiveCriteria,
        display_name: Component,
        render_type: ObjectiveRenderType,
    ) -> &Objective {
        let objective = Objective {
            name: name.into(),
            display_name,
            render_type,
            criteria,
        };
        let name = objective.name.clone();
        self.objectives.insert(name.clone(), objective);
        self.objectives.get(&name).unwrap_or_else(|| unreachable!())
    }

    /// `getOrCreatePlayerScore(holder, objective)` — ScoreAccess.
    pub fn get_or_create_player_score(
        &mut self,
        holder: &ScoreHolder,
        objective: &Objective,
    ) -> Score {
        let key = (holder.scoreboard_name(), objective.name.clone());
        self.player_scores.entry(key).or_default().clone()
    }

    /// `getOrCreatePlayerScore` с записью — возвращает Score (копия).
    pub fn score_mut(
        &mut self,
        holder: &ScoreHolder,
        objective: &Objective,
    ) -> &mut Score {
        let key = (holder.scoreboard_name(), objective.name.clone());
        self.player_scores.entry(key).or_default()
    }

    /// `getPlayerScoreInfo(holder, objective)` — `ReadOnlyScoreInfo`.
    pub fn get_player_score_info(
        &self,
        holder_name: &str,
        objective_name: &str,
    ) -> Option<Score> {
        self.player_scores
            .get(&(holder_name.to_string(), objective_name.to_string()))
            .cloned()
    }

    /// `getPlayerScoreInfo(holder, objective)` по объектам.
    pub fn get_player_score_info_obj(
        &self,
        holder: &ScoreHolder,
        objective: &Objective,
    ) -> Option<Score> {
        self.get_player_score_info(&holder.scoreboard_name(), &objective.name)
    }

    /// `listPlayerScores(objective)` — PlayerScoreEntry'и.
    pub fn list_player_scores(&self, objective: &Objective) -> Vec<PlayerScoreEntry> {
        self.player_scores
            .iter()
            .filter(|((_, obj), _)| obj == &objective.name)
            .map(|((owner, _), score)| PlayerScoreEntry {
                owner: owner.clone(),
                value: score.value,
                display: score.display.clone(),
                number_format_override: score.number_format.clone(),
            })
            .collect()
    }

    /// `getObjectives()`.
    pub fn objectives(&self) -> impl Iterator<Item = &Objective> {
        self.objectives.values()
    }

    /// `getObjectiveNames()`.
    pub fn objective_names(&self) -> Vec<String> {
        self.objectives.keys().cloned().collect()
    }

    /// `getTrackedPlayers()`.
    pub fn tracked_players(&self) -> impl Iterator<Item = &String> {
        self.tracked.iter()
    }

    /// `resetAllPlayerScores(holder)`.
    pub fn reset_all_player_scores(&mut self, holder: &ScoreHolder) {
        let name = holder.scoreboard_name();
        self.player_scores.retain(|(h, _), _| h != &name);
        self.tracked.remove(&name);
    }

    /// `resetSinglePlayerScore(holder, objective)`.
    pub fn reset_single_player_score(&mut self, holder: &ScoreHolder, objective: &Objective) {
        let key = (holder.scoreboard_name(), objective.name.clone());
        self.player_scores.remove(&key);
    }

    /// `listPlayerScores(holder)` — Object2IntMap<Objective> → Vec<(name, value)>.
    pub fn list_player_scores_for(&self, holder: &ScoreHolder) -> Vec<(String, i32)> {
        let name = holder.scoreboard_name();
        self.player_scores
            .iter()
            .filter(|((h, _), _)| h == &name)
            .map(|((_, o), s)| (o.clone(), s.value))
            .collect()
    }

    /// `removeObjective(objective)`.
    pub fn remove_objective(&mut self, objective: &Objective) {
        self.objectives.remove(&objective.name);
        self.player_scores.retain(|(_, o), _| o != &objective.name);
        self.display_objectives.retain(|_, o| o != &objective.name);
    }

    /// `setDisplayObjective(slot, objective)`.
    pub fn set_display_objective(&mut self, slot: DisplaySlot, objective: Option<&Objective>) {
        match objective {
            Some(o) => { self.display_objectives.insert(slot, o.name.clone()); }
            None => { self.display_objectives.remove(&slot); }
        }
    }

    /// `getDisplayObjective(slot)`.
    pub fn get_display_objective(&self, slot: DisplaySlot) -> Option<&Objective> {
        self.display_objectives
            .get(&slot)
            .and_then(|n| self.objectives.get(n))
    }

    /// `getPlayerTeam(name)` / `addPlayerTeam` / `removePlayerTeam`.
    pub fn get_player_team(&self, name: &str) -> Option<&PlayerTeam> {
        self.player_teams.get(name)
    }
    pub fn add_player_team(&mut self, name: impl Into<String>) -> &PlayerTeam {
        self.player_teams.entry(name.into()).or_insert_with(|| PlayerTeam::new(String::new()))
    }
    pub fn remove_player_team(&mut self, name: &str) {
        self.player_teams.remove(name);
    }

    /// `getPlayersTeam(playerName)`.
    pub fn get_players_team(&self, player_name: &str) -> Option<&PlayerTeam> {
        self.player_teams.values().find(|t| t.players.contains(&player_name.to_string()))
    }

    /// `addPlayerToTeam`.
    pub fn add_player_to_team(&mut self, player_name: &str, team_name: &str) -> bool {
        if let Some(team) = self.player_teams.get_mut(team_name) {
            if !team.players.contains(&player_name.to_string()) {
                team.players.push(player_name.to_string());
            }
            true
        } else {
            false
        }
    }

    /// `removePlayerFromTeam`.
    pub fn remove_player_from_team(&mut self, player_name: &str) {
        for team in self.player_teams.values_mut() {
            team.players.retain(|p| p != player_name);
        }
    }

    /// `getTeamNames()`.
    pub fn team_names(&self) -> Vec<String> {
        self.player_teams.keys().cloned().collect()
    }

    /// `getPlayerTeams()`.
    pub fn player_teams(&self) -> impl Iterator<Item = &PlayerTeam> {
        self.player_teams.values()
    }

    /// `entityRemoved(entity)` — снять все счёта сущности.
    pub fn entity_removed(&mut self, entity: &Entity) {
        let name = entity.get_scoreboard_name();
        self.reset_all_player_scores(&ScoreHolder::for_name_only(name));
    }

    /// `onPlayerRemoved`.
    pub fn on_player_removed(&mut self, holder: &ScoreHolder) {
        self.reset_all_player_scores(holder);
    }
}

/// `NumberFormat` — формат отображения счёта.
pub mod number_format {
    /// `NumberFormat` — трейт форматирования.
    pub trait NumberFormat: std::fmt::Debug + Send + Sync {
        /// `format(value)`.
        fn format(&self, value: i32) -> String;
    }
}
