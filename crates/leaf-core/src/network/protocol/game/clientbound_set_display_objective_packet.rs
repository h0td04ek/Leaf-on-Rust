//! Перенос `ClientboundSetDisplayObjectivePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ClientboundSetDisplayObjectivePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundSetDisplayObjectivePacket {
    /// `slot`.
    pub slot: crate::world::scores::display_slot::DisplaySlot,
    /// `objectiveName`.
    pub objective_name: String,
}

impl ClientboundSetDisplayObjectivePacket {
    /// `ClientboundSetDisplayObjectivePacket::new`.
    pub fn new(slot: crate::world::scores::display_slot::DisplaySlot, objective_name: String) -> Self {
        Self { slot, objective_name }
    }
    /// `getSlot`.
    pub fn get_slot(&self) -> crate::world::scores::display_slot::DisplaySlot { self.slot }
    /// `getObjectiveName` — `""` → None.
    pub fn get_objective_name(&self) -> Option<&str> {
        if self.objective_name.is_empty() { None } else { Some(&self.objective_name) }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundSetDisplayObjectivePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundSetDisplayObjectivePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let slot = crate::world::scores::display_slot::DisplaySlot::by_id(buf.read_var_int());
                let objective_name = buf.read_utf().expect("invalid utf");
                ClientboundSetDisplayObjectivePacket::new(slot, objective_name)
            },
        )
    }
}

impl Packet for ClientboundSetDisplayObjectivePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_set_display_objective::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_by_id(&self.slot, |s| s.id());
        buf.write_utf(&self.objective_name).expect("invalid utf");    }
}
