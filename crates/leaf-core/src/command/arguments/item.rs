//! `ItemArgument` for the registry item and its component patch.

use crate::command::brigadier::CommandSuggestion;
use crate::core::component::data_component_patch::DataComponentPatchBuilder;
use crate::core::component::data_component_type::{
    all_data_component_types, data_component_type_by_identifier,
};
use crate::core::component::DataComponentPatch;
use crate::core::holder::Holder;
use crate::nbt::snbt_parse;
use crate::resources::Identifier;
use crate::world::item::{
    component::item_container_contents::ItemContainerContents, item, item_stack::ItemStack,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ItemInput {
    pub item: Holder<item::Item>,
    pub components: DataComponentPatch,
}

impl ItemInput {
    pub fn create_stack(&self, count: i32) -> ItemStack {
        ItemStack::with_patch(self.item.clone(), count, self.components.clone())
    }

    fn has_valid_components(&self) -> bool {
        let stack = self.create_stack(1);
        let max_stack = Identifier::minecraft("max_stack_size");
        if stack.has_component(&Identifier::minecraft("max_damage"))
            && stack.get_component::<i32>(&max_stack).unwrap_or(1) > 1
        {
            return false;
        }
        stack
            .components
            .get_by_id::<ItemContainerContents>(&Identifier::minecraft("container"))
            .is_none_or(|container| {
                container
                    .non_empty_items()
                    .all(|item| item.get_count() <= item.get_max_stack_size())
            })
    }
}

enum Completion {
    Item,
    Component {
        offset: usize,
        removal: bool,
    },
    Suffix {
        offset: usize,
        choices: &'static [&'static str],
    },
    None,
}

struct Reader<'a> {
    input: &'a str,
    cursor: usize,
    completion: Completion,
}

impl<'a> Reader<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            cursor: 0,
            completion: Completion::Item,
        }
    }

    fn remaining(&self) -> &'a str {
        &self.input[self.cursor..]
    }

    fn skip_whitespace(&mut self) {
        self.cursor = self.input.len() - self.remaining().trim_start().len();
    }

    fn take(&mut self, symbol: char) -> bool {
        if !self.remaining().starts_with(symbol) {
            return false;
        }
        self.cursor += symbol.len_utf8();
        true
    }

    fn identifier(&mut self) -> Option<Identifier> {
        let remaining = self.remaining();
        let len = remaining.len()
            - remaining
                .trim_start_matches(Identifier::is_allowed_in_identifier)
                .len();
        self.cursor += len;
        if len == 0 {
            return None;
        }
        Identifier::try_parse(&remaining[..len])
    }

    fn suffix(&mut self, choices: &'static [&'static str]) {
        self.completion = Completion::Suffix {
            offset: self.cursor,
            choices,
        };
    }

    fn read(&mut self) -> Option<ItemInput> {
        let item = item::item_holder_by_identifier(&self.identifier()?)?;
        self.suffix(&["["]);
        let mut components = DataComponentPatch::builder();
        if self.take('[') {
            self.read_patch(&mut components)?;
        }
        let value = ItemInput {
            item,
            components: components.build(),
        };
        value.has_valid_components().then_some(value)
    }

    fn read_patch(&mut self, components: &mut DataComponentPatchBuilder) -> Option<()> {
        loop {
            self.skip_whitespace();
            if self.take(']') {
                self.completion = Completion::None;
                return Some(());
            }
            let removal = self.take('!');
            self.completion = Completion::Component {
                offset: self.cursor,
                removal,
            };
            let component = data_component_type_by_identifier(&self.identifier()?)?;
            if component.transient || components.is_set(&component) {
                self.completion = Completion::None;
                return None;
            }

            self.skip_whitespace();
            if removal {
                components.remove(&component);
            } else {
                self.suffix(&["="]);
                if !self.take('=') {
                    return None;
                }
                self.completion = Completion::None;
                let codec = component.nbt_codec.as_ref()?;
                let (tag, len) = snbt_parse::parse_prefix(self.remaining()).ok()?;
                self.cursor += len;
                components.set_boxed(component.id, codec.decode(&tag).ok()?);
            }

            self.skip_whitespace();
            self.suffix(&[",", "]"]);
            if !self.take(',') {
                if self.take(']') {
                    self.completion = Completion::None;
                    return Some(());
                }
                return None;
            }
        }
    }
}

pub(crate) fn parse(input: &str) -> Option<(ItemInput, usize)> {
    let mut reader = Reader::new(input);
    let value = reader.read()?;
    Some((value, reader.cursor))
}

fn matches_word_prefix(text: &str, prefix: &str) -> bool {
    text.starts_with(prefix)
        || text
            .match_indices(['.', '_', '/'])
            .any(|(index, _)| text[index + 1..].starts_with(prefix))
}

fn matches_resource(id: &Identifier, prefix: &str) -> bool {
    if prefix.contains(':') {
        matches_word_prefix(&id.to_string(), prefix)
    } else {
        matches_word_prefix(id.get_namespace(), prefix)
            || matches_word_prefix(id.get_path(), prefix)
    }
}

pub(crate) fn suggest(input: &str, entries: &mut Vec<CommandSuggestion>) -> usize {
    let mut reader = Reader::new(input);
    reader.read();
    let mut add = |text| {
        entries.push(CommandSuggestion {
            text,
            tooltip: None,
        })
    };
    match reader.completion {
        Completion::Item => {
            let prefix = input.to_ascii_lowercase();
            for id in item::item_identifiers() {
                if matches_resource(&id, &prefix) {
                    add(id.to_string());
                }
            }
            0
        }
        Completion::Component { offset, removal } => {
            let prefix = input[offset..].to_ascii_lowercase();
            if "!".starts_with(&prefix) && !removal {
                add("!".into());
            }
            for component in all_data_component_types() {
                if !component.transient
                    && (removal || component.nbt_codec.is_some())
                    && matches_resource(&component.id, &prefix)
                {
                    let suffix = if removal { "" } else { "=" };
                    add(format!("{}{suffix}", component.id));
                }
            }
            offset
        }
        Completion::Suffix { offset, choices } if input.len() == offset => {
            for &choice in choices {
                add(choice.to_owned());
            }
            offset
        }
        _ => input.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::brigadier::{CommandDispatcher, CommandNode};
    use crate::command::brigadier_argument::{ArgumentType, ArgumentValue};
    use crate::network::chat::component::Component;
    use crate::network::friendly_byte_buf::FriendlyByteBuf;

    #[test]
    fn parses_bracket_patches_and_a_following_argument() {
        let mut components = crate::core::component::DataComponentMap::new();
        let size =
            data_component_type_by_identifier(&Identifier::minecraft("max_stack_size")).unwrap();
        components.set(&size, 32_i32);
        let wire_id = item::register_item_with_components(
            item::Item::new(Identifier::parse("fixture:command_blade").unwrap()),
            components,
        );
        let mut dispatcher = CommandDispatcher::<()>::new();
        let root = dispatcher
            .add_node(dispatcher.root(), CommandNode::literal("test_item"))
            .unwrap();
        let item = dispatcher
            .add_node(root, CommandNode::argument("item", ArgumentType::ItemStack))
            .unwrap();
        dispatcher
            .add_node(
                item,
                CommandNode::argument("count", ArgumentType::Integer { min: 1, max: 99 }).executes(
                    move |_, context| {
                        let Some(ArgumentValue::Item(input)) = context.argument("item") else {
                            panic!("expected an item");
                        };
                        let Some(ArgumentValue::Integer(count)) = context.argument("count") else {
                            panic!("expected a count");
                        };
                        let stack = input.create_stack(*count);
                        assert_eq!(stack.item_holder(), Some(&Holder::Reference(wire_id)));
                        let label = stack
                            .get_component::<Component>(&Identifier::minecraft("custom_name"))
                            .unwrap();
                        assert_eq!(label.get_string(), "Leaf, 🍃 ]");
                        let patch = input.components.to_nbt().unwrap();
                        assert!(patch
                            .get_compound("!minecraft:unbreakable")
                            .unwrap()
                            .is_empty());
                        assert!(patch
                            .get_compound("minecraft:custom_data")
                            .unwrap()
                            .get_list("items")
                            .is_some());
                        stack.get_count()
                    },
                ),
            )
            .unwrap();
        let input = r#"test_item fixture:command_blade[custom_name="Leaf, 🍃 ]", custom_data={items:[1,"two"]}, !unbreakable,] 3"#;
        assert_eq!(dispatcher.execute(input, &mut ()), Ok(3));
        for invalid in [
            "test_item fixture:missing[] 3",
            "test_item fixture:command_blade[damage=1,!minecraft:damage] 3",
            "test_item fixture:command_blade[damage=-1] 3",
            "test_item fixture:command_blade[max_stack_size=3,max_damage=20] 3",
            "test_item fixture:command_blade[max_damage=20] 3",
            "test_item fixture:command_blade[custom_data={items:[1,2}] 3",
        ] {
            assert!(dispatcher.execute(invalid, &mut ()).is_err(), "{invalid}");
        }
        let mut buffer = FriendlyByteBuf::new();
        assert_eq!(ArgumentType::ItemStack.registry_id(), 14);
        ArgumentType::ItemStack.write_template(&mut buffer);
        assert_eq!(buffer.writer_index(), 0);
        assert_eq!(
            ArgumentType::read_template(14, &mut buffer),
            Some(ArgumentType::ItemStack)
        );
    }

    #[test]
    fn completes_item_names_and_patch_tokens_at_their_cursor_positions() {
        item::register_item(item::Item::new(
            Identifier::parse("fixture:autocomplete_sword").unwrap(),
        ));

        let mut dispatcher = CommandDispatcher::<()>::new();
        let root = dispatcher
            .add_node(dispatcher.root(), CommandNode::literal("test_item"))
            .unwrap();
        dispatcher
            .add_node(root, CommandNode::argument("item", ArgumentType::ItemStack))
            .unwrap();
        for (input, replace, suggestions) in [
            (
                "/test_item autocomplete_sw",
                "autocomplete_sw",
                &["fixture:autocomplete_sword"][..],
            ),
            ("/test_item fixture:autocomplete_sword", "", &["["][..]),
            (
                "/test_item fixture:autocomplete_sword[!unbre",
                "unbre",
                &["minecraft:unbreakable"][..],
            ),
            (
                "/test_item fixture:autocomplete_sword[custom_na",
                "custom_na",
                &["minecraft:custom_name="][..],
            ),
            (
                "/test_item fixture:autocomplete_sword[custom_name",
                "",
                &["="][..],
            ),
            (
                "/test_item fixture:autocomplete_sword[custom_name='🍃'",
                "",
                &[",", "]"][..],
            ),
        ] {
            let result = dispatcher.complete(input, input.len(), &());
            assert_eq!(result.range, input.len() - replace.len()..input.len());
            assert_eq!(
                result
                    .entries
                    .iter()
                    .map(|value| value.text.as_str())
                    .collect::<Vec<_>>(),
                suggestions
            );
        }
    }
}
