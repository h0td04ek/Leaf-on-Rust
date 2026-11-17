//! Перенос `ServerboundPlayerAbilitiesPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

const FLAG_FLYING: i32 = 2;

/// `ServerboundPlayerAbilitiesPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPlayerAbilitiesPacket {
    /// `isFlying`.
    pub is_flying: bool,
}

impl ServerboundPlayerAbilitiesPacket {
    /// `new ServerboundPlayerAbilitiesPacket(isFlying)`.
    pub fn new(is_flying: bool) -> Self {
        Self { is_flying }
    }

    /// `isFlying`.
    pub fn is_flying(&self) -> bool {
        self.is_flying
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPlayerAbilitiesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPlayerAbilitiesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let flags = buf.read_byte() as i32;

                ServerboundPlayerAbilitiesPacket::new(flags & FLAG_FLYING != 0)
            },
        )
    }
}

impl Packet for ServerboundPlayerAbilitiesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_player_abilities::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        let mut flags: i32 = 0;
        if self.is_flying {
            flags |= FLAG_FLYING;
        }
        buf.write_byte(flags);
    }
}
