//! `ClientboundMerchantOffersPacket` — предложения трейда.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{Packet, PacketTypePtr};
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::world::item::trading::merchant_offers::MerchantOffers;

/// `ClientboundMerchantOffersPacket`.
#[derive(Clone, Debug)]
pub struct ClientboundMerchantOffersPacket {
    /// `containerId`.
    pub container_id: i32,
    /// `offers`.
    pub offers: MerchantOffers,
    /// `villagerLevel`.
    pub villager_level: i32,
    /// `villagerXp`.
    pub villager_xp: i32,
    /// `showProgress`.
    pub show_progress: bool,
    /// `canRestock`.
    pub can_restock: bool,
}

impl ClientboundMerchantOffersPacket {
    /// `new`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        container_id: i32,
        offers: MerchantOffers,
        villager_level: i32,
        villager_xp: i32,
        show_progress: bool,
        can_restock: bool,
    ) -> Self {
        Self { container_id, offers, villager_level, villager_xp, show_progress, can_restock }
    }

    /// `STREAM_CODEC` — `Packet.codec(write, new)`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundMerchantOffersPacket> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, pkt: &ClientboundMerchantOffersPacket| {
                buf.write_container_id(pkt.container_id);
                MerchantOffers::stream_codec().encode(buf, &pkt.offers);
                buf.write_var_int(pkt.villager_level);
                buf.write_var_int(pkt.villager_xp);
                buf.write_bool(pkt.show_progress);
                buf.write_bool(pkt.can_restock);
            },
            |buf: &mut FriendlyByteBuf| {
                let container_id = buf.read_container_id();
                let offers = MerchantOffers::stream_codec().decode(buf);
                let villager_level = buf.read_var_int();
                let villager_xp = buf.read_var_int();
                let show_progress = buf.read_bool();
                let can_restock = buf.read_bool();
                Self::new(container_id, offers, villager_level, villager_xp, show_progress, can_restock)
            },
        )
    }
}

impl Packet for ClientboundMerchantOffersPacket {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_merchant_offers::<Self>().erase()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_container_id(self.container_id);
        MerchantOffers::stream_codec().encode(buf, &self.offers);
        buf.write_var_int(self.villager_level);
        buf.write_var_int(self.villager_xp);
        buf.write_bool(self.show_progress);
        buf.write_bool(self.can_restock);
    }
}
