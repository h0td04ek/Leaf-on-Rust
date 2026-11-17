//! Перенос `ClientboundDeleteChatPacket`.

use crate::network::chat::message_signature::PackedSignature;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundDeleteChatPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundDeleteChatPacket {
    /// `messageSignature` — `MessageSignature.Packed`.
    pub message_signature: PackedSignature,
}

impl ClientboundDeleteChatPacket {
    pub fn new(message_signature: PackedSignature) -> Self {
        Self { message_signature }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDeleteChatPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundDeleteChatPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundDeleteChatPacket::new(PackedSignature::read(buf))
            },
        )
    }
}

impl Packet for ClientboundDeleteChatPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_delete_chat::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        PackedSignature::write(buf, &self.message_signature);
    }
}
