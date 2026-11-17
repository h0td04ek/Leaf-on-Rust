//! Перенос `ClientboundPlayerInfoRemovePacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
/// `ClientboundPlayerInfoRemovePacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundPlayerInfoRemovePacket {
    /// `profileIds`.
    pub profile_ids: Vec<crate::util::uuid::Uuid>,
}

impl ClientboundPlayerInfoRemovePacket {
    /// `ClientboundPlayerInfoRemovePacket::new`.
    pub fn new(profile_ids: Vec<crate::util::uuid::Uuid>) -> Self {
        Self { profile_ids }
    }
    /// `profileIds`.
    pub fn profile_ids(&self) -> &[crate::util::uuid::Uuid] {
        &self.profile_ids
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundPlayerInfoRemovePacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundPlayerInfoRemovePacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let profile_ids = buf.read_list(|b| b.read_uuid());
                ClientboundPlayerInfoRemovePacket::new(profile_ids)
            },
        )
    }
}

impl Packet for ClientboundPlayerInfoRemovePacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_player_info_remove::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_collection(&self.profile_ids, |b, u| { b.write_uuid(u); });    }
}
