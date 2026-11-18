//! `HashedPatchMap` — хэшированная карта компонентов для `HashedStack`.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::resources::Identifier;
use crate::core::component::data_component_patch::DataComponentPatch;
use crate::core::component::typed_data_component::TypedDataComponent;

/// `HashedPatchMap` — `{ addedComponents: Map<DataComponentType, Integer>, removedComponents: Set<DataComponentType> }`.
/// Type-erased: `Identifier` вместо `DataComponentType`.
#[derive(Clone, Debug, Default)]
pub struct HashedPatchMap {
    /// `addedComponents` — `Map<DataComponentType, Integer>` → `HashMap<Identifier, i32>`.
    pub added_components: HashMap<Identifier, i32>,
    /// `removedComponents` — `Set<DataComponentType>` → `HashSet<Identifier>`.
    pub removed_components: HashSet<Identifier>,
}

impl HashedPatchMap {
    /// `new`.
    pub fn new(
        added_components: HashMap<Identifier, i32>,
        removed_components: HashSet<Identifier>,
    ) -> Self {
        Self { added_components, removed_components }
    }

    /// `create(patch, hashGenerator)` — из `DataComponentPatch`.
    /// `hashGenerator` — `Function<TypedDataComponent, Integer>`.
    pub fn create(patch: &DataComponentPatch, hash_generator: &dyn Fn(&TypedDataComponent) -> i32) -> Self {
        let mut added = HashMap::new();
        let mut removed = HashSet::new();
        // `patch.split()` — разделение на added/removed
        // Упрощённо: все компоненты patch — added
        for (id, value) in patch.entries() {
            match value {
                crate::core::component::data_component_patch::ComponentPatchValue::Set(_) => {
                    added.insert(id.clone(), hash_generator_id(id));
                },
                crate::core::component::data_component_patch::ComponentPatchValue::Removed => {
                    removed.insert(id.clone());
                },
            }
        }
        Self::new(added, removed)
    }

    /// `matches(patch, hashGenerator)`.
    pub fn matches(&self, patch: &DataComponentPatch, hash_generator: &dyn Fn(&TypedDataComponent) -> i32) -> bool {
        // Упрощённо: сравнение по id
        let mut added_ids = HashSet::new();
        let mut removed_ids = HashSet::new();
        for (id, value) in patch.entries() {
            match value {
                crate::core::component::data_component_patch::ComponentPatchValue::Set(_) => {
                    added_ids.insert(id.clone());
                },
                crate::core::component::data_component_patch::ComponentPatchValue::Removed => {
                    removed_ids.insert(id.clone());
                },
            }
        }
        self.added_components.keys().cloned().collect::<HashSet<_>>() == added_ids
            && self.removed_components == removed_ids
    }

    /// `STREAM_CODEC` — `StreamCodec.composite(map(registry, INT), collection(registry), new)`.
    /// Type-erased: `Identifier` вместо `DataComponentType`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, HashedPatchMap> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, map: &HashedPatchMap| {
                // addedComponents — map(Identifier → VarInt)
                buf.write_var_int(map.added_components.len() as i32);
                for (id, hash) in &map.added_components {
                    buf.write_identifier(id);
                    buf.write_var_int(*hash);
                }
                // removedComponents — collection(Identifier)
                buf.write_var_int(map.removed_components.len() as i32);
                for id in &map.removed_components {
                    buf.write_identifier(id);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let added_len = buf.read_var_int();
                let mut added = HashMap::new();
                for _ in 0..added_len {
                    let id = buf.read_identifier().unwrap_or_else(|_| crate::resources::Identifier::parse("minecraft:unknown").unwrap());
                    let hash = buf.read_var_int();
                    added.insert(id, hash);
                }
                let removed_len = buf.read_var_int();
                let mut removed = HashSet::new();
                for _ in 0..removed_len {
                    removed.insert(buf.read_identifier().unwrap_or_else(|_| crate::resources::Identifier::parse("minecraft:unknown").unwrap()));
                }
                Self::new(added, removed)
            },
        )
    }
}

fn hash_generator_id(id: &Identifier) -> i32 {
    // Упрощённый хэш — `id.toString().hashCode()`
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    id.hash(&mut h);
    h.finish() as i32
}
