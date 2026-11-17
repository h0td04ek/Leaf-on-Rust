//! Перенос `ClientboundSetScorePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::chat::component::Component;
use crate::network::chat::numbers::number_format::NumberFormat;

/// `ClientboundSetScorePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetScorePacket {
    /// `owner`.
    pub owner: String,
    /// `objectiveName`.
    pub objective_name: String,
    /// `score`.
    pub score: i32,
    /// `display`.
    pub display: Option<Component>,
    /// `numberFormat`.
    pub number_format: Option<NumberFormat>,
}

impl ClientboundSetScorePacket {
    /// `ClientboundSetScorePacket::new`.
    pub fn new(owner: String, objective_name: String, score: i32, display: Option<Component>, number_format: Option<NumberFormat>) -> Self {
        Self { owner, objective_name, score, display, number_format }
    }
    /// `owner`.
    pub fn owner(&self) -> &str { &self.owner }
    /// `objectiveName`.
    pub fn objective_name(&self) -> &str { &self.objective_name }
    /// `score`.
    pub fn score(&self) -> i32 { self.score }
    /// `display`.
    pub fn display(&self) -> Option<&Component> { self.display.as_ref() }
    /// `numberFormat`.
    pub fn number_format(&self) -> Option<&NumberFormat> { self.number_format.as_ref() }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetScorePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetScorePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let owner = buf.read_utf().expect("invalid utf");
                let objective_name = buf.read_utf().expect("invalid utf");
                let score = buf.read_var_int();
                let display = crate::network::chat::component_json::optional_component_stream_codec().decode(buf);
                let number_format = crate::network::chat::numbers::number_format::optional_number_format_stream_codec().decode(buf);
                ClientboundSetScorePacket::new(owner, objective_name, score, display, number_format)            },
        )
    }
}

impl Packet for ClientboundSetScorePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_score::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.owner).expect("invalid utf");
        buf.write_utf(&self.objective_name).expect("invalid utf");
        buf.write_var_int(self.score);
        crate::network::chat::component_json::optional_component_stream_codec().encode(buf, &self.display);
        crate::network::chat::numbers::number_format::optional_number_format_stream_codec().encode(buf, &self.number_format);    }
}
