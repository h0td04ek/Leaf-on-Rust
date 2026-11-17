//! Перенос `ServerboundChatPacket`.
//!
//! Leaf-флаг `ChatMessageSignature.enabled` управляет записью подписи;
//! здесь подпись всегда пишется/читается если присутствует (nullable).

use crate::network::chat::last_seen_messages::LastSeenMessagesUpdate;
use crate::network::chat::message_signature::MessageSignature;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `MAX_CHAT_PACKET_INPUT_SIZE`-эквивалент для chat (256).
const MAX_MESSAGE_LENGTH: usize = 256;

/// `ServerboundChatPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ServerboundChatPacket {
    /// `message`.
    pub message: String,
    /// `timeStamp` — epoch milli.
    pub time_stamp: i64,
    /// `salt`.
    pub salt: i64,
    /// `signature` — `None` для неподписанных.
    pub signature: Option<MessageSignature>,
    /// `lastSeenMessages`.
    pub last_seen_messages: LastSeenMessagesUpdate,
}

impl ServerboundChatPacket {
    /// Конструктор.
    pub fn new(
        message: String,
        time_stamp: i64,
        salt: i64,
        signature: Option<MessageSignature>,
        last_seen_messages: LastSeenMessagesUpdate,
    ) -> Self {
        Self { message, time_stamp, salt, signature, last_seen_messages }
    }

    /// `STREAM_CODEC` — utf(256) + instant + long + nullable signature + Update.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundChatPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundChatPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let message = buf.read_utf_n(MAX_MESSAGE_LENGTH).expect("invalid chat message");
                let time_stamp = buf.read_instant();
                let salt = buf.read_long();
                let signature = buf.read_nullable(|b| MessageSignature::read(b));
                let last_seen_messages = LastSeenMessagesUpdate::read(buf);

                ServerboundChatPacket::new(message, time_stamp, salt, signature, last_seen_messages)
            },
        )
    }
}

impl Packet for ServerboundChatPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_chat::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf_n(&self.message, MAX_MESSAGE_LENGTH).expect("message too long");
        buf.write_instant(self.time_stamp);
        buf.write_long(self.salt);
        buf.write_nullable(&self.signature, |b, s| MessageSignature::write(b, s));
        self.last_seen_messages.write(buf);
    }
}
