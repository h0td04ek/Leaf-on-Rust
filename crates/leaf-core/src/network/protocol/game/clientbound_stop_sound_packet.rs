//! Перенос `ClientboundStopSoundPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::identifier::Identifier;
use crate::sounds::sound_source::SoundSource;

/// `ClientboundStopSoundPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundStopSoundPacket {
    /// `name`.
    pub name: Option<Identifier>,
    /// `source`.
    pub source: Option<SoundSource>,
}

impl ClientboundStopSoundPacket {
    /// `ClientboundStopSoundPacket::new`.
    pub fn new(name: Option<Identifier>, source: Option<SoundSource>) -> Self {
        Self { name, source }
    }
    /// `getName`.
    pub fn get_name(&self) -> Option<&Identifier> { self.name.as_ref() }
    /// `getSource`.
    pub fn get_source(&self) -> Option<SoundSource> { self.source }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundStopSoundPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundStopSoundPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let flags = buf.read_byte() as u8;
                let source = if flags & 1 != 0 { Some(SoundSource::by_id(buf.read_var_int())) } else { None };
                let name = if flags & 2 != 0 {
                    Some(buf.read_identifier().expect("invalid identifier"))
                } else {
                    None
                };
                ClientboundStopSoundPacket::new(name, source)            },
        )
    }
}

impl Packet for ClientboundStopSoundPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_stop_sound::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        let flags = (if self.source.is_some() { 1u8 } else { 0 })
            | (if self.name.is_some() { 2u8 } else { 0 });
        buf.write_byte(flags as i32);
        if let Some(source) = &self.source {
            buf.write_var_int(source.ordinal());
        }
        if let Some(name) = &self.name {
            buf.write_identifier(name).expect("invalid identifier");
        }    }
}
