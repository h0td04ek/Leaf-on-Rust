//! Перенос `ClientboundCustomQueryPacket` (+ Paper `PlayerInfoChannelPayload`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::login::custom::custom_query_payload::CustomQueryPayload;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

/// `MAX_PAYLOAD_SIZE`.
const MAX_PAYLOAD_SIZE: usize = 1048576;

static CUSTOM_QUERY_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("custom_query").unwrap(),
    )
});

/// `ClientboundCustomQueryPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundCustomQueryPacket {
    /// `transactionId`.
    pub transaction_id: i32,
    /// `payload`.
    pub payload: CustomQueryPayload,
}

impl ClientboundCustomQueryPacket {
    pub fn new(transaction_id: i32, payload: CustomQueryPayload) -> Self {
        Self { transaction_id, payload }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundCustomQueryPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundCustomQueryPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let transaction_id = buf.read_var_int();
                let id = buf.read_identifier().expect("invalid payload id");
                let payload = read_payload(id, buf);
                ClientboundCustomQueryPacket { transaction_id, payload }
            },
        )
    }
}

/// `readPayload` → `readUnknownPayload` — неизвестный payload: пропускаем
/// оставшиеся байты и возвращаем `Discarded`.
fn read_payload(id: Identifier, buffer: &mut FriendlyByteBuf) -> CustomQueryPayload {
    let readable = buffer.readable_bytes();
    if readable > MAX_PAYLOAD_SIZE {
        panic!("Payload may not be larger than {} bytes", MAX_PAYLOAD_SIZE);
    }
    if readable == 0 {
        return CustomQueryPayload::Discarded { id };
    }
    let data = buffer.read_bytes_vec(readable);
    CustomQueryPayload::PlayerInfoChannel { id, data }
}

impl Packet for ClientboundCustomQueryPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CUSTOM_QUERY_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        buf.write_identifier(self.payload.id()).expect("invalid identifier");
        self.payload.write(buf);
    }
}
