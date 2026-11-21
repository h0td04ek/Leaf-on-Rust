//! Перенос `ServerboundCommandSuggestionPacket`.

use crate::command::brigadier::CommandDispatcher;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::clientbound_command_suggestions_packet::{
    ClientboundCommandSuggestionsPacket, CommandSuggestionEntry,
};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `MAX_COMMAND_LENGTH` (Paper: readUtf(2048), writeUtf 32500).
const MAX_COMMAND_LENGTH_READ: usize = 2048;
const MAX_COMMAND_LENGTH_WRITE: usize = 32500;
const MAX_COMMAND_SUGGESTIONS: usize = 1000;

/// `ServerboundCommandSuggestionPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundCommandSuggestionPacket {
    /// `id`.
    pub id: i32,
    /// `command`.
    pub command: String,
}

impl ServerboundCommandSuggestionPacket {
    pub fn new(id: i32, command: String) -> Self {
        Self { id, command }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `getCommand`.
    pub fn get_command(&self) -> &str {
        &self.command
    }

    pub fn reply<S>(
        &self,
        dispatcher: &CommandDispatcher<S>,
        source: &S,
    ) -> ClientboundCommandSuggestionsPacket {
        let answers = dispatcher.complete(&self.command, self.command.len(), source);
        if answers.entries.is_empty() {
            return ClientboundCommandSuggestionsPacket::new(self.id, 0, 0, Vec::new());
        }

        // Brigadier работает с индексами Java String, измеряемыми в UTF-16.
        let start = self.command[..answers.range.start].encode_utf16().count() as i32;
        let length = self.command[answers.range].encode_utf16().count() as i32;
        let entries = answers
            .entries
            .into_iter()
            .take(MAX_COMMAND_SUGGESTIONS)
            .map(|entry| CommandSuggestionEntry::new(entry.text, entry.tooltip))
            .collect();
        ClientboundCommandSuggestionsPacket::new(self.id, start, length, entries)
    }

    /// STREAM_CODEC.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundCommandSuggestionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundCommandSuggestionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let command = buf
                    .read_utf_n(MAX_COMMAND_LENGTH_READ)
                    .expect("invalid command");

                ServerboundCommandSuggestionPacket::new(id, command)
            },
        )
    }
}

impl Packet for ServerboundCommandSuggestionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_command_suggestion::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_utf_n(&self.command, MAX_COMMAND_LENGTH_WRITE)
            .expect("command too long");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::brigadier::{CommandNode, CommandSuggestion};
    use crate::command::brigadier_argument::{ArgumentType, ArgumentValue, StringMode};
    use crate::network::chat::component::Component;
    use crate::resources::Identifier;

    #[test]
    fn replies_to_client_with_utf16_range_and_tooltip() {
        let mut dispatcher = CommandDispatcher::<()>::new();
        let root = dispatcher.root();
        let command = dispatcher
            .add_node(root, CommandNode::literal("tell"))
            .unwrap();
        let recipient = dispatcher
            .add_node(
                command,
                CommandNode::argument("recipient", ArgumentType::String(StringMode::Quotable)),
            )
            .unwrap();
        let provider = Identifier::minecraft("test_provider");
        dispatcher
            .add_node(
                recipient,
                CommandNode::argument("text", ArgumentType::String(StringMode::Greedy))
                    .with_suggestion_provider(provider.clone()),
            )
            .unwrap();
        dispatcher.register_suggestion_provider(provider, |_, context, remaining, suggestions| {
            assert_eq!(
                context.argument("recipient"),
                Some(&ArgumentValue::String("🍃".to_owned()))
            );
            assert_eq!(remaining, "Fa");
            suggestions.push(CommandSuggestion {
                text: "Fancy leaf".to_owned(),
                tooltip: Some(Component::literal("example")),
            });
        });

        let packet = ServerboundCommandSuggestionPacket::new(72, r#"/tell "🍃" Fa"#.to_owned())
            .reply(&dispatcher, &());
        assert_eq!((packet.id, packet.start, packet.length), (72, 11, 2));
        let mut buf = FriendlyByteBuf::new();
        ClientboundCommandSuggestionsPacket::stream_codec().encode(&mut buf, &packet);
        let decoded = ClientboundCommandSuggestionsPacket::stream_codec().decode(&mut buf);
        assert_eq!(decoded, packet);
    }
}
