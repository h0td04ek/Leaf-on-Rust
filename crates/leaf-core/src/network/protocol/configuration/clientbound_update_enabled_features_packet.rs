//! Перенос `ClientboundUpdateEnabledFeaturesPacket`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use std::collections::BTreeSet;
use std::sync::LazyLock;

static UPDATE_ENABLED_FEATURES_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("update_enabled_features").unwrap(),
    )
});

/// `ClientboundUpdateEnabledFeaturesPacket`.
/// Java использует `Set<Identifier>` — у нас `BTreeSet` (детерминированный порядок).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientboundUpdateEnabledFeaturesPacket {
    /// `features`.
    pub features: BTreeSet<Identifier>,
}

impl ClientboundUpdateEnabledFeaturesPacket {
    pub fn new(features: BTreeSet<Identifier>) -> Self {
        Self { features }
    }

    /// `STREAM_CODEC` — `readCollection`/`writeCollection` идентификаторов.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundUpdateEnabledFeaturesPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundUpdateEnabledFeaturesPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let count = buf.read_var_int();
                let mut features = BTreeSet::new();
                for _ in 0..count {
                    features.insert(buf.read_identifier().expect("invalid feature id"));
                }
                ClientboundUpdateEnabledFeaturesPacket::new(features)
            },
        )
    }
}

impl Packet for ClientboundUpdateEnabledFeaturesPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        UPDATE_ENABLED_FEATURES_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_var_int(self.features.len() as i32);
        for id in &self.features {
            buf.write_identifier(id).expect("invalid identifier");
        }
    }
}
