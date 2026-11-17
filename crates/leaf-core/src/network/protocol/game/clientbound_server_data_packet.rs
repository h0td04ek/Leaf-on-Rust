//! Перенос `ClientboundServerDataPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundServerDataPacket`.
///
/// `motd` — `ComponentSerialization.TRUSTED_CONTEXT_FREE_STREAM_CODEC`;
/// `iconBytes` — `ByteBufCodecs.BYTE_ARRAY.optional` (bool + byte[1024] array).
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundServerDataPacket {
    /// `motd`.
    pub motd: Component,
    /// `iconBytes`.
    pub icon_bytes: Option<Vec<u8>>,
}

impl ClientboundServerDataPacket {
    /// `ClientboundServerDataPacket::new`.
    pub fn new(motd: Component, icon_bytes: Option<Vec<u8>>) -> Self {
        Self { motd, icon_bytes }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundServerDataPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundServerDataPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let motd = component_json::component_stream_codec().decode(buf);
                let icon_bytes = buf
                    .read_optional(|b| b.read_byte_array())
                    .and_then(|r| r.ok());

                ClientboundServerDataPacket::new(motd, icon_bytes)
            },
        )
    }
}

impl Packet for ClientboundServerDataPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_server_data::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_json::component_stream_codec().encode(buf, &self.motd);
        buf.write_optional(&self.icon_bytes, |b, bytes| {
            let _ = b.write_byte_array(bytes);
        });
    }
}
