//! Перенос `ServerboundChatCommandSignedPacket`.

use crate::commands::arguments::argument_signatures::ArgumentSignatures;
use crate::network::chat::last_seen_messages::LastSeenMessagesUpdate;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::game::serverbound_chat_command_packet::MAX_CHAT_PACKET_INPUT_SIZE;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundChatCommandSignedPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundChatCommandSignedPacket {
    /// `command`.
    pub command: String,
    /// `timeStamp` — epoch milli.
    pub time_stamp: i64,
    /// `salt`.
    pub salt: i64,
    /// `argumentSignatures`.
    pub argument_signatures: ArgumentSignatures,
    /// `lastSeenMessages`.
    pub last_seen_messages: LastSeenMessagesUpdate,
}

impl ServerboundChatCommandSignedPacket {
    /// Конструктор.
    pub fn new(
        command: String,
        time_stamp: i64,
        salt: i64,
        argument_signatures: ArgumentSignatures,
        last_seen_messages: LastSeenMessagesUpdate,
    ) -> Self {
        Self { command, time_stamp, salt, argument_signatures, last_seen_messages }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChatCommandSignedPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChatCommandSignedPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let command = buf
                    .read_utf_n(MAX_CHAT_PACKET_INPUT_SIZE)
                    .expect("invalid command");
                let time_stamp = buf.read_instant();
                let salt = buf.read_long();
                let argument_signatures = ArgumentSignatures::read(buf);
                let last_seen_messages = LastSeenMessagesUpdate::read(buf);

                ServerboundChatCommandSignedPacket::new(
                    command,
                    time_stamp,
                    salt,
                    argument_signatures,
                    last_seen_messages,
                )
            },
        )
    }
}

impl Packet for ServerboundChatCommandSignedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_chat_command_signed::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf_n(&self.command, MAX_CHAT_PACKET_INPUT_SIZE).expect("command too long");
        buf.write_instant(self.time_stamp);
        buf.write_long(self.salt);
        self.argument_signatures.write(buf);
        self.last_seen_messages.write(buf);
    }
}
