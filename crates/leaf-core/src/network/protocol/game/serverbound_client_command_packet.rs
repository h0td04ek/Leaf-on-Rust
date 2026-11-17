//! Перенос `ServerboundClientCommandPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundClientCommandPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientCommandAction {
    /// `PERFORM_RESPAWN`.
    PerformRespawn = 0,
    /// `REQUEST_STATS`.
    RequestStats = 1,
}

impl ClientCommandAction {
    /// `values()`-порядок.
    pub const VALUES: [ClientCommandAction; 2] = [
        ClientCommandAction::PerformRespawn,
        ClientCommandAction::RequestStats,
    ];

    /// `byId` — ordinal (`readEnum`).
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(2) as usize]
    }

    /// `ordinal` (`writeEnum` = VarInt ordinal).
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// `ServerboundClientCommandPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundClientCommandPacket {
    /// `action`.
    pub action: ClientCommandAction,
}

impl ServerboundClientCommandPacket {
    pub fn new(action: ClientCommandAction) -> Self {
        Self { action }
    }

    /// `getAction`.
    pub fn get_action(&self) -> ClientCommandAction {
        self.action
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundClientCommandPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundClientCommandPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundClientCommandPacket::new(
                    ClientCommandAction::by_id(buf.read_var_int()),
                )
            },
        )
    }
}

impl Packet for ServerboundClientCommandPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_client_command::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.action.ordinal());
    }
}
