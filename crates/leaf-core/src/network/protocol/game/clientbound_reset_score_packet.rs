//! Перенос `ClientboundResetScorePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ClientboundResetScorePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundResetScorePacket {
    /// `owner`.
    pub owner: String,
    /// `objectiveName`.
    pub objective_name: Option<String>,
}

impl ClientboundResetScorePacket {
    /// `ClientboundResetScorePacket::new`.
    pub fn new(owner: String, objective_name: Option<String>) -> Self {
        Self { owner, objective_name }
    }
    /// `owner`.
    pub fn owner(&self) -> &str { &self.owner }
    /// `objectiveName`.
    pub fn objective_name(&self) -> Option<&str> { self.objective_name.as_deref() }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundResetScorePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundResetScorePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let owner = buf.read_utf().expect("invalid utf");
                let objective_name = buf.read_nullable(|b| b.read_utf().expect("invalid utf"));
                ClientboundResetScorePacket::new(owner, objective_name)
            },
        )
    }
}

impl Packet for ClientboundResetScorePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_reset_score::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.owner).expect("invalid utf");
        buf.write_nullable(&self.objective_name, |b, s| { b.write_utf(s).expect("invalid utf"); });    }
}
