//! Перенос `ClientboundMoveMinecartPacket`.

use crate::entity::minecart_step::MinecartStep;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundMoveMinecartPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundMoveMinecartPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `lerpSteps`.
    pub lerp_steps: Vec<MinecartStep>,
}

impl ClientboundMoveMinecartPacket {
    /// `ClientboundMoveMinecartPacket::new`.
    pub fn new(entity_id: i32, lerp_steps: Vec<MinecartStep>) -> Self {
        Self { entity_id, lerp_steps }
    }

    /// `STREAM_CODEC` — VAR_INT + list(MinecartStep).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundMoveMinecartPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundMoveMinecartPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let lerp_steps = buf.read_list(|b| MinecartStep::read(b));

                ClientboundMoveMinecartPacket::new(entity_id, lerp_steps)
            },
        )
    }
}

impl Packet for ClientboundMoveMinecartPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_move_minecart_along_track::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        buf.write_collection(&self.lerp_steps, |b, s| {
            s.write(b);
        });
    }
}
