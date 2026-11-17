//! Перенос `ClientboundPlayerCombatEndPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundPlayerCombatEndPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundPlayerCombatEndPacket {
    /// `duration` — длительность combat-режима в тиках.
    pub duration: i32,
}

impl ClientboundPlayerCombatEndPacket {
    pub fn new(duration: i32) -> Self {
        Self { duration }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerCombatEndPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerCombatEndPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ClientboundPlayerCombatEndPacket::new(buf.read_var_int())
            },
        )
    }
}

impl Packet for ClientboundPlayerCombatEndPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_combat_end::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.duration);
    }
}
