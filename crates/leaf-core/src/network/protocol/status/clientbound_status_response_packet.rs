//! Перенос `ClientboundStatusResponsePacket` + `StatusPacketTypes`.

use super::server_status::ServerStatus;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use serde_json::Value;
use std::sync::LazyLock;

static STATUS_RESPONSE_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("status_response").unwrap())
});

/// `ClientboundStatusResponsePacket`.
#[derive(Clone, Debug)]
pub struct ClientboundStatusResponsePacket {
    pub status: ServerStatus,
}

impl ClientboundStatusResponsePacket {
    pub fn new(status: ServerStatus) -> Self {
        Self { status }
    }
}

/// `ByteBufCodecs.lenientJson(32767)` — JSON как UTF-8 строка.
fn write_json(buf: &mut FriendlyByteBuf, json: &str) {
    buf.write_utf_n(json, 32767).expect("status json too long");
}

fn read_json(buf: &mut FriendlyByteBuf) -> Result<String, String> {
    buf.read_utf_n(32767).map_err(|e| e.0)
}

impl ClientboundStatusResponsePacket {
    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundStatusResponsePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundStatusResponsePacket| {
                let json = serde_json::to_string(&p.status.to_json()).expect("status serialization");
                write_json(buf, &json);
            },
            |buf: &mut FriendlyByteBuf| {
                let json = read_json(buf).expect("invalid status json");
                let value: Value = serde_json::from_str(&json).expect("invalid status json");
                let status = ServerStatus::from_json(&value).expect("invalid server status");
                ClientboundStatusResponsePacket::new(status)
            },
        )
    }
}

impl Packet for ClientboundStatusResponsePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        STATUS_RESPONSE_TYPE.clone()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        let json = serde_json::to_string(&self.status.to_json()).expect("status serialization");
        write_json(buf, &json);
    }
}
