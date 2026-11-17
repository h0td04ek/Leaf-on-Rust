//! Перенос `ServerboundContainerSlotStateChangedPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ServerboundContainerSlotStateChangedPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundContainerSlotStateChangedPacket {
    /// `slotId`.
    pub slot_id: i32,
    /// `containerId`.
    pub container_id: i32,
    /// `newState`.
    pub new_state: bool,
}

impl ServerboundContainerSlotStateChangedPacket {
    /// `ServerboundContainerSlotStateChangedPacket::new`.
    pub fn new(slot_id: i32, container_id: i32, new_state: bool) -> Self {
        Self { slot_id, container_id, new_state }
    }
    /// `slotId`.
    pub fn slot_id(&self) -> i32 { self.slot_id }
    /// `containerId`.
    pub fn container_id(&self) -> i32 { self.container_id }
    /// `newState`.
    pub fn new_state(&self) -> bool { self.new_state }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundContainerSlotStateChangedPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundContainerSlotStateChangedPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let slot_id = buf.read_var_int();
                let container_id = buf.read_container_id();
                let new_state = buf.read_bool();
                ServerboundContainerSlotStateChangedPacket::new(slot_id, container_id, new_state)
            },
        )
    }
}

impl Packet for ServerboundContainerSlotStateChangedPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_container_slot_state_changed::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.slot_id);
        buf.write_container_id(self.container_id);
        buf.write_bool(self.new_state);    }
}
