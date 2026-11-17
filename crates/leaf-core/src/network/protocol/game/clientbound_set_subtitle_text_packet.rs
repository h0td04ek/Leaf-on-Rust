//! Перенос `ClientboundSetSubtitleTextPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetSubtitleTextPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetSubtitleTextPacket {
    /// `text`.
    pub text: Component,
}

impl ClientboundSetSubtitleTextPacket {
    pub fn new(text: Component) -> Self {
        Self { text }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetSubtitleTextPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetSubtitleTextPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundSetSubtitleTextPacket::new(
                    component_json::component_stream_codec().decode(buf),
                )
            },
        )
    }
}

impl Packet for ClientboundSetSubtitleTextPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_subtitle_text::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_json::component_stream_codec().encode(buf, &self.text);
    }
}
