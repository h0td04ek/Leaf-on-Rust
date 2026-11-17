//! Перенос `ClientboundLevelEventPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::core::block_pos::BlockPos;

/// `ClientboundLevelEventPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundLevelEventPacket {
    /// `type` — тип level event.
    pub event_type: i32,
    /// `pos`.
    pub pos: BlockPos,
    /// `data`.
    pub data: i32,
    /// `globalEvent`.
    pub global_event: bool,
}

impl ClientboundLevelEventPacket {
    pub fn new(event_type: i32, pos: BlockPos, data: i32, global_event: bool) -> Self {
        Self { event_type, pos, data, global_event }
    }

    /// `isGlobalEvent`.
    pub fn is_global_event(&self) -> bool {
        self.global_event
    }

    /// `getType`.
    pub fn get_type(&self) -> i32 {
        self.event_type
    }

    /// `getData`.
    pub fn get_data(&self) -> i32 {
        self.data
    }

    /// `getPos`.
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundLevelEventPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundLevelEventPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let event_type = buf.read_int();
                let pos = buf.read_block_pos();
                let data = buf.read_int();
                let global_event = buf.read_bool();

                ClientboundLevelEventPacket::new(event_type, pos, data, global_event)
            },
        )
    }
}

impl Packet for ClientboundLevelEventPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_level_event::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_int(self.event_type);
        buf.write_block_pos(&self.pos);
        buf.write_int(self.data);
        buf.write_bool(self.global_event);
    }
}
