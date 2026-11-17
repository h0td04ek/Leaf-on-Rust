//! Перенос `ServerboundCommandSuggestionPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `MAX_COMMAND_LENGTH` (Paper: readUtf(2048), writeUtf 32500).
const MAX_COMMAND_LENGTH_READ: usize = 2048;
const MAX_COMMAND_LENGTH_WRITE: usize = 32500;

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

    /// `STREAM_CODEC`.
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
