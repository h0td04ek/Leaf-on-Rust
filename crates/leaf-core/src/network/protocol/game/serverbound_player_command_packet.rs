//! Перенос `ServerboundPlayerCommandPacket` (+ `Action`).

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundPlayerCommandPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerCommandAction {
    /// `STOP_SLEEPING`.
    StopSleeping = 0,
    /// `START_SPRINTING`.
    StartSprinting = 1,
    /// `STOP_SPRINTING`.
    StopSprinting = 2,
    /// `START_RIDING_JUMP`.
    StartRidingJump = 3,
    /// `STOP_RIDING_JUMP`.
    StopRidingJump = 4,
    /// `OPEN_INVENTORY`.
    OpenInventory = 5,
    /// `START_FALL_FLYING`.
    StartFallFlying = 6,
}

impl PlayerCommandAction {
    /// `values()`-порядок.
    pub const VALUES: [PlayerCommandAction; 7] = [
        PlayerCommandAction::StopSleeping,
        PlayerCommandAction::StartSprinting,
        PlayerCommandAction::StopSprinting,
        PlayerCommandAction::StartRidingJump,
        PlayerCommandAction::StopRidingJump,
        PlayerCommandAction::OpenInventory,
        PlayerCommandAction::StartFallFlying,
    ];

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(7) as usize]
    }

    /// `ordinal`.
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// `ServerboundPlayerCommandPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPlayerCommandPacket {
    /// `id` — entity id.
    pub id: i32,
    /// `action`.
    pub action: PlayerCommandAction,
    /// `data` — доп. параметр (jump boost ticks и т.п.).
    pub data: i32,
}

impl ServerboundPlayerCommandPacket {
    /// `new ServerboundPlayerCommandPacket(id, action)`.
    pub fn new(id: i32, action: PlayerCommandAction) -> Self {
        Self { id, action, data: 0 }
    }

    /// `new ServerboundPlayerCommandPacket(id, action, data)`.
    pub fn with_data(id: i32, action: PlayerCommandAction, data: i32) -> Self {
        Self { id, action, data }
    }

    /// `getId`.
    pub fn get_id(&self) -> i32 {
        self.id
    }

    /// `getAction`.
    pub fn get_action(&self) -> PlayerCommandAction {
        self.action
    }

    /// `getData`.
    pub fn get_data(&self) -> i32 {
        self.data
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPlayerCommandPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPlayerCommandPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let action = PlayerCommandAction::by_id(buf.read_var_int());
                let data = buf.read_var_int();
                ServerboundPlayerCommandPacket::with_data(id, action, data)
            },
        )
    }
}

impl Packet for ServerboundPlayerCommandPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_player_command::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.id);
        buf.write_var_int(self.action.ordinal());
        buf.write_var_int(self.data);
    }
}
