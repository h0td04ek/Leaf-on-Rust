//! Дерево и диспетчер команд Brigadier.

use std::collections::HashMap;

use super::brigadier_argument::{ArgumentType, ArgumentValue};
use crate::network::chat::component::Component;
use crate::resources::Identifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

pub struct CommandDispatcher<S> {
    nodes: Vec<CommandNode<S>>,
    suggestions: HashMap<Identifier, SuggestionProvider<S>>,
}

pub(crate) enum NodeKind {
    Root,
    Literal,
    Argument(ArgumentType),
}

type Requirement<S> = Box<dyn Fn(&S) -> bool + Send + Sync>;
type Command<S> = Box<dyn Fn(&mut S, &CommandContext) -> i32 + Send + Sync>;
type SuggestionProvider<S> =
    Box<dyn Fn(&S, &CommandContext, &str, &mut Vec<CommandSuggestion>) + Send + Sync>;

#[derive(Clone, Debug, PartialEq)]
pub struct CommandSuggestion {
    pub text: String,
    pub tooltip: Option<Component>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CommandSuggestions {
    /// Диапазон замены в байтах UTF-8.
    pub range: std::ops::Range<usize>,
    pub entries: Vec<CommandSuggestion>,
}

pub struct CommandNode<S> {
    pub(crate) kind: NodeKind,
    pub(crate) name: String,
    pub(crate) children: Vec<NodeId>,
    pub(crate) redirect: Option<NodeId>,
    pub(crate) suggestion_provider: Option<Identifier>,
    pub(crate) restricted: bool,
    requirement: Option<Requirement<S>>,
    command: Option<Command<S>>,
}

#[derive(Default, Debug, Clone)]
pub struct CommandContext {
    arguments: HashMap<String, ArgumentValue>,
}

impl CommandContext {
    pub fn argument(&self, name: &str) -> Option<&ArgumentValue> {
        self.arguments.get(name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegistrationError {
    UnknownParent,
    UnknownRedirect,
    ParentRedirects,
    DuplicateChild,
    InvalidName,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandErrorKind {
    UnknownCommand,
    InvalidArgument,
    ExpectedSeparator,
    IncompleteCommand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommandError {
    /// Позиция ошибки в байтах UTF-8.
    pub cursor: usize,
    pub kind: CommandErrorKind,
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} at byte {}", self.kind, self.cursor)
    }
}

impl std::error::Error for CommandError {}

impl<S> CommandNode<S> {
    pub fn literal(name: impl Into<String>) -> Self {
        Self::new(NodeKind::Literal, name.into())
    }

    pub fn argument(name: impl Into<String>, argument: ArgumentType) -> Self {
        Self::new(NodeKind::Argument(argument), name.into())
    }

    fn new(kind: NodeKind, name: String) -> Self {
        Self {
            kind,
            name,
            children: Vec::new(),
            redirect: None,
            suggestion_provider: None,
            restricted: false,
            requirement: None,
            command: None,
        }
    }

    pub fn executes(
        mut self,
        command: impl Fn(&mut S, &CommandContext) -> i32 + Send + Sync + 'static,
    ) -> Self {
        self.command = Some(Box::new(command));
        self
    }

    pub fn requires(mut self, requirement: impl Fn(&S) -> bool + Send + Sync + 'static) -> Self {
        self.requirement = Some(Box::new(requirement));
        self
    }

    pub fn redirects_to(mut self, target: NodeId) -> Self {
        self.redirect = Some(target);
        self
    }

    pub fn with_suggestion_provider(mut self, provider: Identifier) -> Self {
        self.suggestion_provider = Some(provider);
        self
    }

    pub fn restricted(mut self) -> Self {
        self.restricted = true;
        self
    }

    pub(crate) fn can_use(&self, source: &S) -> bool {
        self.requirement
            .as_ref()
            .is_none_or(|requirement| requirement(source))
    }

    pub(crate) fn executable(&self) -> bool {
        self.command.is_some()
    }
}

struct ParseBranch {
    node: NodeId,
    cursor: usize,
    context: CommandContext,
}

impl<S> Default for CommandDispatcher<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> CommandDispatcher<S> {
    pub fn new() -> Self {
        Self {
            nodes: vec![CommandNode::new(NodeKind::Root, String::new())],
            suggestions: HashMap::new(),
        }
    }

    pub fn root(&self) -> NodeId {
        NodeId(0)
    }

    pub fn add_node(
        &mut self,
        parent: NodeId,
        node: CommandNode<S>,
    ) -> Result<NodeId, RegistrationError> {
        let parent_node = self
            .nodes
            .get(parent.0)
            .ok_or(RegistrationError::UnknownParent)?;
        if parent_node.redirect.is_some() {
            return Err(RegistrationError::ParentRedirects);
        }
        if node.name.is_empty() || node.name.chars().any(char::is_whitespace) {
            return Err(RegistrationError::InvalidName);
        }
        if node
            .redirect
            .is_some_and(|target| target.0 >= self.nodes.len())
        {
            return Err(RegistrationError::UnknownRedirect);
        }
        if parent_node
            .children
            .iter()
            .any(|&id| self.node(id).name == node.name)
        {
            return Err(RegistrationError::DuplicateChild);
        }

        let id = NodeId(self.nodes.len());
        self.nodes.push(node);
        self.nodes[parent.0].children.push(id);
        Ok(id)
    }

    pub(crate) fn node(&self, id: NodeId) -> &CommandNode<S> {
        &self.nodes[id.0]
    }

    pub(crate) fn can_use(&self, id: NodeId, source: &S) -> bool {
        let node = self.node(id);
        node.can_use(source)
            && node
                .redirect
                .is_none_or(|target| self.node(target).can_use(source))
    }

    /// Разбирает и выполняет команду. Вход уже очищен от начального '/'.
    pub fn execute(&self, input: &str, source: &mut S) -> Result<i32, CommandError> {
        let parsed = self.parse(input, source)?;
        let command = self.node(parsed.node).command.as_ref().unwrap();
        Ok(command(source, &parsed.context))
    }

    fn children(&self, id: NodeId) -> &[NodeId] {
        &self.node(self.node(id).redirect.unwrap_or(id)).children
    }

    fn relevant_children<'a>(
        &'a self,
        node: NodeId,
        token: &str,
        source: &'a S,
    ) -> impl DoubleEndedIterator<Item = NodeId> + 'a {
        let children = self.children(node);
        let exact = children.iter().copied().find(|&id| {
            let child = self.node(id);
            matches!(child.kind, NodeKind::Literal) && child.name == token
        });
        children.iter().copied().filter(move |&id| {
            (exact == Some(id)
                || (exact.is_none() && matches!(self.node(id).kind, NodeKind::Argument(_))))
                && self.can_use(id, source)
        })
    }

    pub fn register_suggestion_provider(
        &mut self,
        id: Identifier,
        provider: impl Fn(&S, &CommandContext, &str, &mut Vec<CommandSuggestion>)
            + Send
            + Sync
            + 'static,
    ) {
        self.suggestions.insert(id, Box::new(provider));
    }

    /// Подсказки для ввода до курсора. Начальный '/' допускается.
    pub fn complete(&self, input: &str, cursor: usize, source: &S) -> CommandSuggestions {
        let Some(prefix) = input.get(..cursor) else {
            return CommandSuggestions {
                range: 0..0,
                entries: Vec::new(),
            };
        };
        let start = usize::from(prefix.starts_with('/'));
        let mut branches = vec![ParseBranch {
            node: self.root(),
            cursor: start,
            context: CommandContext::default(),
        }];
        let mut frontier = Vec::new();
        let mut furthest = start;
        while let Some(branch) = branches.pop() {
            if branch.cursor > furthest {
                frontier.clear();
                furthest = branch.cursor;
            }
            let remaining = &prefix[branch.cursor..];
            let literal = remaining.split(' ').next().unwrap();
            for id in self.relevant_children(branch.node, literal, source).rev() {
                let child = self.node(id);
                let (value, end) = match child.kind {
                    NodeKind::Literal => (None, branch.cursor + literal.len()),
                    NodeKind::Argument(argument) => {
                        let Some((value, end)) = argument.parse(prefix, branch.cursor) else {
                            continue;
                        };
                        (Some(value), end)
                    }
                    NodeKind::Root => unreachable!(),
                };
                if prefix.as_bytes().get(end) == Some(&b' ') {
                    let mut context = branch.context.clone();
                    if let Some(value) = value {
                        context.arguments.insert(child.name.clone(), value);
                    }
                    branches.push(ParseBranch {
                        node: id,
                        cursor: end + 1,
                        context,
                    });
                }
            }
            if branch.cursor == furthest {
                frontier.push(branch);
            }
        }

        let remaining = &prefix[furthest..];
        let lower = remaining.to_lowercase();
        let mut candidates = Vec::new();
        for branch in frontier {
            for id in self
                .children(branch.node)
                .iter()
                .copied()
                .filter(|&id| self.can_use(id, source))
            {
                let child = self.node(id);
                let mut entries = Vec::new();
                let offset = match child.kind {
                    NodeKind::Literal => {
                        add_matching(&mut entries, &child.name, &lower);
                        0
                    }
                    NodeKind::Argument(argument) => {
                        if let Some(provider) = child
                            .suggestion_provider
                            .as_ref()
                            .and_then(|id| self.suggestions.get(id))
                        {
                            provider(source, &branch.context, remaining, &mut entries);
                            0
                        } else {
                            builtin_suggestions(argument, remaining, &lower, &mut entries)
                        }
                    }
                    NodeKind::Root => unreachable!(),
                };
                candidates.extend(
                    entries
                        .into_iter()
                        .filter(|entry| entry.text != remaining[offset..])
                        .map(|entry| (furthest + offset, entry)),
                );
            }
        }

        let start = candidates
            .iter()
            .map(|(start, _)| *start)
            .min()
            .unwrap_or(furthest);
        let mut entries: Vec<_> = candidates
            .into_iter()
            .map(|(offset, mut entry)| {
                if offset != start {
                    entry.text.insert_str(0, &prefix[start..offset]);
                }
                entry
            })
            .collect();
        entries.sort_by_cached_key(|entry| (entry.text.to_lowercase(), entry.text.clone()));
        entries.dedup_by(|a, b| a.text == b.text);
        CommandSuggestions {
            range: start..cursor,
            entries,
        }
    }

    fn parse(&self, input: &str, source: &S) -> Result<ParseBranch, CommandError> {
        let mut branches = vec![ParseBranch {
            node: self.root(),
            cursor: 0,
            context: CommandContext::default(),
        }];
        let mut best = CommandError {
            cursor: 0,
            kind: CommandErrorKind::UnknownCommand,
        };
        while let Some(branch) = branches.pop() {
            let node = self.node(branch.node);
            if branch.cursor == input.len() {
                if node.executable() {
                    return Ok(branch);
                }
                if branch.node != self.root() {
                    record_error(
                        &mut best,
                        branch.cursor,
                        CommandErrorKind::IncompleteCommand,
                    );
                }
                continue;
            }

            let remaining = &input[branch.cursor..];
            let literal = remaining.split(' ').next().unwrap();
            record_error(&mut best, branch.cursor, CommandErrorKind::UnknownCommand);
            for id in self.relevant_children(branch.node, literal, source).rev() {
                let child = self.node(id);
                let mut context = branch.context.clone();
                let end = match child.kind {
                    NodeKind::Literal => branch.cursor + literal.len(),
                    NodeKind::Argument(argument) => {
                        let Some((value, end)) = argument.parse(input, branch.cursor) else {
                            record_error(
                                &mut best,
                                branch.cursor,
                                CommandErrorKind::InvalidArgument,
                            );
                            continue;
                        };
                        context.arguments.insert(child.name.clone(), value);
                        end
                    }
                    NodeKind::Root => unreachable!(),
                };
                let cursor = match input.as_bytes().get(end) {
                    None => end,
                    Some(b' ') if end + 1 < input.len() => end + 1,
                    _ => {
                        record_error(&mut best, end, CommandErrorKind::ExpectedSeparator);
                        continue;
                    }
                };
                branches.push(ParseBranch {
                    node: id,
                    cursor,
                    context,
                });
            }
        }
        Err(best)
    }
}

fn builtin_suggestions(
    argument: ArgumentType,
    prefix: &str,
    lower: &str,
    entries: &mut Vec<CommandSuggestion>,
) -> usize {
    match argument {
        ArgumentType::Bool => {
            add_matching(entries, "true", lower);
            add_matching(entries, "false", lower);
        }
        ArgumentType::GameMode => {
            for mode in crate::world::level::game_type::GameType::values() {
                add_matching(entries, mode.get_name(), lower);
            }
        }
        ArgumentType::Time { .. } => {
            let number_end = crate::command::arguments::vanilla::number_end(prefix);
            if prefix[..number_end].parse::<f32>().is_ok() {
                for unit in ["d", "s", "t"] {
                    add_matching(entries, unit, &lower[number_end..]);
                }
                return number_end;
            }
        }
        _ => {}
    }
    0
}

fn add_matching(entries: &mut Vec<CommandSuggestion>, text: &str, lower_prefix: &str) {
    if text.to_lowercase().starts_with(lower_prefix) {
        entries.push(CommandSuggestion {
            text: text.to_owned(),
            tooltip: None,
        });
    }
}

fn record_error(best: &mut CommandError, cursor: usize, kind: CommandErrorKind) {
    if cursor > best.cursor
        || cursor == best.cursor && best.kind == CommandErrorKind::UnknownCommand
    {
        *best = CommandError { cursor, kind };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::brigadier_argument::StringMode;

    #[test]
    fn executes_typed_arguments_and_redirects() {
        let mut dispatcher = CommandDispatcher::<Vec<String>>::new();
        let root = dispatcher.root();
        let say = dispatcher
            .add_node(root, CommandNode::literal("say"))
            .unwrap();
        let message = CommandNode::<Vec<String>>::argument(
            "message",
            ArgumentType::String(StringMode::Quotable),
        )
        .executes(|source, context| {
            let Some(ArgumentValue::String(message)) = context.argument("message") else {
                return 0;
            };
            source.push(message.clone());
            1
        });
        dispatcher.add_node(say, message).unwrap();
        dispatcher
            .add_node(root, CommandNode::literal("s").redirects_to(say))
            .unwrap();

        let mut output = Vec::new();
        assert_eq!(
            dispatcher.execute(r#"s "hello \"Leaf\" 🍃""#, &mut output),
            Ok(1)
        );
        assert_eq!(output, ["hello \"Leaf\" 🍃"]);
        assert_eq!(
            dispatcher.execute("say", &mut output).unwrap_err().kind,
            CommandErrorKind::IncompleteCommand
        );
    }

    #[test]
    fn literal_has_priority_over_argument_and_requirements_are_enforced() {
        let mut dispatcher = CommandDispatcher::<bool>::new();
        let root = dispatcher.root();
        let set = dispatcher
            .add_node(root, CommandNode::literal("set"))
            .unwrap();
        dispatcher
            .add_node(
                set,
                CommandNode::<bool>::literal("1")
                    .requires(|source| *source)
                    .executes(|_, _| 10),
            )
            .unwrap();
        dispatcher
            .add_node(
                set,
                CommandNode::argument("count", ArgumentType::Integer { min: 1, max: 3 }).executes(
                    |_, context| match context.argument("count") {
                        Some(ArgumentValue::Integer(count)) => *count,
                        _ => 0,
                    },
                ),
            )
            .unwrap();

        assert_eq!(dispatcher.execute("set 1", &mut true), Ok(10));
        assert!(dispatcher.execute("set 1", &mut false).is_err());
        assert_eq!(dispatcher.execute("set 2", &mut false), Ok(2));
        assert_eq!(
            dispatcher.execute("set 4", &mut true).unwrap_err().kind,
            CommandErrorKind::InvalidArgument
        );
    }

    #[test]
    fn completes_permissions_redirects_and_builtin_bool_before_the_cursor() {
        let mut dispatcher = CommandDispatcher::<bool>::new();
        let root = dispatcher.root();
        let set = dispatcher
            .add_node(root, CommandNode::literal("set"))
            .unwrap();
        dispatcher
            .add_node(set, CommandNode::argument("flag", ArgumentType::Bool))
            .unwrap();
        dispatcher
            .add_node(
                set,
                CommandNode::literal("hidden").requires(|source| *source),
            )
            .unwrap();
        dispatcher
            .add_node(root, CommandNode::literal("s").redirects_to(set))
            .unwrap();
        let cases = [
            ("/SE", 3, false, 1..3, vec!["set"]),
            ("/s Tr trailing", 5, false, 3..5, vec!["true"]),
            ("/s ", 3, false, 3..3, vec!["false", "true"]),
            ("/s ", 3, true, 3..3, vec!["false", "hidden", "true"]),
        ];
        for (input, cursor, source, range, expected) in cases {
            let actual = dispatcher.complete(input, cursor, &source);
            assert_eq!(actual.range, range, "{input}");
            assert_eq!(
                actual
                    .entries
                    .iter()
                    .map(|entry| entry.text.as_str())
                    .collect::<Vec<_>>(),
                expected,
                "{input}"
            );
        }
    }
}
