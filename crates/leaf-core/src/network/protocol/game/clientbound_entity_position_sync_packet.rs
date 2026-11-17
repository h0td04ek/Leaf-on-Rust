//! Перенос `ClientboundEntityPositionSyncPacket`.

use crate::entity::position_move_rotation::PositionMoveRotation;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundEntityPositionSyncPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundEntityPositionSyncPacket {
    /// `id`.
    pub id: i32,
    /// `values`.
    pub values: PositionMoveRotation,
    /// `onGround`.
    pub on_ground: bool,
}

impl ClientboundEntityPositionSyncPacket {
    /// `ClientboundEntityPositionSyncPacket::new`.
    pub fn new(id: i32, values: PositionMoveRotation, on_ground: bool) -> Self {
        Self { id, values, on_ground }
    }

    /// `STREAM_CODEC` — VAR_INT + PositionMoveRotation + BOOL.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundEntityPositionSyncPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundEntityPositionSyncPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let values = PositionMoveRotation::read(buf);
                let on_ground = buf.read_bool();
                ClientboundEntityPositionSyncPacket::new(id, values, on_ground)
            },
        )
    }
}

impl Packet for ClientboundEntityPositionSyncPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_entity_position_sync::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        self.values.write(buf);
        buf.write_bool(self.on_ground);
    }
}
