//! Перенос `ClientboundSetPassengersPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetPassengersPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundSetPassengersPacket {
    /// `vehicle` (entity id).
    pub vehicle: i32,
    /// `passengers` (entity ids).
    pub passengers: Vec<i32>,
}

impl ClientboundSetPassengersPacket {
    /// `ClientboundSetPassengersPacket::new` (поля пакета).
    pub fn new(vehicle: i32, passengers: Vec<i32>) -> Self {
        Self { vehicle, passengers }
    }

    /// `getPassengers`.
    pub fn get_passengers(&self) -> &[i32] {
        &self.passengers
    }

    /// `getVehicle`.
    pub fn get_vehicle(&self) -> i32 {
        self.vehicle
    }

    /// `STREAM_CODEC` — VAR_INT + VarIntArray.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetPassengersPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetPassengersPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let vehicle = buf.read_var_int();
                let passengers = buf.read_var_int_array();
                ClientboundSetPassengersPacket::new(vehicle, passengers)
            },
        )
    }
}

impl Packet for ClientboundSetPassengersPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_passengers::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.vehicle);
        buf.write_var_int_array(&self.passengers);
    }
}
