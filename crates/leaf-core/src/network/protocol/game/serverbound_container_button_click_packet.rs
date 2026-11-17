//! Перенос `ServerboundContainerButtonClickPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundContainerButtonClickPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundContainerButtonClickPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `buttonId`.
    pub button_id: i32,
}

impl ServerboundContainerButtonClickPacket {
    pub fn new(container_id: i32, button_id: i32) -> Self {
        Self { container_id, button_id }
    }

    /// `containerId`.
    pub fn container_id(&self) -> i32 {
        self.container_id
    }

    /// `buttonId`.
    pub fn button_id(&self) -> i32 {
        self.button_id
    }

    /// `STREAM_CODEC` (`CONTAINER_ID` + `VAR_INT`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundContainerButtonClickPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundContainerButtonClickPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let button_id = buf.read_var_int();

                ServerboundContainerButtonClickPacket::new(container_id, button_id)
            },
        )
    }
}

impl Packet for ServerboundContainerButtonClickPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_container_button_click::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_var_int(self.button_id);
    }
}
