//! Перенос `ClientboundContainerClosePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundContainerClosePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundContainerClosePacket {
    /// `containerId`.
    pub container_id: i32,
}

impl ClientboundContainerClosePacket {
    pub fn new(container_id: i32) -> Self {
        Self { container_id }
    }

    /// `getContainerId`.
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    /// `STREAM_CODEC` (`readContainerId`/`writeContainerId` = VarInt).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundContainerClosePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundContainerClosePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundContainerClosePacket::new(buf.read_container_id())
            },
        )
    }
}

impl Packet for ClientboundContainerClosePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_container_close::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
    }
}
