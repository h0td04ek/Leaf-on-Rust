use std::time::Instant;

use leaf_core::command::brigadier::{CommandDispatcher, CommandExecutionError, CommandNode};
use leaf_core::command::brigadier_argument::{ArgumentType, ArgumentValue};
use leaf_core::commands::gamerule::{register_game_rule_command, GameRuleCommandSource};
use leaf_core::commands::world::{
    register_world_commands, TimeChange, WeatherParameters, WorldCommandSource,
};
use leaf_core::commands::CommandSource;
use leaf_core::network::chat::component::Component;
use leaf_core::network::protocol::game::{
    clientbound_change_difficulty_packet::ClientboundChangeDifficultyPacket,
    clientbound_game_event_packet::{
        ClientboundGameEventPacket, RAIN_LEVEL_CHANGE, START_RAINING, STOP_RAINING,
        THUNDER_LEVEL_CHANGE,
    },
    clientbound_set_time_packet::ClientboundSetTimePacket,
};
use leaf_core::random::{LegacyRandomSource, RandomSource};
use leaf_core::util::uuid::Uuid;
use leaf_core::world::difficulty::Difficulty;
use leaf_core::world::flag::feature_flag_set::FeatureFlagSet;
use leaf_core::world::level::game_type::GameType;
use leaf_core::world::level::gamerules::game_rules::{advance_time, advance_weather, block_drops};
use leaf_core::world::level::gamerules::GameRules;

use crate::audience::Audience;
use crate::wire::{PacketIds, Wire};

const SEED: i64 = 0;

pub enum PlayerCommand {
    Kit,
    SetMode(GameType),
}

pub struct Commands {
    pub dispatcher: CommandDispatcher<CommandWorld>,
    pub world: std::sync::Mutex<CommandWorld>,
}

impl Commands {
    pub fn new() -> Self {
        let features = FeatureFlagSet::empty();
        let mut dispatcher = CommandDispatcher::<CommandWorld>::new();
        register_world_commands(&mut dispatcher, true).expect("register world commands");
        register_game_rule_command(&mut dispatcher, features.clone()).expect("register gamerule");
        dispatcher
            .add_node(
                dispatcher.root(),
                CommandNode::literal("help").executes(|source: &mut CommandWorld, _| {
                    source.send_success(
                        Component::literal(
                            "/help, /time, /weather, /difficulty, /seed, /gamerule, /gamemode, /kit",
                        ),
                        false,
                    );
                    1
                }),
            )
            .expect("register help");
        dispatcher
            .add_node(
                dispatcher.root(),
                CommandNode::literal("kit").executes(|source: &mut CommandWorld, _| {
                    source.action = Some(PlayerCommand::Kit);
                    1
                }),
            )
            .expect("register kit");
        let mode = dispatcher
            .add_node(dispatcher.root(), CommandNode::literal("gamemode"))
            .expect("register gamemode");
        dispatcher
            .add_node(
                mode,
                CommandNode::argument("mode", ArgumentType::GameMode).executes(
                    |source: &mut CommandWorld, context| {
                        let Some(ArgumentValue::GameMode(mode)) = context.argument("mode") else {
                            unreachable!("registered GameMode argument");
                        };
                        source.action = Some(PlayerCommand::SetMode(*mode));
                        1
                    },
                ),
            )
            .expect("register gamemode argument");
        Self {
            dispatcher,
            world: std::sync::Mutex::new(CommandWorld::new(features)),
        }
    }

    pub fn execute(
        &self,
        command: &str,
        audience: &Audience,
        player: Uuid,
    ) -> Option<PlayerCommand> {
        let mut world = self.world.lock().unwrap();
        world.refresh(audience);
        let before = world.snapshot();
        if let Err(error) = self.dispatcher.execute_with_feedback(command, &mut world) {
            let message = match error {
                CommandExecutionError::Feedback(component) => *component,
                CommandExecutionError::Syntax(error) => Component::literal(format!(
                    "Unknown or incomplete command near character {}. Try /help.",
                    command.get(..error.cursor).map_or(0, |p| p.chars().count())
                )),
            };
            world.feedback.push((message, false));
        }
        let after = world.snapshot();
        after.broadcast_changes(before, audience);
        for (message, broadcast) in world.feedback.drain(..) {
            audience.chat(message, (!broadcast).then_some(player));
        }
        world.action.take()
    }
}

pub struct CommandWorld {
    started: Instant,
    day_start: i64,
    day_value: i64,
    rules: GameRules,
    random: LegacyRandomSource,
    weather: WeatherParameters,
    expires: Option<i64>,
    difficulty: Difficulty,
    feedback: Vec<(Component, bool)>,
    action: Option<PlayerCommand>,
}

impl CommandWorld {
    fn new(features: FeatureFlagSet) -> Self {
        Self {
            started: Instant::now(),
            day_start: 0,
            day_value: 6000,
            rules: GameRules::new(features),
            random: LegacyRandomSource::new(SEED),
            weather: WeatherParameters {
                clear_time: 0,
                rain_time: 0,
                raining: false,
                thundering: false,
            },
            expires: None,
            difficulty: Difficulty::Peaceful,
            feedback: vec![],
            action: None,
        }
    }

    pub fn drops_enabled(&self) -> bool {
        self.rules.get_or_default(&block_drops())
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            time: ClientboundSetTimePacket::new(self.game_time(), self.day_time(), self.daylight()),
            difficulty: self.difficulty,
            rain: self.weather.raining,
            thunder: self.weather.thundering,
            day_base: (self.day_start, self.day_value),
        }
    }

    fn daylight(&self) -> bool {
        self.rules.get_or_default(&advance_time())
    }

    pub fn refresh(&mut self, audience: &Audience) {
        if self.rules.get_or_default(&advance_weather())
            && self.expires.is_some_and(|end| end <= self.game_time())
        {
            let before = self.snapshot();
            self.weather.raining = false;
            self.weather.thundering = false;
            self.expires = None;
            self.snapshot().broadcast_changes(before, audience);
        }
    }
}

impl CommandSource for CommandWorld {
    fn has_permission(&self, level: u8) -> bool {
        level <= 2
    }
    fn send_success(&mut self, message: Component, broadcast: bool) {
        self.feedback.push((message, broadcast));
    }
}

impl WorldCommandSource for CommandWorld {
    fn day_time(&self) -> i64 {
        self.day_value
            + if self.daylight() {
                self.game_time() - self.day_start
            } else {
                0
            }
    }
    fn day_count(&self) -> i64 {
        self.day_time().div_euclid(24_000)
    }
    fn game_time(&self) -> i64 {
        (self.started.elapsed().as_millis() / 50) as i64
    }
    fn change_time(&mut self, change: TimeChange) {
        self.day_value = match change {
            TimeChange::Set(value) => i64::from(value),
            TimeChange::Add(value) => self.day_time() + i64::from(value),
        };
        self.day_start = self.game_time();
    }
    fn weather_random(&mut self) -> &mut dyn RandomSource {
        &mut self.random
    }
    fn set_weather(&mut self, parameters: WeatherParameters) {
        self.expires =
            Some(self.game_time() + i64::from(parameters.clear_time.max(parameters.rain_time)));
        self.weather = parameters;
    }
    fn difficulty(&self) -> Difficulty {
        self.difficulty
    }
    fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.difficulty = difficulty;
    }
    fn seed(&self) -> i64 {
        SEED
    }
}

impl GameRuleCommandSource for CommandWorld {
    fn game_rules(&self) -> &GameRules {
        &self.rules
    }
    fn game_rules_mut(&mut self) -> &mut GameRules {
        &mut self.rules
    }
    fn set_game_rule<T: Clone + Send + Sync + 'static>(
        &mut self,
        rule: &leaf_core::world::level::gamerules::GameRule<T>,
        value: T,
    ) -> T {
        if rule.get_identifier() == advance_time().get_identifier() {
            self.day_value = self.day_time();
            self.day_start = self.game_time();
        }
        self.rules.set(rule, value.clone());
        value
    }
}

#[derive(Clone, Copy)]
pub struct Snapshot {
    time: ClientboundSetTimePacket,
    difficulty: Difficulty,
    rain: bool,
    thunder: bool,
    day_base: (i64, i64),
}

impl Snapshot {
    pub fn send(self, wire: &mut Wire, ids: &PacketIds) -> std::io::Result<()> {
        wire.send(ids, &self.time)?;
        wire.send(
            ids,
            &ClientboundChangeDifficultyPacket::new(self.difficulty, false),
        )?;
        for packet in self.weather_packets() {
            wire.send(ids, &packet)?;
        }
        Ok(())
    }

    fn weather_packets(self) -> [ClientboundGameEventPacket; 3] {
        let state = if self.rain {
            START_RAINING
        } else {
            STOP_RAINING
        };
        [
            ClientboundGameEventPacket::new(state, 0.0),
            ClientboundGameEventPacket::new(RAIN_LEVEL_CHANGE, if self.rain { 1.0 } else { 0.0 }),
            ClientboundGameEventPacket::new(
                THUNDER_LEVEL_CHANGE,
                if self.thunder { 1.0 } else { 0.0 },
            ),
        ]
    }

    fn broadcast_changes(self, before: Self, audience: &Audience) {
        if self.day_base != before.day_base || self.time.tick_day_time != before.time.tick_day_time
        {
            audience.broadcast(self.time, None);
        }
        if self.difficulty != before.difficulty {
            audience.broadcast(
                ClientboundChangeDifficultyPacket::new(self.difficulty, false),
                None,
            );
        }
        if (self.rain, self.thunder) != (before.rain, before.thunder) {
            for packet in self.weather_packets() {
                audience.broadcast(packet, None);
            }
        }
    }
}
