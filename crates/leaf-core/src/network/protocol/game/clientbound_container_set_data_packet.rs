//! Перенос `ClientboundContainerSetDataPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundContainerSetDataPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundContainerSetDataPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `id`.
    pub id: i32,
    /// `value`.
    pub value: i32,
}

impl ClientboundContainerSetDataPacket {
    pub fn new(container_id: i32, id: i32, value: i32) -> Self {
        Self { container_id, id, value }
    }

    /// `getContainerId`.
    pub fn get_container_id(&self) -> i32 {
        self.container_id
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `getValue`.
    pub fn get_value(&self) -> i32 {
        self.value
    }

    /// `STREAM_CODEC` (`readContainerId` + SHORT×2).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundContainerSetDataPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundContainerSetDataPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let id = buf.read_short() as i32;
                let value = buf.read_short() as i32;

                ClientboundContainerSetDataPacket::new(container_id, id, value)
            },
        )
    }
}

impl Packet for ClientboundContainerSetDataPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_container_set_data::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_short(self.id);
        buf.write_short(self.value);
    }
}
