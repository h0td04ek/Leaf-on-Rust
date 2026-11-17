//! Перенос `ClientboundPlayerPositionPacket`.

use crate::entity::position_move_rotation::PositionMoveRotation;
use crate::entity::relative::{Relative, RelativeSet};
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundPlayerPositionPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPlayerPositionPacket {
    /// `id`.
    pub id: i32,
    /// `change`.
    pub change: PositionMoveRotation,
    /// `relatives`.
    pub relatives: RelativeSet,
}

impl ClientboundPlayerPositionPacket {
    /// `ClientboundPlayerPositionPacket::of`.
    pub fn of(id: i32, change: PositionMoveRotation, relatives: RelativeSet) -> Self {
        Self { id, change, relatives }
    }

    /// `STREAM_CODEC` — VAR_INT + PositionMoveRotation + SET.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerPositionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerPositionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let change = PositionMoveRotation::read(buf);
                let relatives = Relative::read_set(buf);
                ClientboundPlayerPositionPacket::of(id, change, relatives)
            },
        )
    }
}

impl Packet for ClientboundPlayerPositionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_position::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        self.change.write(buf);
        Relative::write_set(buf, &self.relatives);
    }
}
