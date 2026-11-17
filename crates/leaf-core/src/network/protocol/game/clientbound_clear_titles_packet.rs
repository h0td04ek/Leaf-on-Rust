//! Перенос `ClientboundClearTitlesPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundClearTitlesPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundClearTitlesPacket {
    /// `resetTimes`.
    pub reset_times: bool,
}

impl ClientboundClearTitlesPacket {
    pub fn new(reset_times: bool) -> Self {
        Self { reset_times }
    }

    /// `shouldResetTimes`.
    pub fn should_reset_times(&self) -> bool {
        self.reset_times
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundClearTitlesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundClearTitlesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ClientboundClearTitlesPacket::new(buf.read_bool()),
        )
    }
}

impl Packet for ClientboundClearTitlesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_clear_titles::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_bool(self.reset_times);
    }
}
