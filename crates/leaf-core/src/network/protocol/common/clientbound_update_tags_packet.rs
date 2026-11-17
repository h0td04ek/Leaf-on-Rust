//! Перенос `ClientboundUpdateTagsPacket` + `TagNetworkSerialization.NetworkPayload`.

use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::network::protocol::packet::{packet_codec, Packet, PacketTypePtr};
use crate::network::protocol::packet_flow::PacketFlow;
use crate::network::protocol::packet_type::PacketType;
use crate::resources::identifier::Identifier;
use crate::core::registry::Registry;
use crate::resources::resource_key::ResourceKey;
use std::collections::HashMap;
use std::sync::LazyLock;

static UPDATE_TAGS_TYPE: LazyLock<PacketTypePtr> = LazyLock::new(|| {
    PacketType::new(PacketFlow::CLIENTBOUND, Identifier::with_default_namespace("update_tags").unwrap())
});

/// `TagNetworkSerialization.NetworkPayload` — теги реестра:
/// `Map<Identifier(tag name), IntList(registry ids)>`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NetworkPayload {
    /// `tags` — tag location → список id элементов реестра.
    pub tags: HashMap<Identifier, Vec<i32>>,
}

impl NetworkPayload {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self::default()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.tags.is_empty()
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.tags.len()
    }

    /// `write(buffer)`.
    pub fn write(&self, buffer: &mut FriendlyByteBuf) {
        buffer.write_map(&self.tags, |b, id| {
            b.write_identifier(id).expect("tag id");
        }, |b, list| {
            b.write_int_id_list(list);
        });
    }

    /// `read(buffer)`.
    pub fn read(buffer: &mut FriendlyByteBuf) -> Self {
        let tags = buffer.read_map(
            HashMap::with_capacity,
            |b| b.read_identifier().expect("tag id"),
            |b| b.read_int_id_list(),
        );
        NetworkPayload { tags }
    }
}

/// `ClientboundUpdateTagsPacket`.
#[derive(Clone, Default, PartialEq)]
pub struct ClientboundUpdateTagsPacket {
    /// `tags` — registry key → payload.
    /// `ResourceKey<Registry<()>>` — реестры типизированы динамически.
    pub tags: HashMap<ResourceKey<Registry<()>>, NetworkPayload>,
}

impl ClientboundUpdateTagsPacket {
    pub fn new(tags: HashMap<ResourceKey<Registry<()>>, NetworkPayload>) -> Self {
        Self { tags }
    }

    /// `getTags`.
    pub fn get_tags(&self) -> &HashMap<ResourceKey<Registry<()>>, NetworkPayload> {
        &self.tags
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, ClientboundUpdateTagsPacket> {
        packet_codec(
            |buf: &mut FriendlyByteBuf, p: &ClientboundUpdateTagsPacket| p.write_packet(buf),
            |buf: &mut FriendlyByteBuf| {
                let tags = buf.read_map(
                    HashMap::with_capacity,
                    |b| -> ResourceKey<Registry<()>> { b.read_registry_key() },
                    |b| NetworkPayload::read(b),
                );
                ClientboundUpdateTagsPacket::new(tags)
            },
        )
    }
}

impl Packet for ClientboundUpdateTagsPacket {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn packet_type(&self) -> PacketTypePtr {
        UPDATE_TAGS_TYPE.clone()
    }
    fn write_packet(&self, buf: &mut FriendlyByteBuf) {
        buf.write_map(&self.tags, |b, key| {
            b.write_resource_key(key).expect("registry key");
        }, |b, payload| {
            payload.write(b);
        });
    }
}
