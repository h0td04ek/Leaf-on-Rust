//! Перенос `ClientboundPlayerChatPacket`.

use crate::network::chat::chat_type::ChatTypeBound;
use crate::network::chat::component::Component;
use crate::network::chat::component_json::{
    component_stream_codec, optional_component_stream_codec,
};
use crate::network::chat::filter_mask::FilterMask;
use crate::network::chat::message_signature::MessageSignature;
use crate::network::chat::signed_message_body::SignedMessageBodyPacked;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::uuid::Uuid;

/// `ClientboundPlayerChatPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPlayerChatPacket {
    /// `globalIndex`.
    pub global_index: i32,
    /// `sender`.
    pub sender: Uuid,
    /// `index`.
    pub index: i32,
    /// `signature`.
    pub signature: Option<MessageSignature>,
    /// `body` — `SignedMessageBody.Packed`.
    pub body: SignedMessageBodyPacked,
    /// `unsignedContent`.
    pub unsigned_content: Option<Component>,
    /// `filterMask`.
    pub filter_mask: FilterMask,
    /// `chatType`.
    pub chat_type: ChatTypeBound,
}

impl ClientboundPlayerChatPacket {
    /// Конструктор.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        global_index: i32,
        sender: Uuid,
        index: i32,
        signature: Option<MessageSignature>,
        body: SignedMessageBodyPacked,
        unsigned_content: Option<Component>,
        filter_mask: FilterMask,
        chat_type: ChatTypeBound,
    ) -> Self {
        Self {
            global_index,
            sender,
            index,
            signature,
            body,
            unsigned_content,
            filter_mask,
            chat_type,
        }
    }

    /// `isSkippable` → true.
    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerChatPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerChatPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let global_index = buf.read_var_int();
                let sender = buf.read_uuid();
                let index = buf.read_var_int();
                let signature = buf.read_nullable(|b| MessageSignature::read(b));
                let body = SignedMessageBodyPacked::read(buf).expect("invalid signed body");
                let unsigned_content = optional_component_stream_codec().decode(buf);
                let filter_mask = FilterMask::read(buf);
                let chat_type = ChatTypeBound::stream_codec().decode(buf);
                ClientboundPlayerChatPacket::new(
                    global_index,
                    sender,
                    index,
                    signature,
                    body,
                    unsigned_content,
                    filter_mask,
                    chat_type,
                )
            },
        )
    }
}

impl Packet for ClientboundPlayerChatPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_chat::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.global_index);
        buf.write_uuid(&self.sender);
        buf.write_var_int(self.index);
        buf.write_nullable(&self.signature, |b, s| MessageSignature::write(b, s));
        self.body.write(buf);
        optional_component_stream_codec().encode(buf, &self.unsigned_content);
        self.filter_mask.write(buf);
        ChatTypeBound::stream_codec().encode(buf, &self.chat_type);
    }

    fn is_skippable(&self) -> bool {
        true
    }
}
