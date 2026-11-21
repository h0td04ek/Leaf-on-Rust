//! Дерево и диспетчер команд Brigadier.

use std::collections::HashMap;

use super::brigadier_argument::{ArgumentType, ArgumentValue};
use crate::resources::Identifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

pub struct CommandDispatcher<S> {
    nodes: Vec<CommandNode<S>>,
}

pub(crate) enum NodeKind {
    Root,
    Literal,
    Argument(ArgumentType),
}

type Requirement<S> = Box<dyn Fn(&S) -> bool + Send + Sync>;
type Command<S> = Box<dyn Fn(&mut S, &CommandContext) -> i32 + Send + Sync>;

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
            let children = &self.node(node.redirect.unwrap_or(branch.node)).children;
            let exact = children.iter().copied().find(|&id| {
                let child = self.node(id);
                matches!(child.kind, NodeKind::Literal) && child.name == literal
            });

            record_error(&mut best, branch.cursor, CommandErrorKind::UnknownCommand);
            for &id in children.iter().rev() {
                let child = self.node(id);
                let mut context = branch.context.clone();
                let end = match child.kind {
                    NodeKind::Literal if exact == Some(id) && self.can_use(id, source) => {
                        branch.cursor + literal.len()
                    }
                    NodeKind::Argument(argument) if exact.is_none() && self.can_use(id, source) => {
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
                    _ => continue,
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
}
