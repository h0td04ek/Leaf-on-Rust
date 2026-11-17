//! Перенос `ClientboundProjectilePowerPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundProjectilePowerPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundProjectilePowerPacket {
    /// `id`.
    pub id: i32,
    /// `accelerationPower`.
    pub acceleration_power: f64,
}

impl ClientboundProjectilePowerPacket {
    /// `ClientboundProjectilePowerPacket::new`.
    pub fn new(id: i32, acceleration_power: f64) -> Self {
        Self { id, acceleration_power }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `getAccelerationPower`.
    pub fn get_acceleration_power(&self) -> f64 {
        self.acceleration_power
    }

    /// `STREAM_CODEC` — `Packet.codec`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundProjectilePowerPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundProjectilePowerPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let acceleration_power = buf.read_double();
                ClientboundProjectilePowerPacket::new(id, acceleration_power)
            },
        )
    }
}

impl Packet for ClientboundProjectilePowerPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_projectile_power::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_double(self.acceleration_power);
    }
}
