//! `ClientboundUpdateAdvancementsPacket` — обновление advancements.

use crate::advancements::advancement_holder::AdvancementHolder;
use crate::advancements::advancement_progress::AdvancementProgress;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::game::game_packet_types::GamePacketTypes;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr, PacketListenerObj};
use crate::resources::identifier::Identifier;
use std::collections::{HashMap, HashSet};

/// `ClientboundUpdateAdvancementsPacket`.
pub struct ClientboundUpdateAdvancementsPacket {
    /// `reset`.
    pub reset: bool,
    /// `added`.
    pub added: Vec<AdvancementHolder>,
    /// `removed`.
    pub removed: HashSet<Identifier>,
    /// `progress`.
    pub progress: HashMap<Identifier, AdvancementProgress>,
    /// `showAdvancements`.
    pub show_advancements: bool,
}

impl ClientboundUpdateAdvancementsPacket {
    /// `new(reset, added, removed, progress, showAdvancements)`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        reset: bool,
        added: Vec<AdvancementHolder>,
        removed: HashSet<Identifier>,
        progress: HashMap<Identifier, AdvancementProgress>,
        show_advancements: bool,
    ) -> Self {
        Self { reset, added, removed, progress, show_advancements }
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundUpdateAdvancementsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundUpdateAdvancementsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let reset = buf.read_bool();
                let added = AdvancementHolder::decode_list(buf);
                let removed: HashSet<Identifier> = buf.read_collection(|n| Vec::with_capacity(n), |b| {
                    b.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap_or_else(|_| Identifier::parse("minecraft:unknown").unwrap()))
                }).into_iter().collect();
                let progress = buf.read_map(
                    |n| HashMap::with_capacity(n),
                    |b| b.read_identifier().unwrap_or_else(|_| Identifier::from_namespace_and_path("minecraft", "unknown").unwrap_or_else(|_| Identifier::parse("minecraft:unknown").unwrap())),
                    |b| AdvancementProgress::from_network(b),
                );
                let show_advancements = buf.read_bool();
                Self::new(reset, added, removed, progress, show_advancements)
            },
        )
    }
}

impl Packet for ClientboundUpdateAdvancementsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn packet_type(&self) -> PacketTypePtr {
        GamePacketTypes::clientbound_update_advancements::<Self>().erase()
    }

    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_bool(self.reset);
        AdvancementHolder::encode_list(&self.added, buf);
        buf.write_collection(&self.removed.iter().collect::<Vec<_>>(), |b, id| {
            b.write_identifier(id);
        });
        buf.write_map(&self.progress, |b, k| {
            b.write_identifier(k);
        }, |b, v| {
            v.serialize_to_network(b);
        });
        buf.write_bool(self.show_advancements);
    }

    fn handle(&self, _handler: &mut dyn PacketListenerObj) {}
}
