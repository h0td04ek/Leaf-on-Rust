//! Перенос `ClientboundCustomReportDetailsPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::collections::HashMap;
use std::sync::LazyLock;

/// `MAX_DETAIL_KEY_LENGTH`.
pub const MAX_DETAIL_KEY_LENGTH: usize = 128;
/// `MAX_DETAIL_VALUE_LENGTH`.
pub const MAX_DETAIL_VALUE_LENGTH: usize = 4096;
/// `MAX_DETAIL_COUNT`.
pub const MAX_DETAIL_COUNT: usize = 32;

static CUSTOM_REPORT_DETAILS_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("custom_report_details").unwrap())
});

/// `ClientboundCustomReportDetailsPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundCustomReportDetailsPacket {
    /// `details` — до 32 пар (key ≤128, value ≤4096).
    pub details: HashMap<String, String>,
}

impl ClientboundCustomReportDetailsPacket {
    pub fn new(details: HashMap<String, String>) -> Self {
        Self { details }
    }

    /// `DETAILS_STREAM_CODEC` (`ByteBufCodecs.map(..., 32)`).
    fn write_details(buf: &mut FriendlyByteBuf, details: &HashMap<String, String>) {
        buf.write_var_int(details.len() as i32);
        for (k, v) in details {
            buf.write_utf_n(k, MAX_DETAIL_KEY_LENGTH).expect("detail key too long");
            buf.write_utf_n(v, MAX_DETAIL_VALUE_LENGTH).expect("detail value too long");
        }
    }

    fn read_details(buf: &mut FriendlyByteBuf) -> HashMap<String, String> {
        let count = buf.read_var_int();
        let mut map = HashMap::with_capacity(count.max(0) as usize);
        for _ in 0..count {
            let k = buf.read_utf_n(MAX_DETAIL_KEY_LENGTH).expect("invalid detail key");
            let v = buf.read_utf_n(MAX_DETAIL_VALUE_LENGTH).expect("invalid detail value");
            map.insert(k, v);
        }
        map
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCustomReportDetailsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCustomReportDetailsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundCustomReportDetailsPacket::new(Self::read_details(buf))
            },
        )
    }
}

impl Packet for ClientboundCustomReportDetailsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CUSTOM_REPORT_DETAILS_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Self::write_details(buf, &self.details);
    }
}
