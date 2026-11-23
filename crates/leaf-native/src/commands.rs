use std::time::{Duration, Instant};

use leaf_core::command::arguments::vanilla::Coordinates;
use leaf_core::command::brigadier::{
    CommandContext, CommandDispatcher, CommandExecutionError, CommandNode, CommandSuggestion,
};
use leaf_core::command::brigadier_argument::{ArgumentType, ArgumentValue, StringMode};
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
    clientbound_ticking_state_packet::ClientboundTickingStatePacket,
    clientbound_ticking_step_packet::ClientboundTickingStepPacket,
};
use leaf_core::random::{LegacyRandomSource, RandomSource};
use leaf_core::resources::Identifier;
use leaf_core::util::uuid::Uuid;
use leaf_core::world::difficulty::Difficulty;
use leaf_core::world::flag::feature_flag_set::FeatureFlagSet;
use leaf_core::world::level::game_type::GameType;
use leaf_core::world::level::gamerules::game_rules::{
    advance_time, advance_weather, block_drops, mob_drops, spawn_mobs, tnt_explodes,
    tnt_explosion_drop_decay,
};
use leaf_core::world::level::gamerules::GameRules;

use crate::audience::Audience;
use crate::mob_types::{self, MobType};
use crate::mobs::{self, COMMAND_LIMIT};
use crate::operators::{Operators, ALL, GAMEMASTERS, OWNERS};
use crate::ticking::{Action as TickAction, Clock, TickState, ADMIN_PERMISSION, MAX_RATE};
use crate::wire::{PacketIds, Wire};

const SEED: i64 = 0;

pub enum PlayerCommand {
    Kit,
    SetMode(GameType, Option<String>),
    Summon {
        kind: &'static MobType,
        position: Option<Coordinates>,
        count: usize,
    },
}

pub struct Commands {
    pub dispatcher: CommandDispatcher<CommandWorld>,
    pub world: std::sync::Mutex<CommandWorld>,
}

impl Commands {
    pub fn new(operators: Operators) -> Self {
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
                            "/help, /kit. Operators: /gamemode, /summon, /tick, /time, /weather, /difficulty, /seed, /gamerule, /op, /deop",
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
            .add_node(
                dispatcher.root(),
                CommandNode::literal("gamemode")
                    .requires(|s: &CommandWorld| s.has_permission(GAMEMASTERS)),
            )
            .expect("register gamemode");
        let mode_argument = dispatcher
            .add_node(
                mode,
                CommandNode::argument("mode", ArgumentType::GameMode).executes(
                    |source: &mut CommandWorld, context| {
                        let Some(ArgumentValue::GameMode(mode)) = context.argument("mode") else {
                            unreachable!("registered GameMode argument");
                        };
                        source.action = Some(PlayerCommand::SetMode(*mode, None));
                        1
                    },
                ),
            )
            .expect("register gamemode argument");
        dispatcher
            .add_node(
                mode_argument,
                CommandNode::argument("player", ArgumentType::String(StringMode::Word)).executes(
                    |source: &mut CommandWorld, context| {
                        let (
                            Some(ArgumentValue::GameMode(mode)),
                            Some(ArgumentValue::String(player)),
                        ) = (context.argument("mode"), context.argument("player"))
                        else {
                            unreachable!("registered game mode and player arguments")
                        };
                        source.action = Some(PlayerCommand::SetMode(*mode, Some(player.clone())));
                        1
                    },
                ),
            )
            .expect("register gamemode target");
        register_summon(&mut dispatcher);
        register_tick(&mut dispatcher);
        for (command, grant) in [("op", true), ("deop", false)] {
            let root = dispatcher
                .add_node(
                    dispatcher.root(),
                    CommandNode::literal(command)
                        .requires(|s: &CommandWorld| s.has_permission(OWNERS)),
                )
                .expect("register operator command");
            dispatcher
                .add_node(
                    root,
                    CommandNode::argument("player", ArgumentType::String(StringMode::Word))
                        .executes(move |source: &mut CommandWorld, context| {
                            let Some(ArgumentValue::String(player)) = context.argument("player")
                            else {
                                unreachable!("registered player name argument");
                            };
                            let (message, success) = match source.operators.change(player, grant) {
                                Ok(true) if grant => {
                                    (format!("Made {player} a server operator"), 1)
                                }
                                Ok(true) => (format!("Removed operator rights from {player}"), 1),
                                Ok(false) => {
                                    (format!("Operator rights for {player} are already set"), 0)
                                }
                                Err(error) => (error.to_string(), 0),
                            };
                            source.send_success(Component::literal(message), false);
                            success
                        }),
                )
                .expect("register player name argument");
        }
        Self {
            dispatcher,
            world: std::sync::Mutex::new(CommandWorld::new(features, operators)),
        }
    }

    pub fn game_time(&self) -> i64 {
        self.world.lock().unwrap().game_time()
    }

    pub fn for_player<T>(&self, name: &str, run: impl FnOnce(&CommandWorld) -> T) -> T {
        let mut world = self.world.lock().unwrap();
        world.permission = world.operators.level(name);
        let result = run(&world);
        world.permission = ALL;
        result
    }

    pub fn execute(
        &self,
        command: &str,
        audience: &Audience,
        player: Uuid,
        name: &str,
    ) -> Option<PlayerCommand> {
        let mut world = self.world.lock().unwrap();
        world.refresh(audience);
        world.permission = world.operators.level(name);
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
        world.permission = ALL;
        world.action.take()
    }
}

fn register_summon(dispatcher: &mut CommandDispatcher<CommandWorld>) {
    let suggestions = Identifier::minecraft("ask_server");
    dispatcher.register_suggestion_provider(suggestions.clone(), |_, _, prefix, entries| {
        let path = prefix.strip_prefix("minecraft:").unwrap_or(prefix);
        entries.extend(
            mob_types::names()
                .filter(|name| name.starts_with(path))
                .map(|name| CommandSuggestion {
                    text: format!("minecraft:{name}"),
                    tooltip: None,
                }),
        );
    });
    let summon = dispatcher
        .add_node(
            dispatcher.root(),
            CommandNode::literal("summon")
                .requires(|source: &CommandWorld| source.has_permission(GAMEMASTERS)),
        )
        .unwrap();
    let entity = dispatcher
        .add_node(
            summon,
            CommandNode::argument("entity", ArgumentType::Identifier)
                .with_suggestion_provider(suggestions)
                .executes(resolve_summon),
        )
        .unwrap();
    let pos = dispatcher
        .add_node(
            entity,
            CommandNode::argument("position", ArgumentType::Vec3).executes(resolve_summon),
        )
        .unwrap();
    dispatcher
        .add_node(
            pos,
            CommandNode::argument(
                "count",
                ArgumentType::Integer {
                    min: 1,
                    max: COMMAND_LIMIT,
                },
            )
            .executes(resolve_summon),
        )
        .unwrap();
}

fn resolve_summon(world: &mut CommandWorld, context: &CommandContext) -> i32 {
    let Some(ArgumentValue::Identifier(id)) = context.argument("entity") else {
        unreachable!("registered entity argument");
    };
    let Some(kind) = mob_types::get(id) else {
        world.send_success(Component::literal(format!("Cannot summon {id}")), false);
        return 0;
    };
    let position = match context.argument("position") {
        Some(ArgumentValue::Coordinates(position)) => Some(*position),
        _ => None,
    };
    let count = match context.argument("count") {
        Some(ArgumentValue::Integer(count)) => *count as usize,
        _ => 1,
    };
    world.action = Some(PlayerCommand::Summon {
        kind,
        position,
        count,
    });
    1
}

fn register_tick(dispatcher: &mut CommandDispatcher<CommandWorld>) {
    let root = dispatcher
        .add_node(
            dispatcher.root(),
            CommandNode::literal("tick")
                .requires(|source: &CommandWorld| source.has_permission(ADMIN_PERMISSION))
                .executes(|source: &mut CommandWorld, _| {
                    source.run_tick_command(TickAction::Query)
                }),
        )
        .unwrap();
    for (name, action) in [
        ("query", TickAction::Query),
        ("freeze", TickAction::Freeze(true)),
        ("unfreeze", TickAction::Freeze(false)),
    ] {
        dispatcher
            .add_node(
                root,
                CommandNode::literal(name)
                    .executes(move |source: &mut CommandWorld, _| source.run_tick_command(action)),
            )
            .unwrap();
    }
    let rate = dispatcher
        .add_node(root, CommandNode::literal("rate"))
        .unwrap();
    dispatcher
        .add_node(
            rate,
            CommandNode::argument(
                "rate",
                ArgumentType::Float {
                    min: 1.0,
                    max: MAX_RATE,
                },
            )
            .executes(|source: &mut CommandWorld, context| {
                let Some(ArgumentValue::Float(rate)) = context.argument("rate") else {
                    unreachable!("registered tick rate");
                };
                source.run_tick_command(TickAction::Rate(*rate))
            }),
        )
        .unwrap();
    let step = dispatcher
        .add_node(
            root,
            CommandNode::literal("step").executes(|source: &mut CommandWorld, _| {
                source.run_tick_command(TickAction::Step(1))
            }),
        )
        .unwrap();
    let sprint = dispatcher
        .add_node(root, CommandNode::literal("sprint"))
        .unwrap();
    let modes: [(_, fn(i32) -> TickAction, _); 2] = [
        (step, TickAction::Step, TickAction::StopStep),
        (sprint, TickAction::Sprint, TickAction::StopSprint),
    ];
    for (parent, action, stop) in modes {
        dispatcher
            .add_node(
                parent,
                CommandNode::literal("stop")
                    .executes(move |source: &mut CommandWorld, _| source.run_tick_command(stop)),
            )
            .unwrap();
        dispatcher
            .add_node(
                parent,
                CommandNode::argument("time", ArgumentType::Time { min: 1 }).executes(
                    move |source: &mut CommandWorld, context| {
                        let Some(ArgumentValue::Integer(ticks)) = context.argument("time") else {
                            unreachable!("registered tick duration");
                        };
                        source.run_tick_command(action(*ticks))
                    },
                ),
            )
            .unwrap();
    }
}

pub struct CommandWorld {
    clock: Clock,
    operators: Operators,
    permission: u8,
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
    fn new(features: FeatureFlagSet, operators: Operators) -> Self {
        Self {
            clock: Clock::new(Instant::now()),
            operators,
            permission: ALL,
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

    fn run_tick_command(&mut self, action: TickAction) -> i32 {
        let before = self.clock.state();
        let (message, result) = self.clock.execute(action, Instant::now());
        self.send_success(
            Component::literal(message),
            result != 0 && !matches!(action, TickAction::Query) && before != self.clock.state(),
        );
        result
    }

    pub fn poll_tick(
        &mut self,
        now: Instant,
        audience: &Audience,
    ) -> Result<(i64, bool, mobs::Rules), Duration> {
        let (tick, finished_step) = self.clock.poll(now)?;
        self.refresh(audience);
        Ok((tick, finished_step, self.mob_rules()))
    }

    pub fn complete_tick(
        &mut self,
        elapsed: Duration,
        now: Instant,
        finished_step: bool,
        audience: &Audience,
    ) {
        let before = self.snapshot();
        if let Some(summary) = self.clock.complete(elapsed, now) {
            let after = self.snapshot();
            after.broadcast_changes(before, audience);
            if after.tick_state.frozen == before.tick_state.frozen {
                audience.broadcast(after.time, None);
            }
            audience.chat(Component::literal(summary), None);
        } else if finished_step {
            audience.broadcast(self.snapshot().time, None);
        }
    }

    pub fn permission(&self) -> u8 {
        self.permission
    }

    pub fn mob_rules(&self) -> mobs::Rules {
        mobs::Rules {
            natural_spawns: self.rules.get_or_default(&spawn_mobs()),
            hostile_spawns: self.difficulty != Difficulty::Peaceful
                && (13_000..23_000).contains(&self.day_time().rem_euclid(24_000)),
            tnt: self.rules.get_or_default(&tnt_explodes()),
            block_drops: self.drops_enabled(),
            decay: self.rules.get_or_default(&tnt_explosion_drop_decay()),
        }
    }

    pub fn mob_drops(&self) -> bool {
        self.rules.get_or_default(&mob_drops())
    }

    pub fn drops_enabled(&self) -> bool {
        self.rules.get_or_default(&block_drops())
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            time: ClientboundSetTimePacket::new(self.game_time(), self.day_time(), self.daylight()),
            tick_state: self.clock.state(),
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
        level <= self.permission
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
        self.clock.time()
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
    tick_state: TickState,
    difficulty: Difficulty,
    rain: bool,
    thunder: bool,
    day_base: (i64, i64),
}

impl Snapshot {
    pub fn send(self, wire: &mut Wire, ids: &PacketIds) -> std::io::Result<()> {
        wire.send(ids, &self.time)?;
        wire.send(ids, &self.ticking_state_packet())?;
        if self.tick_state.steps > 0 {
            wire.send(
                ids,
                &ClientboundTickingStepPacket::new(self.tick_state.steps),
            )?;
        }
        wire.send(
            ids,
            &ClientboundChangeDifficultyPacket::new(self.difficulty, false),
        )?;
        for packet in self.weather_packets() {
            wire.send(ids, &packet)?;
        }
        Ok(())
    }

    fn ticking_state_packet(self) -> ClientboundTickingStatePacket {
        ClientboundTickingStatePacket::new(self.tick_state.rate, self.tick_state.frozen)
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
        let state_changed = self.tick_state.rate != before.tick_state.rate
            || self.tick_state.frozen != before.tick_state.frozen;
        if state_changed {
            audience.broadcast(self.ticking_state_packet(), None);
        }
        if self.tick_state.steps != before.tick_state.steps {
            audience.broadcast(
                ClientboundTickingStepPacket::new(self.tick_state.steps),
                None,
            );
        }
        if state_changed
            || self.day_base != before.day_base
            || self.time.tick_day_time != before.time.tick_day_time
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
