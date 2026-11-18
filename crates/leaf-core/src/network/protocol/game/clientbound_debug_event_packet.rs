//! `ClientboundDebugEventPacket` — debug-событие.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::util::debug::debug_subscription::Event;

/// `ClientboundDebugEventPacket`.
#[derive(Debug)]
pub struct ClientboundDebugEventPacket {
    /// `event`.
    pub event: Event,
}

impl ClientboundDebugEventPacket {
    /// `new`.
    pub fn new(event: Event) -> Self {
        Self { event }
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(Event.STREAM_CODEC, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundDebugEventPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundDebugEventPacket| {
                Event::stream_codec().encode(buf, &pkt.event);
            },
            |buf: &mut FriendlyByteBuf| {
                let event = Event::stream_codec().decode(buf);
                Self::new(event)
            },
        )
    }
}

impl Packet for ClientboundDebugEventPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_debug_event::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Event::stream_codec().encode(buf, &self.event);
    }
}
