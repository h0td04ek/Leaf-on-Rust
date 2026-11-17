//! Перенос `ServerboundSelectTradePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundSelectTradePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundSelectTradePacket {
    /// `item` — индекс оффера в merchant menu.
    pub item: i32,
}

impl ServerboundSelectTradePacket {
    pub fn new(item: i32) -> Self {
        Self { item }
    }

    /// `getItem`.
    pub fn get_item(&self) -> i32 {
        self.item
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSelectTradePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSelectTradePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ServerboundSelectTradePacket::new(buf.read_var_int()),
        )
    }
}

impl Packet for ServerboundSelectTradePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_select_trade::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.item);
    }
}
