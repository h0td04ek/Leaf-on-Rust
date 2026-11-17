//! Перенос `ClientboundRegistryDataPacket`.

use crate::core::registry::Registry;
use crate::core::registry_synchronization::PackedRegistryEntry;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{FnCodec, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::resources::resource_key::ResourceKey;
use std::sync::LazyLock;

static REGISTRY_DATA_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(
        PacketFlow::CLIENTBOUND,
        Identifier::with_default_namespace("registry_data").unwrap(),
    )
});

/// `ClientboundRegistryDataPacket`.
#[derive(Clone, Debug, PartialEq)]
pub struct ClientboundRegistryDataPacket {
    /// `registry` — `ResourceKey<? extends Registry<?>>`.
    pub registry: ResourceKey<Registry<crate::nbt::tag::Tag>>,
    /// `entries`.
    pub entries: Vec<PackedRegistryEntry>,
}

impl ClientboundRegistryDataPacket {
    pub fn new(
        registry: ResourceKey<Registry<crate::nbt::tag::Tag>>,
        entries: Vec<PackedRegistryEntry>,
    ) -> Self {
        Self { registry, entries }
    }

    /// `STREAM_CODEC` — registry key (один identifier) + list(entries).
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundRegistryDataPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundRegistryDataPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let registry = read_registry_key(buf);
                let entry_codec = PackedRegistryEntry::stream_codec();
                let list_codec = byte_buf_codecs::list(entry_codec);
                let entries = list_codec.decode(buf);
                ClientboundRegistryDataPacket::new(registry, entries)
            },
        )
    }
}

/// `REGISTRY_KEY_STREAM_CODEC` — `Identifier` ↔ `ResourceKey::createRegistryKey`.
pub fn read_registry_key(
    buf: &mut FriendlyByteBuf,
) -> ResourceKey<Registry<crate::nbt::tag::Tag>> {
    let id = buf.read_identifier().expect("invalid registry key");
    ResourceKey::create_registry_key(id)
}

/// `REGISTRY_KEY_STREAM_CODEC` encode.
pub fn write_registry_key(
    buf: &mut FriendlyByteBuf,
    key: &ResourceKey<Registry<crate::nbt::tag::Tag>>,
) {
    buf.write_identifier(&key.identifier).expect("invalid identifier");
}

impl Packet for ClientboundRegistryDataPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        REGISTRY_DATA_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        write_registry_key(buf, &self.registry);
        let entry_codec = PackedRegistryEntry::stream_codec();
        let list_codec = byte_buf_codecs::list(entry_codec);
        list_codec.encode(buf, &self.entries);
    }
}
