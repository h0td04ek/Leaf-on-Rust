//! Перенос `ServerboundSetCommandBlockPacket`.

use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::world::level::block::entity::command_block_entity::CommandBlockMode;

const FLAG_TRACK_OUTPUT: i32 = 1;
const FLAG_CONDITIONAL: i32 = 2;
const FLAG_AUTOMATIC: i32 = 4;

/// `ServerboundSetCommandBlockPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSetCommandBlockPacket {
    /// `pos`.
    pub pos: BlockPos,
    /// `command`.
    pub command: String,
    /// `trackOutput`.
    pub track_output: bool,
    /// `conditional`.
    pub conditional: bool,
    /// `automatic`.
    pub automatic: bool,
    /// `mode`.
    pub mode: CommandBlockMode,
}

impl ServerboundSetCommandBlockPacket {
    /// `new`.
    pub fn new(
        pos: BlockPos,
        command: String,
        mode: CommandBlockMode,
        track_output: bool,
        conditional: bool,
        automatic: bool,
    ) -> Self {
        Self { pos, command, track_output, conditional, automatic, mode }
    }

    /// Геттеры.
    pub fn pos(&self) -> BlockPos {
        self.pos
    }
    /// `command`.
    pub fn command(&self) -> &str {
        &self.command
    }
    /// `isTrackOutput`.
    pub fn is_track_output(&self) -> bool {
        self.track_output
    }
    /// `isConditional`.
    pub fn is_conditional(&self) -> bool {
        self.conditional
    }
    /// `isAutomatic`.
    pub fn is_automatic(&self) -> bool {
        self.automatic
    }
    /// `getMode`.
    pub fn get_mode(&self) -> CommandBlockMode {
        self.mode
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetCommandBlockPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetCommandBlockPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let pos = buf.read_block_pos();
                let command = buf.read_utf().expect("invalid utf");
                let ordinal = buf.read_var_int();
                let mode = CommandBlockMode::by_ordinal(ordinal);
                let flags = buf.read_byte() as i32;
                let track_output = (flags & FLAG_TRACK_OUTPUT) != 0;
                let conditional = (flags & FLAG_CONDITIONAL) != 0;
                let automatic = (flags & FLAG_AUTOMATIC) != 0;
                ServerboundSetCommandBlockPacket::new(
                    pos,
                    command,
                    mode,
                    track_output,
                    conditional,
                    automatic,
                )
            },
        )
    }
}

impl Packet for ServerboundSetCommandBlockPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_command_block::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_block_pos(&self.pos);
        buf.write_utf(&self.command).expect("invalid utf");
        buf.write_var_int(self.mode as i32);
        let mut i = 0;
        if self.track_output {
            i |= FLAG_TRACK_OUTPUT;
        }
        if self.conditional {
            i |= FLAG_CONDITIONAL;
        }
        if self.automatic {
            i |= FLAG_AUTOMATIC;
        }
        buf.write_byte(i);
    }
}
