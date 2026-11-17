//! Перенос `ServerboundRenameItemPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundRenameItemPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundRenameItemPacket {
    /// `name` — новое имя предмета (SNBT component в Java;
    /// здесь — строка как в проводе).
    pub name: String,
}

impl ServerboundRenameItemPacket {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    /// `getName`.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// `STREAM_CODEC` — `readUtf`/`writeUtf` (32767).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundRenameItemPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundRenameItemPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundRenameItemPacket::new(buf.read_utf().expect("invalid name"))
            },
        )
    }
}

impl Packet for ServerboundRenameItemPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_rename_item::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.name).expect("name too long");
    }
}
