use crate::command::brigadier::{
    CommandContext, CommandDispatcher, CommandNode, RegistrationError,
};
use crate::command::brigadier_argument::{ArgumentType, ArgumentValue};
use crate::network::chat::{
    chat_formatting::ChatFormatting, click_event::ClickEvent, component::Component,
    component_contents::TranslatableArg, hover_event::HoverEvent,
};
use crate::random::RandomSource;
use crate::util::mth::random_between_inclusive;
use crate::world::difficulty::Difficulty;

use super::CommandSource;

const GAMEMASTERS: u8 = 2;
const TICKS_PER_DAY: i64 = 24_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimeChange {
    Set(i32),
    Add(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeatherParameters {
    pub clear_time: i32,
    pub rain_time: i32,
    pub raining: bool,
    pub thundering: bool,
}

pub trait WorldCommandSource: CommandSource {
    fn day_time(&self) -> i64;
    fn day_count(&self) -> i64;
    fn game_time(&self) -> i64;
    /// Applies Paper's world scope for time changes and synchronizes clients.
    fn change_time(&mut self, change: TimeChange);
    fn weather_random(&mut self) -> &mut dyn RandomSource;
    fn set_weather(&mut self, parameters: WeatherParameters);
    fn difficulty(&self) -> Difficulty;
    fn set_difficulty(&mut self, difficulty: Difficulty);
    fn seed(&self) -> i64;
    fn feature_seed(&self) -> Option<String> {
        None
    }
}

pub fn register_world_commands<S: WorldCommandSource>(
    dispatcher: &mut CommandDispatcher<S>,
    dedicated_server: bool,
) -> Result<(), RegistrationError> {
    register_time(dispatcher)?;
    register_weather(dispatcher)?;
    register_difficulty(dispatcher)?;
    let permission = if dedicated_server { GAMEMASTERS } else { 0 };
    dispatcher.add_node(
        dispatcher.root(),
        CommandNode::literal("seed")
            .requires(move |source: &S| source.has_permission(permission))
            .executes(|source, _| {
                let seed = source.seed();
                source.send_success(
                    copyable_message("commands.seed.success", seed.to_string()),
                    false,
                );
                if let Some(feature_seed) = source.feature_seed() {
                    source.send_success(copyable_message("Feature seed: %s", feature_seed), false);
                }
                seed as i32
            }),
    )?;
    Ok(())
}

fn register_time<S: WorldCommandSource>(
    dispatcher: &mut CommandDispatcher<S>,
) -> Result<(), RegistrationError> {
    let time = dispatcher.add_node(
        dispatcher.root(),
        CommandNode::literal("time").requires(|source: &S| source.has_permission(GAMEMASTERS)),
    )?;
    let set = dispatcher.add_node(time, CommandNode::literal("set"))?;
    for (name, ticks) in [
        ("day", 1000),
        ("noon", 6000),
        ("night", 13000),
        ("midnight", 18000),
    ] {
        dispatcher.add_node(
            set,
            CommandNode::literal(name)
                .executes(move |source, _| change_time(source, TimeChange::Set(ticks))),
        )?;
    }
    let add = dispatcher.add_node(time, CommandNode::literal("add"))?;
    dispatcher.add_node(
        set,
        CommandNode::argument("time", ArgumentType::Time { min: 0 }).executes(|source, context| {
            change_time(source, TimeChange::Set(ticks(context, "time")))
        }),
    )?;
    dispatcher.add_node(
        add,
        CommandNode::argument("time", ArgumentType::Time { min: 0 }).executes(|source, context| {
            change_time(source, TimeChange::Add(ticks(context, "time")))
        }),
    )?;
    let query = dispatcher.add_node(time, CommandNode::literal("query"))?;
    dispatcher.add_node(
        query,
        CommandNode::literal("daytime").executes(|source: &mut S, _| {
            send_time(source, "commands.time.query", daytime(source), false)
        }),
    )?;
    dispatcher.add_node(
        query,
        CommandNode::literal("day").executes(|source: &mut S, _| {
            send_time(
                source,
                "commands.time.query",
                (source.day_count() % i64::from(i32::MAX)) as i32,
                false,
            )
        }),
    )?;
    dispatcher.add_node(
        query,
        CommandNode::literal("gametime").executes(|source: &mut S, _| {
            send_time(
                source,
                "commands.time.query",
                (source.game_time() % i64::from(i32::MAX)) as i32,
                false,
            )
        }),
    )?;
    Ok(())
}

fn ticks(context: &CommandContext, name: &str) -> i32 {
    let Some(ArgumentValue::Integer(ticks)) = context.argument(name) else {
        unreachable!("registered Time argument must be a tick count");
    };
    *ticks
}

fn daytime(source: &impl WorldCommandSource) -> i32 {
    (source.day_time() % TICKS_PER_DAY) as i32
}

fn send_time(source: &mut impl WorldCommandSource, key: &str, time: i32, broadcast: bool) -> i32 {
    source.send_success(
        Component::translatable(key, vec![TranslatableArg::from_display(&time)]),
        broadcast,
    );
    time
}

fn change_time(source: &mut impl WorldCommandSource, change: TimeChange) -> i32 {
    source.change_time(change);
    let current = daytime(source);
    let time = match change {
        TimeChange::Set(ticks) => ticks,
        TimeChange::Add(_) => current,
    };
    send_time(source, "commands.time.set", time, true);
    current
}

#[derive(Clone, Copy)]
enum Weather {
    Clear,
    Rain,
    Thunder,
}

impl Weather {
    fn set(self, source: &mut impl WorldCommandSource, duration: Option<i32>) -> i32 {
        let (min, max, key) = match self {
            Self::Clear => (12000, 180000, "commands.weather.set.clear"),
            Self::Rain => (12000, 24000, "commands.weather.set.rain"),
            Self::Thunder => (3600, 15600, "commands.weather.set.thunder"),
        };
        let time =
            duration.unwrap_or_else(|| random_between_inclusive(source.weather_random(), min, max));
        let raining = !matches!(self, Self::Clear);
        source.set_weather(WeatherParameters {
            clear_time: if raining { 0 } else { time },
            rain_time: if raining { time } else { 0 },
            raining,
            thundering: matches!(self, Self::Thunder),
        });
        source.send_success(Component::translatable(key, Vec::new()), true);
        duration.unwrap_or(-1)
    }
}

fn register_weather<S: WorldCommandSource>(
    dispatcher: &mut CommandDispatcher<S>,
) -> Result<(), RegistrationError> {
    let weather = dispatcher.add_node(
        dispatcher.root(),
        CommandNode::literal("weather").requires(|source: &S| source.has_permission(GAMEMASTERS)),
    )?;
    for (name, kind) in [
        ("clear", Weather::Clear),
        ("rain", Weather::Rain),
        ("thunder", Weather::Thunder),
    ] {
        let node = dispatcher.add_node(
            weather,
            CommandNode::literal(name).executes(move |source, _| kind.set(source, None)),
        )?;
        dispatcher.add_node(
            node,
            CommandNode::argument("duration", ArgumentType::Time { min: 1 }).executes(
                move |source, context| kind.set(source, Some(ticks(context, "duration"))),
            ),
        )?;
    }
    Ok(())
}

fn register_difficulty<S: WorldCommandSource>(
    dispatcher: &mut CommandDispatcher<S>,
) -> Result<(), RegistrationError> {
    let root = dispatcher.add_node(
        dispatcher.root(),
        CommandNode::literal("difficulty")
            .requires(|source: &S| source.has_permission(GAMEMASTERS))
            .executes(|source, _| {
                let difficulty = source.difficulty();
                send_difficulty(source, "commands.difficulty.query", difficulty, false);
                difficulty.get_id()
            }),
    )?;
    for &difficulty in Difficulty::values() {
        dispatcher.add_node(
            root,
            CommandNode::literal(difficulty.get_key()).try_executes(move |source: &mut S, _| {
                if source.difficulty() == difficulty {
                    return Err(Box::new(Component::translatable(
                        "commands.difficulty.failure",
                        vec![TranslatableArg::from_display(&difficulty.get_key())],
                    )));
                }
                source.set_difficulty(difficulty);
                send_difficulty(source, "commands.difficulty.success", difficulty, true);
                Ok(0)
            }),
        )?;
    }
    Ok(())
}

fn send_difficulty(
    source: &mut impl WorldCommandSource,
    key: &str,
    difficulty: Difficulty,
    broadcast: bool,
) {
    source.send_success(
        Component::translatable(
            key,
            vec![TranslatableArg::Component(Box::new(
                difficulty.get_display_name(),
            ))],
        ),
        broadcast,
    );
}

fn copyable_message(key: &str, text: String) -> Component {
    let mut value = Component::literal(&text);
    value.with_style_fn(|style| {
        style
            .with_color_formatting(ChatFormatting::Green)
            .with_click_event(Some(ClickEvent::CopyToClipboard(text.clone())))
            .with_hover_event(Some(HoverEvent::ShowText(Box::new(
                Component::translatable("chat.copy.click", Vec::new()),
            ))))
            .with_insertion(Some(text))
    });
    let brackets = Component::translatable(
        "chat.square_brackets",
        vec![TranslatableArg::Component(Box::new(value))],
    );
    Component::translatable(key, vec![TranslatableArg::Component(Box::new(brackets))])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::brigadier::{CommandErrorKind, CommandExecutionError};
    use crate::network::chat::component_contents::ComponentContents;
    use crate::random::LegacyRandomSource;

    struct Source {
        permission: u8,
        day_time: i64,
        game_time: i64,
        weather: Option<WeatherParameters>,
        random: LegacyRandomSource,
        difficulty: Difficulty,
        feedback: Vec<(Component, bool)>,
    }

    impl Source {
        fn new(permission: u8) -> Self {
            Self {
                permission,
                day_time: 0,
                game_time: i64::from(i32::MAX) + 99,
                weather: None,
                random: LegacyRandomSource::new(42),
                difficulty: Difficulty::Normal,
                feedback: Vec::new(),
            }
        }
    }

    impl CommandSource for Source {
        fn has_permission(&self, level: u8) -> bool {
            self.permission >= level
        }

        fn send_success(&mut self, message: Component, broadcast: bool) {
            self.feedback.push((message, broadcast));
        }
    }

    impl WorldCommandSource for Source {
        fn day_time(&self) -> i64 {
            self.day_time
        }

        fn day_count(&self) -> i64 {
            self.day_time.div_euclid(TICKS_PER_DAY)
        }

        fn game_time(&self) -> i64 {
            self.game_time
        }

        fn change_time(&mut self, change: TimeChange) {
            match change {
                TimeChange::Set(value) => self.day_time = i64::from(value),
                TimeChange::Add(value) => self.day_time += i64::from(value),
            }
        }

        fn weather_random(&mut self) -> &mut dyn RandomSource {
            &mut self.random
        }

        fn set_weather(&mut self, parameters: WeatherParameters) {
            self.weather = Some(parameters);
        }

        fn difficulty(&self) -> Difficulty {
            self.difficulty
        }

        fn set_difficulty(&mut self, difficulty: Difficulty) {
            self.difficulty = difficulty;
        }

        fn seed(&self) -> i64 {
            i64::from(u32::MAX)
        }

        fn feature_seed(&self) -> Option<String> {
            Some("[42]".into())
        }
    }

    fn dispatcher(dedicated: bool) -> CommandDispatcher<Source> {
        let mut dispatcher = CommandDispatcher::new();
        register_world_commands(&mut dispatcher, dedicated).unwrap();
        dispatcher
    }

    #[test]
    fn time_and_weather_apply_world_values_and_preserve_command_results() {
        let dispatcher = dispatcher(true);
        let mut source = Source::new(GAMEMASTERS);
        assert_eq!(
            dispatcher.execute("time set midnight", &mut source),
            Ok(18000)
        );
        assert_eq!(dispatcher.execute("time set 1.5d", &mut source), Ok(12000));
        assert_eq!(source.day_time, 36000);
        assert_eq!(
            source.feedback.last(),
            Some(&(
                Component::translatable(
                    "commands.time.set",
                    vec![TranslatableArg::Text("36000".into())]
                ),
                true
            )),
        );
        assert_eq!(dispatcher.execute("time add 20s", &mut source), Ok(12400));
        assert_eq!(
            dispatcher.execute("time query daytime", &mut source),
            Ok(12400)
        );
        assert_eq!(dispatcher.execute("time query day", &mut source), Ok(1));
        assert_eq!(
            dispatcher.execute("time query gametime", &mut source),
            Ok(99)
        );
        assert!(!source.feedback.last().unwrap().1);

        assert_eq!(
            dispatcher.execute("weather thunder 0.5s", &mut source),
            Ok(10)
        );
        assert_eq!(
            source.weather,
            Some(WeatherParameters {
                clear_time: 0,
                rain_time: 10,
                raining: true,
                thundering: true
            }),
        );
        assert!(dispatcher.execute("weather rain 0t", &mut source).is_err());
        let expected = random_between_inclusive(&mut LegacyRandomSource::new(42), 12000, 180000);
        assert_eq!(dispatcher.execute("weather clear", &mut source), Ok(-1));
        assert_eq!(
            source.weather,
            Some(WeatherParameters {
                clear_time: expected,
                rain_time: 0,
                raining: false,
                thundering: false
            }),
        );
    }

    #[test]
    fn difficulty_and_seed_check_permissions_and_preserve_localized_feedback() {
        let dedicated = dispatcher(true);
        let mut player = Source::new(0);
        for command in ["seed", "time set day", "weather rain", "difficulty"] {
            assert!(dedicated.execute(command, &mut player).is_err());
        }
        assert!(dedicated.complete("/", 1, &player).entries.is_empty());
        assert_eq!(dispatcher(false).execute("seed", &mut player), Ok(-1));
        let [(message, false), (feature_message, false)] = player.feedback.as_slice() else {
            panic!("expected world and feature seed feedback");
        };
        let ComponentContents::Translatable(seed) = &message.contents else {
            panic!("expected translated seed feedback");
        };
        let [TranslatableArg::Component(brackets)] = seed.args.as_slice() else {
            panic!("expected clickable seed");
        };
        let ComponentContents::Translatable(brackets) = &brackets.contents else {
            panic!("expected bracket style");
        };
        let [TranslatableArg::Component(value)] = brackets.args.as_slice() else {
            panic!("expected text to copy");
        };
        assert_eq!(seed.key, "commands.seed.success");
        assert_eq!(
            value.style.click_event,
            Some(ClickEvent::CopyToClipboard(u32::MAX.to_string()))
        );
        assert!(
            matches!(&feature_message.contents, ComponentContents::Translatable(feature) if feature.key == "Feature seed: %s")
        );

        let mut operator = Source::new(GAMEMASTERS);
        assert_eq!(dedicated.execute("difficulty", &mut operator), Ok(2));
        assert_eq!(dedicated.execute("difficulty hard", &mut operator), Ok(0));
        assert_eq!(operator.difficulty, Difficulty::Hard);
        assert_eq!(
            dedicated.execute_with_feedback("difficulty hard", &mut operator),
            Err(CommandExecutionError::Feedback(Box::new(
                Component::translatable(
                    "commands.difficulty.failure",
                    vec![TranslatableArg::Text("hard".into())],
                )
            ))),
        );
        assert_eq!(
            dedicated
                .execute("difficulty hard", &mut operator)
                .unwrap_err()
                .kind,
            CommandErrorKind::ExecutionFailed
        );
        assert_eq!(operator.feedback.len(), 2);
    }
}
