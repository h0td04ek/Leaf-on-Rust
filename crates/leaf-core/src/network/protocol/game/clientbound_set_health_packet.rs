//! Перенос `ClientboundSetHealthPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetHealthPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetHealthPacket {
    /// `health`.
    pub health: f32,
    /// `food`.
    pub food: i32,
    /// `saturation`.
    pub saturation: f32,
}

impl ClientboundSetHealthPacket {
    pub fn new(health: f32, food: i32, saturation: f32) -> Self {
        Self { health, food, saturation }
    }

    /// `getHealth`.
    pub fn get_health(&self) -> f32 {
        self.health
    }

    /// `getFood`.
    pub fn get_food(&self) -> i32 {
        self.food
    }

    /// `getSaturation`.
    pub fn get_saturation(&self) -> f32 {
        self.saturation
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetHealthPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetHealthPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let health = buf.read_float();
                let food = buf.read_var_int();
                let saturation = buf.read_float();
                ClientboundSetHealthPacket::new(health, food, saturation)
            },
        )
    }
}

impl Packet for ClientboundSetHealthPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_health::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.health);
        buf.write_var_int(self.food);
        buf.write_float(self.saturation);
    }
}
