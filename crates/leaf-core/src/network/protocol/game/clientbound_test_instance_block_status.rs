//! `ClientboundTestInstanceBlockStatus` — статус тестового инстанса.

use crate::core::vec3i::Vec3i;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::chat::component_json::component_stream_codec;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundTestInstanceBlockStatus`.
#[derive(Clone, Debug)]
pub struct ClientboundTestInstanceBlockStatus {
    /// `status` — Component.
    pub status: crate::network::chat::component::Component,
    /// `size` — Optional<Vec3i>.
    pub size: Option<Vec3i>,
}

impl ClientboundTestInstanceBlockStatus {
    /// `new`.
    pub fn new(status: crate::network::chat::component::Component, size: Option<Vec3i>) -> Self {
        Self { status, size }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundTestInstanceBlockStatus> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundTestInstanceBlockStatus| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let status = component_stream_codec().decode(buf);
                let size = if buf.read_bool() {
                    Some(Vec3i::new(buf.read_var_int(), buf.read_var_int(), buf.read_var_int()))
                } else { None };
                ClientboundTestInstanceBlockStatus::new(status, size)
            },
        )
    }
}

impl Packet for ClientboundTestInstanceBlockStatus {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_test_instance_block_status::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        component_stream_codec().encode(buf, &self.status);
        match &self.size {
            Some(v) => {
                buf.write_bool(true);
                buf.write_var_int(v.x);
                buf.write_var_int(v.y);
                buf.write_var_int(v.z);
            }
            None => { buf.write_bool(false); }
        }
    }
}
