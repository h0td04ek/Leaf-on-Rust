//! Перенос `ClientboundSelectAdvancementsTabPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::resources::identifier::Identifier;

/// `ClientboundSelectAdvancementsTabPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSelectAdvancementsTabPacket {
    /// `tab`.
    pub tab: Option<crate::resources::identifier::Identifier>,
}

impl ClientboundSelectAdvancementsTabPacket {
    /// `ClientboundSelectAdvancementsTabPacket::new`.
    pub fn new(tab: Option<crate::resources::identifier::Identifier>) -> Self {
        Self { tab }
    }
    /// `getTab`.
    pub fn get_tab(&self) -> Option<&crate::resources::identifier::Identifier> {
        self.tab.as_ref()
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSelectAdvancementsTabPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSelectAdvancementsTabPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let tab = buf.read_nullable(|b| b.read_identifier().expect("invalid identifier"));
                ClientboundSelectAdvancementsTabPacket::new(tab)
            },
        )
    }
}

impl Packet for ClientboundSelectAdvancementsTabPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_select_advancements_tab::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_nullable(&self.tab, |b, id| { b.write_identifier(id).expect("invalid identifier"); });    }
}
