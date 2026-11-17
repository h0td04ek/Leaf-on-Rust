//! Перенос `ClientboundTickingStatePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundTickingStatePacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundTickingStatePacket {
    /// `tickRate`.
    pub tick_rate: f32,
    /// `isFrozen`.
    pub is_frozen: bool,
}

impl ClientboundTickingStatePacket {
    /// `new`.
    pub fn new(tick_rate: f32, is_frozen: bool) -> Self {
        Self { tick_rate, is_frozen }
    }

    /// `from(TickRateManager)`.
    pub fn from_tick_rate_manager(tick_rate: f32, is_frozen: bool) -> Self {
        Self::new(tick_rate, is_frozen)
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTickingStatePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTickingStatePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let tick_rate = buf.read_float();
                let is_frozen = buf.read_bool();
                ClientboundTickingStatePacket::new(tick_rate, is_frozen)
            },
        )
    }
}

impl Packet for ClientboundTickingStatePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_ticking_state::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_float(self.tick_rate);
        buf.write_bool(self.is_frozen);
    }
}
