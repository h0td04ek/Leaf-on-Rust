//! Перенос `ClientboundTeleportEntityPacket`.

use crate::entity::position_move_rotation::PositionMoveRotation;
use crate::entity::relative::{Relative, RelativeSet};
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundTeleportEntityPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundTeleportEntityPacket {
    /// `id`.
    pub id: i32,
    /// `change`.
    pub change: PositionMoveRotation,
    /// `relatives`.
    pub relatives: RelativeSet,
    /// `onGround`.
    pub on_ground: bool,
}

impl ClientboundTeleportEntityPacket {
    /// `ClientboundTeleportEntityPacket::teleport`.
    pub fn teleport(id: i32, change: PositionMoveRotation, relatives: RelativeSet, on_ground: bool) -> Self {
        Self { id, change, relatives, on_ground }
    }

    /// `STREAM_CODEC` — VAR_INT + PositionMoveRotation + SET + BOOL.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTeleportEntityPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTeleportEntityPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let change = PositionMoveRotation::read(buf);
                let relatives = Relative::read_set(buf);
                let on_ground = buf.read_bool();
                ClientboundTeleportEntityPacket::teleport(id, change, relatives, on_ground)
            },
        )
    }
}

impl Packet for ClientboundTeleportEntityPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_teleport_entity::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        self.change.write(buf);
        Relative::write_set(buf, &self.relatives);
        buf.write_bool(self.on_ground);
    }
}
