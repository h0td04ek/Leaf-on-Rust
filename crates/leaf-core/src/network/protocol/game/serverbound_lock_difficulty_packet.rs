//! Перенос `ServerboundLockDifficultyPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundLockDifficultyPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundLockDifficultyPacket {
    /// `locked`.
    pub locked: bool,
}

impl ServerboundLockDifficultyPacket {
    pub fn new(locked: bool) -> Self {
        Self { locked }
    }

    /// `isLocked`.
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundLockDifficultyPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundLockDifficultyPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| ServerboundLockDifficultyPacket::new(buf.read_bool()),
        )
    }
}

impl Packet for ServerboundLockDifficultyPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_lock_difficulty::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_bool(self.locked);
    }
}
