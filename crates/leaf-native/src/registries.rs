use std::collections::HashMap;

use leaf_core::core::registry_synchronization::PackedRegistryEntry;
use leaf_core::network::protocol::common::clientbound_update_tags_packet::{
    ClientboundUpdateTagsPacket, NetworkPayload,
};
use leaf_core::network::protocol::configuration::clientbound_registry_data_packet::ClientboundRegistryDataPacket;
use leaf_core::resources::{Identifier, ResourceKey};

pub struct Registries {
    pub packets: Vec<ClientboundRegistryDataPacket>,
    pub tags: ClientboundUpdateTagsPacket,
    pub overworld_id: i32,
    pub plains_id: i32,
}

impl Registries {
    pub fn load() -> Self {
        let mut packets = Vec::new();
        let mut tags = HashMap::new();
        let mut groups = include_str!("../data/vanilla-774.txt").split("\n[");
        groups.next();
        for group in groups {
            let (registry, entries) = group.split_once(']').expect("registry manifest header");
            let id = Identifier::minecraft(registry);
            let mut values = Vec::new();
            let mut named = HashMap::new();
            for entry in entries
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
            {
                if let Some(tag) = entry.strip_prefix('@') {
                    let mut fields = tag.split_ascii_whitespace();
                    let name = Identifier::minecraft(fields.next().expect("tag name"));
                    let indices = fields.map(|id| id.parse().expect("tag index")).collect();
                    named.insert(name, indices);
                } else {
                    values.push(PackedRegistryEntry::new(Identifier::minecraft(entry), None));
                }
            }
            if !values.is_empty() {
                packets.push(ClientboundRegistryDataPacket::new(
                    ResourceKey::create_registry_key(id.clone()),
                    values,
                ));
            }
            if !named.is_empty() {
                tags.insert(
                    ResourceKey::create_registry_key(id),
                    NetworkPayload { tags: named },
                );
            }
        }
        let index = |registry: &str, name: &str| {
            packets
                .iter()
                .find(|packet| packet.registry.identifier.get_path() == registry)
                .and_then(|packet| {
                    packet
                        .entries
                        .iter()
                        .position(|entry| entry.id.get_path() == name)
                })
                .expect("required sandbox registry entry") as i32
        };
        Self {
            overworld_id: index("dimension_type", "overworld"),
            plains_id: index("worldgen/biome", "plains"),
            packets,
            tags: ClientboundUpdateTagsPacket::new(tags),
        }
    }
}
