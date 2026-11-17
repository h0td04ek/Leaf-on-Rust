//! Перенос `ClientboundPlayerCombatKillPacket`.

use crate::network::chat::component::Component;
use crate::network::chat::component_json;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ClientboundPlayerCombatKillPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPlayerCombatKillPacket {
    /// `playerId`.
    pub player_id: i32,
    /// `message`.
    pub message: Component,
}

impl ClientboundPlayerCombatKillPacket {
    pub fn new(player_id: i32, message: Component) -> Self {
        Self { player_id, message }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerCombatKillPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerCombatKillPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let player_id = buf.read_var_int();
                let message = component_json::component_stream_codec().decode(buf);

                ClientboundPlayerCombatKillPacket::new(player_id, message)
            },
        )
    }
}

impl Packet for ClientboundPlayerCombatKillPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_combat_kill::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.player_id);
        component_json::component_stream_codec().encode(buf, &self.message);
    }

    fn is_skippable(&self) -> bool {
        true
    }
}
