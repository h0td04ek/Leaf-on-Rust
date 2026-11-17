//! Перенос `ClientboundCustomChatCompletionsPacket` (+ `Action`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundCustomChatCompletionsPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CustomChatCompletionsAction {
    /// `ADD`.
    Add = 0,
    /// `REMOVE`.
    Remove = 1,
    /// `SET`.
    Set = 2,
}

impl CustomChatCompletionsAction {
    /// `values()`-порядок.
    pub const VALUES: [CustomChatCompletionsAction; 3] = [
        CustomChatCompletionsAction::Add,
        CustomChatCompletionsAction::Remove,
        CustomChatCompletionsAction::Set,
    ];

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(3) as usize]
    }

    /// `ordinal` (`writeEnum`).
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// `ClientboundCustomChatCompletionsPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundCustomChatCompletionsPacket {
    /// `action`.
    pub action: CustomChatCompletionsAction,
    /// `entries`.
    pub entries: Vec<String>,
}

impl ClientboundCustomChatCompletionsPacket {
    pub fn new(action: CustomChatCompletionsAction, entries: Vec<String>) -> Self {
        Self { action, entries }
    }

    /// `action`.
    pub fn action(&self) -> CustomChatCompletionsAction {
        self.action
    }

    /// `entries`.
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// `STREAM_CODEC` — enum + list(utf).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCustomChatCompletionsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCustomChatCompletionsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let action = CustomChatCompletionsAction::by_id(buf.read_var_int());
                let entries = buf.read_collection(|_| Vec::new(), |b| b.read_utf().expect("invalid entry"));

                ClientboundCustomChatCompletionsPacket::new(action, entries)
            },
        )
    }
}

impl Packet for ClientboundCustomChatCompletionsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_custom_chat_completions::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.action.ordinal());
        buf.write_collection(&self.entries, |b, e| {
            b.write_utf(e).expect("entry too long");
        });
    }
}
