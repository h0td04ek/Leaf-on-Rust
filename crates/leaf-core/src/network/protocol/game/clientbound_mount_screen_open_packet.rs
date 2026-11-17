//! Перенос `ClientboundMountScreenOpenPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ClientboundMountScreenOpenPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundMountScreenOpenPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `inventoryColumns`.
    pub inventory_columns: i32,
    /// `entityId`.
    pub entity_id: i32,
}

impl ClientboundMountScreenOpenPacket {
    /// `ClientboundMountScreenOpenPacket::new`.
    pub fn new(container_id: i32, inventory_columns: i32, entity_id: i32) -> Self {
        Self { container_id, inventory_columns, entity_id }
    }
    /// `getContainerId`.
    pub fn get_container_id(&self) -> i32 { self.container_id }
    /// `getInventoryColumns`.
    pub fn get_inventory_columns(&self) -> i32 { self.inventory_columns }
    /// `getEntityId`.
    pub fn get_entity_id(&self) -> i32 { self.entity_id }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundMountScreenOpenPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundMountScreenOpenPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let inventory_columns = buf.read_var_int();
                let entity_id = buf.read_int();
                ClientboundMountScreenOpenPacket::new(container_id, inventory_columns, entity_id)
            },
        )
    }
}

impl Packet for ClientboundMountScreenOpenPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_mount_screen_open::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        buf.write_var_int(self.inventory_columns);
        buf.write_int(self.entity_id);    }
}
