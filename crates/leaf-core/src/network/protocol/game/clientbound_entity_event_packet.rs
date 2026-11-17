//! Перенос `ClientboundEntityEventPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundEntityEventPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundEntityEventPacket {
    /// `entityId`.
    pub entity_id: i32,
    /// `eventId` — тип entity event.
    pub event_id: i8,
}

impl ClientboundEntityEventPacket {
    pub fn new(entity_id: i32, event_id: i8) -> Self {
        Self { entity_id, event_id }
    }

    /// `getEventId`.
    pub fn get_event_id(&self) -> i8 {
        self.event_id
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundEntityEventPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundEntityEventPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundEntityEventPacket::new(buf.read_int(), buf.read_byte())
            },
        )
    }
}

impl Packet for ClientboundEntityEventPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_entity_event::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.entity_id);
        buf.write_byte(self.event_id as i32);
    }
}
