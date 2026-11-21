//! `net.minecraft.world.level.LevelSettings` — настройки мира
//! из level.dat (`WorldGenSettings` + `Data` секция).

use crate::nbt::{CompoundTag, Tag};
use crate::world::difficulty::Difficulty;
use crate::world::level::game_type::GameType;
use crate::world::level::gamerules::{GameRules, bootstrap as gamerules_bootstrap};
use crate::world::level::world_data_configuration::WorldDataConfiguration;

/// `LevelSettings`.
#[derive(Clone, Debug)]
pub struct LevelSettings {
    /// `levelName`.
    pub level_name: String,
    game_type: GameType,
    /// `hardcore`.
    pub hardcore: bool,
    difficulty: Difficulty,
    allow_commands: bool,
    game_rules: GameRules,
    data_configuration: WorldDataConfiguration,
}

impl LevelSettings {
    /// `new LevelSettings(...)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        level_name: String,
        game_type: GameType,
        hardcore: bool,
        difficulty: Difficulty,
        allow_commands: bool,
        game_rules: GameRules,
        data_configuration: WorldDataConfiguration,
    ) -> Self {
        Self {
            level_name,
            game_type,
            hardcore,
            difficulty,
            allow_commands,
            game_rules,
            data_configuration,
        }
    }

    /// `LevelSettings.parse(Dynamic, WorldDataConfiguration)` —
    /// из NBT level.dat `Data`-секции.
    pub fn parse(tag: &Tag, data_configuration: WorldDataConfiguration) -> Self {
        gamerules_bootstrap();
        let empty = CompoundTag::new();
        let c: &CompoundTag = match tag {
            Tag::Compound(c) => c,
            _ => &empty,
        };
        let game_type = GameType::by_id(c.get_int("GameType").unwrap_or(0));
        let level_name = c
            .get_string("LevelName")
            .unwrap_or("")
            .to_string();
        let hardcore = c.get_bool("hardcore").unwrap_or(false);
        let difficulty = c
            .get("Difficulty")
            .and_then(|t| match *t {
                Tag::Byte(b) => Some(b as i32),
                Tag::Int(i) => Some(i),
                _ => None,
            })
            .map(Difficulty::by_id)
            .unwrap_or(Difficulty::Normal);
        let allow_commands = c
            .get_bool("allowCommands")
            .unwrap_or(game_type == GameType::Creative);
        let game_rules = c
            .get("game_rules")
            .map(|t| {
                GameRules::from_nbt(
                    t,
                    data_configuration.enabled_features.clone(),
                )
            })
            .unwrap_or_else(|| GameRules::new(data_configuration.enabled_features.clone()));
        Self::new(
            level_name,
            game_type,
            hardcore,
            difficulty,
            allow_commands,
            game_rules,
            data_configuration,
        )
    }

    /// `levelName()`.
    pub fn level_name(&self) -> &str {
        &self.level_name
    }

    /// `gameType()`.
    pub fn game_type(&self) -> GameType {
        self.game_type
    }

    /// `hardcore()`.
    pub fn hardcore(&self) -> bool {
        self.hardcore
    }

    /// `difficulty()`.
    pub fn difficulty(&self) -> Difficulty {
        self.difficulty
    }

    /// `allowCommands()`.
    pub fn allow_commands(&self) -> bool {
        self.allow_commands
    }

    /// `gameRules()`.
    pub fn game_rules(&self) -> &GameRules {
        &self.game_rules
    }

    /// `gameRules()` mutable.
    pub fn game_rules_mut(&mut self) -> &mut GameRules {
        &mut self.game_rules
    }

    /// `getDataConfiguration()`.
    pub fn get_data_configuration(&self) -> &WorldDataConfiguration {
        &self.data_configuration
    }

    /// `withGameType(gameType)`.
    pub fn with_game_type(&self, game_type: GameType) -> Self {
        Self::new(
            self.level_name.clone(),
            game_type,
            self.hardcore,
            self.difficulty,
            self.allow_commands,
            self.game_rules.clone(),
            self.data_configuration.clone(),
        )
    }

    /// `withDifficulty(difficulty)`.
    pub fn with_difficulty(&self, difficulty: Difficulty) -> Self {
        Self::new(
            self.level_name.clone(),
            self.game_type,
            self.hardcore,
            difficulty,
            self.allow_commands,
            self.game_rules.clone(),
            self.data_configuration.clone(),
        )
    }

    /// `withDataConfiguration(dataConfiguration)`.
    pub fn with_data_configuration(&self, data_configuration: WorldDataConfiguration) -> Self {
        Self::new(
            self.level_name.clone(),
            self.game_type,
            self.hardcore,
            self.difficulty,
            self.allow_commands,
            self.game_rules.clone(),
            data_configuration,
        )
    }

    /// `copy()` — `gameRules.copy(enabledFeatures)`.
    pub fn copy(&self) -> Self {
        Self::new(
            self.level_name.clone(),
            self.game_type,
            self.hardcore,
            self.difficulty,
            self.allow_commands,
            self.game_rules
                .copy(self.data_configuration.enabled_features.clone()),
            self.data_configuration.clone(),
        )
    }
}
