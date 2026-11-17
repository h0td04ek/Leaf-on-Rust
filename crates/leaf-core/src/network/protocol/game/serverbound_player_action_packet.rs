//! Перенос `ServerboundPlayerActionPacket` (+ `Action`).

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundPlayerActionPacket.Action`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    /// `START_DESTROY_BLOCK`.
    StartDestroyBlock = 0,
    /// `ABORT_DESTROY_BLOCK`.
    AbortDestroyBlock = 1,
    /// `STOP_DESTROY_BLOCK`.
    StopDestroyBlock = 2,
    /// `DROP_ALL_ITEMS`.
    DropAllItems = 3,
    /// `DROP_ITEM`.
    DropItem = 4,
    /// `RELEASE_USE_ITEM`.
    ReleaseUseItem = 5,
    /// `SWAP_ITEM_WITH_OFFHAND`.
    SwapItemWithOffhand = 6,
    /// `STAB`.
    Stab = 7,
}

impl PlayerAction {
    /// `values()`-порядок.
    pub const VALUES: [PlayerAction; 8] = [
        PlayerAction::StartDestroyBlock,
        PlayerAction::AbortDestroyBlock,
        PlayerAction::StopDestroyBlock,
        PlayerAction::DropAllItems,
        PlayerAction::DropItem,
        PlayerAction::ReleaseUseItem,
        PlayerAction::SwapItemWithOffhand,
        PlayerAction::Stab,
    ];

    /// `byId` — ordinal.
    pub fn by_id(id: i32) -> Self {
        Self::VALUES[id.rem_euclid(8) as usize]
    }

    /// `ordinal`.
    pub fn ordinal(&self) -> i32 {
        *self as i32
    }
}

/// `ServerboundPlayerActionPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPlayerActionPacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `direction`.
    pub direction: Direction,
    /// `action`.
    pub action: PlayerAction,
    /// `sequence`.
    pub sequence: i32,
}

impl ServerboundPlayerActionPacket {
    /// `new ServerboundPlayerActionPacket(action, pos, direction)` — sequence=0.
    pub fn new(action: PlayerAction, pos: BlockPos, direction: Direction) -> Self {
        Self { action, pos, direction, sequence: 0 }
    }

    /// `new ServerboundPlayerActionPacket(action, pos, direction, sequence)`.
    pub fn with_sequence(
        action: PlayerAction,
        pos: BlockPos,
        direction: Direction,
        sequence: i32,
    ) -> Self {
        Self { action, pos, direction, sequence }
    }

    /// `getPos`.
    pub fn get_pos(&self) -> BlockPos {
        self.pos
    }

    /// `getDirection`.
    pub fn get_direction(&self) -> Direction {
        self.direction
    }

    /// `getAction`.
    pub fn get_action(&self) -> PlayerAction {
        self.action
    }

    /// `getSequence`.
    pub fn get_sequence(&self) -> i32 {
        self.sequence
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPlayerActionPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPlayerActionPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let action = PlayerAction::by_id(buf.read_var_int());
                let pos = buf.read_block_pos();
                let direction = Direction::from_3d_data_value(buf.read_unsigned_byte());
                let sequence = buf.read_var_int();
                ServerboundPlayerActionPacket::with_sequence(action, pos, direction, sequence)
            },
        )
    }
}

impl Packet for ServerboundPlayerActionPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_player_action::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.action.ordinal());
        buf.write_block_pos(&self.pos);
        buf.write_byte(self.direction.get_3d_data_value());
        buf.write_var_int(self.sequence);
    }
}
