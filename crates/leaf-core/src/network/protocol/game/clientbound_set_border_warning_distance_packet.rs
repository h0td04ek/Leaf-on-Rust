//! Перенос `ClientboundSetBorderWarningDistancePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundSetBorderWarningDistancePacket`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClientboundSetBorderWarningDistancePacket {
    /// `warningBlocks`.
    pub warning_blocks: i32,
}

impl ClientboundSetBorderWarningDistancePacket {
    /// `ClientboundSetBorderWarningDistancePacket::new`.
    pub fn new(warning_blocks: i32) -> Self {
        Self { warning_blocks }
    }
    /// `warningBlocks` getter.
    pub fn get_warning_blocks(&self) -> i32 {
        self.warning_blocks
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetBorderWarningDistancePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetBorderWarningDistancePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let warning_blocks = buf.read_var_int();

                ClientboundSetBorderWarningDistancePacket::new(warning_blocks)
            },
        )
    }
}

impl Packet for ClientboundSetBorderWarningDistancePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_border_warning_distance::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.warning_blocks);
    }
}
