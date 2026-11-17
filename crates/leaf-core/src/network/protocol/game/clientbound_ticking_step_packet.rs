//! Перенос `ClientboundTickingStepPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundTickingStepPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundTickingStepPacket {
    /// `tickSteps`.
    pub tick_steps: i32,
}

impl ClientboundTickingStepPacket {
    /// `new`.
    pub fn new(tick_steps: i32) -> Self {
        Self { tick_steps }
    }

    /// `from(TickRateManager)` — `frozenTicksToRun`.
    pub fn from_tick_rate_manager(frozen_ticks_to_run: i32) -> Self {
        Self::new(frozen_ticks_to_run)
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTickingStepPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTickingStepPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let tick_steps = buf.read_var_int();
                ClientboundTickingStepPacket::new(tick_steps)
            },
        )
    }
}

impl Packet for ClientboundTickingStepPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_ticking_step::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.tick_steps);
    }
}
