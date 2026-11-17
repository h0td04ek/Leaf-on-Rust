//! Перенос `ClientboundSetTimePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetTimePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundSetTimePacket {
    /// `gameTime`.
    pub game_time: i64,
    /// `dayTime`.
    pub day_time: i64,
    /// `tickDayTime`.
    pub tick_day_time: bool,
}

impl ClientboundSetTimePacket {
    pub fn new(game_time: i64, day_time: i64, tick_day_time: bool) -> Self {
        Self { game_time, day_time, tick_day_time }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetTimePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetTimePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let game_time = buf.read_long();
                let day_time = buf.read_long();
                let tick_day_time = buf.read_bool();
                ClientboundSetTimePacket::new(game_time, day_time, tick_day_time)
            },
        )
    }
}

impl Packet for ClientboundSetTimePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_time::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_long(self.game_time);
        buf.write_long(self.day_time);
        buf.write_bool(self.tick_day_time);
    }
}
