//! World game rule queries and changes.

use crate::command::brigadier::{CommandDispatcher, CommandNode, NodeId, RegistrationError};
use crate::command::brigadier_argument::{ArgumentType, ArgumentValue};
use crate::network::chat::{component::Component, component_contents::TranslatableArg};
use crate::world::flag::feature_flag_set::FeatureFlagSet;
use crate::world::level::gamerules::{GameRule, GameRuleArgument, GameRuleTypeVisitor, GameRules};

use super::CommandSource;

pub trait GameRuleCommandSource: CommandSource {
    fn game_rules(&self) -> &GameRules;
    fn game_rules_mut(&mut self) -> &mut GameRules;

    fn set_game_rule<T: Clone + Send + Sync + 'static>(
        &mut self,
        rule: &GameRule<T>,
        value: T,
    ) -> T {
        self.game_rules_mut().set(rule, value.clone());
        value
    }
}

pub fn register_game_rule_command<S: GameRuleCommandSource>(
    dispatcher: &mut CommandDispatcher<S>,
    enabled_features: FeatureFlagSet,
) -> Result<(), RegistrationError> {
    let root = dispatcher.add_node(
        dispatcher.root(),
        CommandNode::literal("gamerule").requires(|source: &S| source.has_permission(2)),
    )?;
    let mut registration = Registration {
        dispatcher,
        root,
        result: Ok(()),
    };
    GameRules::new(enabled_features).visit_game_rule_types(&mut registration);
    registration.result
}

struct Registration<'a, S> {
    dispatcher: &'a mut CommandDispatcher<S>,
    root: NodeId,
    result: Result<(), RegistrationError>,
}

impl<S: GameRuleCommandSource> Registration<'_, S> {
    fn register<T: Clone + Send + Sync + 'static>(
        &mut self,
        rule: &GameRule<T>,
        read_value: fn(&ArgumentValue) -> T,
    ) {
        if self.result.is_err() {
            return;
        }

        let name = rule.id();
        let qualified = rule.get_identifier().to_string();
        self.result = self.add_rule(&name, rule, read_value);
        if self.result.is_ok() && name != qualified {
            self.result = self.add_rule(&qualified, rule, read_value);
        }
    }

    fn add_rule<T: Clone + Send + Sync + 'static>(
        &mut self,
        name: &str,
        rule: &GameRule<T>,
        read_value: fn(&ArgumentValue) -> T,
    ) -> Result<(), RegistrationError> {
        let query_rule = rule.clone();
        let literal = self.dispatcher.add_node(
            self.root,
            CommandNode::literal(name).executes(move |source: &mut S, _| {
                let value = source.game_rules().get_or_default(&query_rule);
                send_result(
                    source,
                    "commands.gamerule.query",
                    &query_rule,
                    &value,
                    false,
                )
            }),
        )?;
        let argument = match *rule.argument() {
            GameRuleArgument::Bool => ArgumentType::Bool,
            GameRuleArgument::Int { min, max } => ArgumentType::Integer { min, max },
        };
        let rule = rule.clone();
        self.dispatcher.add_node(
            literal,
            CommandNode::argument("value", argument).executes(move |source: &mut S, context| {
                let value = read_value(
                    context
                        .argument("value")
                        .expect("registered game rule argument"),
                );
                let value = source.set_game_rule(&rule, value);
                send_result(source, "commands.gamerule.set", &rule, &value, true)
            }),
        )?;
        Ok(())
    }
}

impl<S: GameRuleCommandSource> GameRuleTypeVisitor for Registration<'_, S> {
    fn visit_boolean(&mut self, rule: &GameRule<bool>) {
        self.register(rule, |value| {
            let ArgumentValue::Bool(value) = value else {
                unreachable!("boolean rule")
            };
            *value
        });
    }

    fn visit_integer(&mut self, rule: &GameRule<i32>) {
        self.register(rule, |value| {
            let ArgumentValue::Integer(value) = value else {
                unreachable!("integer rule")
            };
            *value
        });
    }
}

fn send_result<T: Clone + Send + Sync + 'static>(
    source: &mut impl GameRuleCommandSource,
    key: &str,
    rule: &GameRule<T>,
    value: &T,
    broadcast: bool,
) -> i32 {
    source.send_success(
        Component::translatable(
            key,
            vec![
                TranslatableArg::Text(rule.id()),
                TranslatableArg::Text(rule.serialize(value)),
            ],
        ),
        broadcast,
    );
    rule.get_command_result(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::level::gamerules::game_rules::{keep_inventory, random_tick_speed};

    struct Source {
        rules: GameRules,
        messages: Vec<(Component, bool)>,
        operator: bool,
    }

    impl CommandSource for Source {
        fn has_permission(&self, level: u8) -> bool {
            self.operator || level == 0
        }

        fn send_success(&mut self, message: Component, broadcast: bool) {
            self.messages.push((message, broadcast));
        }
    }

    impl GameRuleCommandSource for Source {
        fn game_rules(&self) -> &GameRules {
            &self.rules
        }

        fn game_rules_mut(&mut self) -> &mut GameRules {
            &mut self.rules
        }
    }

    #[test]
    fn mutates_only_the_source_world_and_uses_registry_types_and_feature_flags() {
        let mut dispatcher = CommandDispatcher::new();
        let features = FeatureFlagSet::empty();
        register_game_rule_command(&mut dispatcher, features.clone()).unwrap();
        let mut source = Source {
            rules: GameRules::new(features.clone()),
            messages: Vec::new(),
            operator: false,
        };
        let other_world = GameRules::new(features);
        assert!(dispatcher
            .execute("gamerule keep_inventory true", &mut source)
            .is_err());
        assert!(dispatcher.complete("/", 1, &source).entries.is_empty());
        source.operator = true;
        assert_eq!(
            dispatcher.execute("gamerule keep_inventory", &mut source),
            Ok(0)
        );
        assert_eq!(
            dispatcher.execute("gamerule minecraft:keep_inventory true", &mut source),
            Ok(1)
        );
        assert_eq!(
            dispatcher.execute("gamerule keep_inventory", &mut source),
            Ok(1)
        );
        assert_eq!(
            dispatcher.execute("gamerule random_tick_speed 42", &mut source),
            Ok(42)
        );
        assert!(dispatcher
            .execute("gamerule random_tick_speed -1", &mut source)
            .is_err());
        assert!(dispatcher
            .execute("gamerule max_minecart_speed", &mut source)
            .is_err());
        assert_eq!(source.rules.get(&keep_inventory()), Some(true));
        assert_eq!(other_world.get(&keep_inventory()), Some(false));
        assert_eq!(source.rules.get(&random_tick_speed()), Some(42));
        assert_eq!(
            source
                .messages
                .iter()
                .map(|(_, broadcast)| *broadcast)
                .collect::<Vec<_>>(),
            [false, true, false, true]
        );
        assert_eq!(
            source.messages[1].0,
            Component::translatable(
                "commands.gamerule.set",
                vec![
                    TranslatableArg::Text("keep_inventory".into()),
                    TranslatableArg::Text("true".into()),
                ]
            ),
        );

        let features = FeatureFlagSet::minecart_improvements();
        let mut dispatcher = CommandDispatcher::new();
        register_game_rule_command(&mut dispatcher, features.clone()).unwrap();
        source.rules = GameRules::new(features);
        assert_eq!(
            dispatcher.execute("gamerule max_minecart_speed", &mut source),
            Ok(8)
        );
    }
}
