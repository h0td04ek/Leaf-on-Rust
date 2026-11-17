//! Перенос `ClientboundAnimatePacket` (+ константы action).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundAnimatePacket.SWING_MAIN_HAND`.
pub const SWING_MAIN_HAND: i32 = 0;
/// `ClientboundAnimatePacket.WAKE_UP`.
pub const WAKE_UP: i32 = 2;
/// `ClientboundAnimatePacket.SWING_OFF_HAND`.
pub const SWING_OFF_HAND: i32 = 3;
/// `ClientboundAnimatePacket.CRITICAL_HIT`.
pub const CRITICAL_HIT: i32 = 4;
/// `ClientboundAnimatePacket.MAGIC_CRITICAL_HIT`.
pub const MAGIC_CRITICAL_HIT: i32 = 5;

/// `ClientboundAnimatePacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundAnimatePacket {
    /// `id` — entity id.
    pub id: i32,
    /// `action`.
    pub action: i32,
}

impl ClientboundAnimatePacket {
    /// `new ClientboundAnimatePacket(id, action)`.
    pub fn new(id: i32, action: i32) -> Self {
        Self { id, action }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `getAction`.
    pub fn get_action(&self) -> i32 {
        self.action
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundAnimatePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundAnimatePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let action = buf.read_unsigned_byte();
                ClientboundAnimatePacket::new(id, action)
            },
        )
    }
}

impl Packet for ClientboundAnimatePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_animate::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_byte(self.action);
    }
}
