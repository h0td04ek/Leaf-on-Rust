//! Перенос `ClientboundSetBorderWarningDelayPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetBorderWarningDelayPacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetBorderWarningDelayPacket {
    /// `warningDelay`.
    pub warning_delay: i32,
}

impl ClientboundSetBorderWarningDelayPacket {
    /// `ClientboundSetBorderWarningDelayPacket::new`.
    pub fn new(warning_delay: i32) -> Self {
        Self { warning_delay }
    }
    /// `warningDelay` getter.
    pub fn get_warning_delay(&self) -> i32 {
        self.warning_delay
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetBorderWarningDelayPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetBorderWarningDelayPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let warning_delay = buf.read_var_int();

                ClientboundSetBorderWarningDelayPacket::new(warning_delay)
            },
        )
    }
}

impl Packet for ClientboundSetBorderWarningDelayPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_border_warning_delay::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.warning_delay);
    }
}
