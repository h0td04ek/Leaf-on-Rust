//! Перенос `ClientboundStopSoundPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::identifier::Identifier;
use crate::sounds::sound_source::SoundSource;

const HAS_SOURCE: i32 = 1;
const HAS_SOUND: i32 = 2;

/// `ClientboundStopSoundPacket`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundStopSoundPacket {
    /// `name` — `None` = все звуки источника.
    pub name: Option<Identifier>,
    /// `source` — `None` = все источники.
    pub source: Option<SoundSource>,
}

impl ClientboundStopSoundPacket {
    /// `new ClientboundStopSoundPacket(name, source)`.
    pub fn new(name: Option<Identifier>, source: Option<SoundSource>) -> Self {
        Self { name, source }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundStopSoundPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundStopSoundPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let flags = buf.read_byte() as i32;
                let source = if flags & HAS_SOURCE != 0 {
                    Some(SoundSource::by_id(buf.read_var_int()))
                } else {
                    None
                };
                let name = if flags & HAS_SOUND != 0 {
                    Some(buf.read_identifier().expect("invalid sound name"))
                } else {
                    None
                };
                ClientboundStopSoundPacket::new(name, source)
            },
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
        let mut flags: i32 = 0;
        if self.source.is_some() {
            flags |= HAS_SOURCE;
        }
        if self.name.is_some() {
            flags |= HAS_SOUND;
        }
        buf.write_byte(flags);
        if let Some(source) = self.source {
            buf.write_var_int(source.ordinal());
        }
        if let Some(name) = &self.name {
            buf.write_identifier(name).expect("invalid identifier");
        }
    }
}
