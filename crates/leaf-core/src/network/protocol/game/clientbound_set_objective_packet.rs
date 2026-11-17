//! Перенос `ClientboundSetObjectivePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::chat::numbers::number_format::NumberFormat;
use crate::world::scores::objective_criteria::RenderType;
use crate::network::chat::component_json::{component_stream_codec, optional_component_stream_codec};
use crate::network::chat::numbers::number_format::{number_format_stream_codec, optional_number_format_stream_codec};

/// `ClientboundSetObjectivePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetObjectivePacket {
    /// `objectiveName`.
    pub objective_name: String,
    /// `displayName`.
    pub display_name: crate::network::chat::component::Component,
    /// `renderType`.
    pub render_type: crate::world::scores::objective_criteria::RenderType,
    /// `numberFormat`.
    pub number_format: Option<crate::network::chat::numbers::number_format::NumberFormat>,
    /// `method`.
    pub method: i32,
}

impl ClientboundSetObjectivePacket {
    /// `ClientboundSetObjectivePacket::new`.
    pub fn new(objective_name: String, display_name: crate::network::chat::component::Component, render_type: RenderType, number_format: Option<NumberFormat>, method: i32) -> Self {
        Self { objective_name, display_name, render_type, number_format, method }
    }
    /// `METHOD_ADD`.
    pub const METHOD_ADD: i32 = 0;
    /// `METHOD_REMOVE`.
    pub const METHOD_REMOVE: i32 = 1;
    /// `METHOD_CHANGE`.
    pub const METHOD_CHANGE: i32 = 2;

    /// `getObjectiveName`.
    pub fn get_objective_name(&self) -> &str { &self.objective_name }
    /// `getDisplayName`.
    pub fn get_display_name(&self) -> &crate::network::chat::component::Component { &self.display_name }
    /// `getMethod`.
    pub fn get_method(&self) -> i32 { self.method }
    /// `getRenderType`.
    pub fn get_render_type(&self) -> RenderType { self.render_type }
    /// `getNumberFormat`.
    pub fn get_number_format(&self) -> Option<&crate::network::chat::numbers::number_format::NumberFormat> {
        self.number_format.as_ref()
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetObjectivePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetObjectivePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let objective_name = buf.read_utf().expect("invalid utf");
                let method = buf.read_byte() as i32;
                let (display_name, render_type, number_format) =
                    if method == ClientboundSetObjectivePacket::METHOD_ADD
                        || method == ClientboundSetObjectivePacket::METHOD_CHANGE
                    {
                        (
                            component_stream_codec().decode(buf),
                            RenderType::by_ordinal(buf.read_var_int())
                                .expect("invalid render type"),
                            optional_number_format_stream_codec().decode(buf),
                        )
                    } else {
                        (
                            crate::network::chat::common_components::empty(),
                            RenderType::Integer,
                            None,
                        )
                    };
                ClientboundSetObjectivePacket::new(
                    objective_name,
                    display_name,
                    render_type,
                    number_format,
                    method,
                )
            },
        )
    }
}

impl Packet for ClientboundSetObjectivePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_objective::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_utf(&self.objective_name).expect("invalid utf");
        buf.write_byte(self.method);    }
}
