//! Перенос `ClientboundDebugSamplePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::util::debugchart::remote_debug_sample_type::RemoteDebugSampleType;

/// `ClientboundDebugSamplePacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundDebugSamplePacket {
    /// `sample` — long[].
    pub sample: Vec<i64>,
    /// `debugSampleType`.
    pub debug_sample_type: RemoteDebugSampleType,
}

impl ClientboundDebugSamplePacket {
    pub fn new(sample: Vec<i64>, debug_sample_type: RemoteDebugSampleType) -> Self {
        Self { sample, debug_sample_type }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDebugSamplePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundDebugSamplePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let sample = buf.read_long_array();
                let ordinal = buf.read_var_int();
                let debug_sample_type = RemoteDebugSampleType::by_id(ordinal);
                ClientboundDebugSamplePacket::new(sample, debug_sample_type)
            },
        )
    }
}

impl Packet for ClientboundDebugSamplePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_debug_sample::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_long_array(&self.sample);
        buf.write_var_int(self.debug_sample_type.ordinal());
    }
}
