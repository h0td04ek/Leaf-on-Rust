//! Перенос `ClientboundCommandSuggestionsPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json::optional_component_stream_codec;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundCommandSuggestionsPacket.Entry`.
#[derive(Clone, Debug, PartialEq)]
pub struct CommandSuggestionEntry {
    /// `text`.
    pub text: String,
    /// `tooltip`.
    pub tooltip: Option<Component>,
}

impl CommandSuggestionEntry {
    /// `Entry::new`.
    pub fn new(text: String, tooltip: Option<Component>) -> Self {
        Self { text, tooltip }
    }

    /// `STREAM_CODEC` — STRING_UTF8 + optional component.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, CommandSuggestionEntry> {
        FnCodec::new(
            |buf: &mut FriendlyByteBuf, e: &CommandSuggestionEntry| {
                buf.write_utf(&e.text).expect("invalid utf");
                optional_component_stream_codec().encode(buf, &e.tooltip);
            },
            |buf: &mut FriendlyByteBuf| {
                let text = buf.read_utf().expect("invalid utf");
                let tooltip = optional_component_stream_codec().decode(buf);
                CommandSuggestionEntry::new(text, tooltip)
            },
        )
    }
}

/// `ClientboundCommandSuggestionsPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundCommandSuggestionsPacket {
    /// `id`.
    pub id: i32,
    /// `start`.
    pub start: i32,
    /// `length`.
    pub length: i32,
    /// `suggestions`.
    pub suggestions: Vec<CommandSuggestionEntry>,
}

impl ClientboundCommandSuggestionsPacket {
    /// `ClientboundCommandSuggestionsPacket::new`.
    pub fn new(id: i32, start: i32, length: i32, suggestions: Vec<CommandSuggestionEntry>) -> Self {
        Self { id, start, length, suggestions }
    }

    /// `id`/`start`/`length`/`suggestions`.
    pub fn id(&self) -> i32 {
        self.id
    }
    /// `start`.
    pub fn start(&self) -> i32 {
        self.start
    }
    /// `length`.
    pub fn length(&self) -> i32 {
        self.length
    }
    /// `suggestions`.
    pub fn suggestions(&self) -> &[CommandSuggestionEntry] {
        &self.suggestions
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCommandSuggestionsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCommandSuggestionsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let start = buf.read_var_int();
                let length = buf.read_var_int();
                let suggestions = buf.read_list(|b| CommandSuggestionEntry::stream_codec().decode(b));
                ClientboundCommandSuggestionsPacket::new(id, start, length, suggestions)
            },
        )
    }
}

impl Packet for ClientboundCommandSuggestionsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_command_suggestions::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_var_int(self.start);
        buf.write_var_int(self.length);
        buf.write_collection(&self.suggestions, |b, e| {
            CommandSuggestionEntry::stream_codec().encode(b, e);
        });
    }
}
