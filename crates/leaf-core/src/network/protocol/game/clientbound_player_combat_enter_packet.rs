//! Перенос `ClientboundPlayerCombatEnterPacket` — singleton.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundPlayerCombatEnterPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClientboundPlayerCombatEnterPacket;

/// `INSTANCE`.
pub const CLIENTBOUND_PLAYER_COMBAT_ENTER_PACKET: ClientboundPlayerCombatEnterPacket =
    ClientboundPlayerCombatEnterPacket;

impl ClientboundPlayerCombatEnterPacket {
    /// `INSTANCE`.
    pub fn instance() -> Self {
        Self
    }

    /// `STREAM_CODEC` (`StreamCodec.unit`).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerCombatEnterPacket> {
        packet_codec(
            |_buf: &mut FriendlyByteBuf, _p: &ClientboundPlayerCombatEnterPacket| {},
            |_buf: &mut FriendlyByteBuf| ClientboundPlayerCombatEnterPacket,
        )
    }
}

impl Packet for ClientboundPlayerCombatEnterPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_combat_enter::<Self>().erase()
    }
    fn write_packet(&self, _buf: &mut FriendlyByteBuf) {}
}
