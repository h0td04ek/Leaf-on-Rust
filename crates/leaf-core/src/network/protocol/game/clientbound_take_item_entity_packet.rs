//! Перенос `ClientboundTakeItemEntityPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundTakeItemEntityPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundTakeItemEntityPacket {
    /// `itemId`.
    pub item_id: i32,
    /// `playerId`.
    pub player_id: i32,
    /// `amount`.
    pub amount: i32,
}

impl ClientboundTakeItemEntityPacket {
    /// `ClientboundTakeItemEntityPacket::new`.
    pub fn new(item_id: i32, player_id: i32, amount: i32) -> Self {
        Self { item_id, player_id, amount }
    }

    /// `getItemId`.
    pub fn get_item_id(&self) -> i32 {
        self.item_id
    }

    /// `getPlayerId`.
    pub fn get_player_id(&self) -> i32 {
        self.player_id
    }

    /// `getAmount`.
    pub fn get_amount(&self) -> i32 {
        self.amount
    }

    /// `STREAM_CODEC` — VAR_INT×3.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTakeItemEntityPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTakeItemEntityPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let item_id = buf.read_var_int();
                let player_id = buf.read_var_int();
                let amount = buf.read_var_int();

                ClientboundTakeItemEntityPacket::new(item_id, player_id, amount)
            },
        )
    }
}

impl Packet for ClientboundTakeItemEntityPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_take_item_entity::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.item_id);
        buf.write_var_int(self.player_id);
        buf.write_var_int(self.amount);
    }
}
