//! Перенос `ServerboundCustomQueryAnswerPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::login::custom::custom_query_answer_payload::CustomQueryAnswerPayload;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::sync::LazyLock;

/// `MAX_PAYLOAD_SIZE`.
const MAX_PAYLOAD_SIZE: usize = 1048576;

static CUSTOM_QUERY_ANSWER_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::SERVERBOUND,
        Identifier::with_default_namespace("custom_query_answer").unwrap(),
    )
});

/// `ServerboundCustomQueryAnswerPacket`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerboundCustomQueryAnswerPacket {
    /// `transactionId`.
    pub transaction_id: i32,
    /// `payload` — `None` = ответ "нет данных".
    pub payload: Option<CustomQueryAnswerPayload>,
}

impl ServerboundCustomQueryAnswerPacket {
    /// `QueryAnswerPayload` — конструктор из сырых байт.
    pub fn query_answer(transaction_id: i32, data: Vec<u8>) -> Self {
        Self {
            transaction_id,
            payload: Some(CustomQueryAnswerPayload::QueryAnswer { data }),
        }
    }

    /// `DiscardedQueryAnswerPayload` — ответ с отбрасываемым телом.
    pub fn discarded(transaction_id: i32) -> Self {
        Self {
            transaction_id,
            payload: Some(CustomQueryAnswerPayload::Discarded),
        }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundCustomQueryAnswerPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundCustomQueryAnswerPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let transaction_id = buf.read_var_int();
                let payload = read_payload(transaction_id, buf);
                ServerboundCustomQueryAnswerPacket { transaction_id, payload }
            },
        )
    }
}

/// `readPayload` — nullable payload (остальные байты → QueryAnswer).
fn read_payload(transaction_id: i32, buffer: &mut FriendlyByteBuf) -> Option<CustomQueryAnswerPayload> {
    let _ = transaction_id;
    if !buffer.read_bool() {
        return None;
    }
    let readable = buffer.readable_bytes();
    if readable > MAX_PAYLOAD_SIZE {
        panic!("Payload may not be larger than {} bytes", MAX_PAYLOAD_SIZE);
    }
    let data = buffer.read_bytes_vec(readable);
    Some(CustomQueryAnswerPayload::QueryAnswer { data })
}

impl Packet for ServerboundCustomQueryAnswerPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        CUSTOM_QUERY_ANSWER_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.transaction_id);
        match &self.payload {
            Some(p) => {
                buf.write_bool(true);
                p.write(buf);
            }
            None => {
                buf.write_bool(false);
            }
        }
    }
}
