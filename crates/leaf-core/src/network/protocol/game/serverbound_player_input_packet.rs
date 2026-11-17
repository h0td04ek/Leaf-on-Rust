//! Перенос `ServerboundPlayerInputPacket`.

use crate::entity::player::input::Input;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};

/// `ServerboundPlayerInputPacket`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ServerboundPlayerInputPacket {
    /// `input`.
    pub input: Input,
}

impl ServerboundPlayerInputPacket {
    pub fn new(input: Input) -> Self {
        Self { input }
    }

    /// `input()`.
    pub fn input(&self) -> Input {
        self.input
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ServerboundPlayerInputPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ServerboundPlayerInputPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                ServerboundPlayerInputPacket::new(Input::stream_codec().decode(buf))
            },
        )
    }
}

impl Packet for ServerboundPlayerInputPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::serverbound_player_input::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        Input::stream_codec().encode(buf, &self.input);
    }
}
