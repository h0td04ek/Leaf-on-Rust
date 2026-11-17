//! Перенос `ServerboundSetCommandMinecartPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundSetCommandMinecartPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ServerboundSetCommandMinecartPacket {
    /// `entity`.
    pub entity: i32,
    /// `command`.
    pub command: String,
    /// `trackOutput`.
    pub track_output: bool,
}

impl ServerboundSetCommandMinecartPacket {
    /// `new`.
    pub fn new(entity: i32, command: String, track_output: bool) -> Self {
        Self { entity, command, track_output }
    }

    /// `getCommand`/`isTrackOutput`.
    pub fn get_command(&self) -> &str {
        &self.command
    }
    /// `isTrackOutput`.
    pub fn is_track_output(&self) -> bool {
        self.track_output
    }
    /// `entity` id — для `getCommandBlock(level)` при порте level.
    pub fn entity(&self) -> i32 {
        self.entity
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundSetCommandMinecartPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundSetCommandMinecartPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let entity = buf.read_var_int();
                let command = buf.read_utf().expect("invalid utf");
                let track_output = buf.read_bool();
                ServerboundSetCommandMinecartPacket::new(entity, command, track_output)
            },
        )
    }
}

impl Packet for ServerboundSetCommandMinecartPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_set_command_minecart::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.entity);
        buf.write_utf(&self.command).expect("invalid utf");
        buf.write_bool(self.track_output);
    }
}
