//! Перенос `ClientboundSetSimulationDistancePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetSimulationDistancePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundSetSimulationDistancePacket {
    /// `simulationDistance`.
    pub simulation_distance: i32,
}

impl ClientboundSetSimulationDistancePacket {
    pub fn new(simulation_distance: i32) -> Self {
        Self { simulation_distance }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetSimulationDistancePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetSimulationDistancePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundSetSimulationDistancePacket::new(buf.read_var_int())
            },
        )
    }
}

impl Packet for ClientboundSetSimulationDistancePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_simulation_distance::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.simulation_distance);
    }
}
