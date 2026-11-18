//! `ClientboundDebugEntityValuePacket` — debug-значение entity.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::util::debug::debug_subscription::Update;

/// `ClientboundDebugEntityValuePacket`.
#[derive(Debug)]
pub struct ClientboundDebugEntityValuePacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `update`.
    pub update: Update,
}

impl ClientboundDebugEntityValuePacket {
    /// `new`.
    pub fn new(entity_id: i32, update: Update) -> Self {
        Self { entity_id, update }
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(VAR_INT, update, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDebugEntityValuePacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundDebugEntityValuePacket| {
                buf.write_var_int(pkt.entity_id);
                Update::stream_codec().encode(buf, &pkt.update);
            },
            |buf: &mut FriendlyByteBuf| {
                let entity_id = buf.read_var_int();
                let update = Update::stream_codec().decode(buf);
                Self::new(entity_id, update)
            },
        )
    }
}

impl Packet for ClientboundDebugEntityValuePacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_debug_entity_value::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity_id);
        Update::stream_codec().encode(buf, &self.update);
    }
}
