//! Перенос `net.minecraft.core.component.DataComponentPatch` —
//! diff поверх `DataComponentMap` (установленные/удалённые типы).
//!
//! Wire-формат (`STREAM_CODEC`/`DELIMITED_STREAM_CODEC`):
//! VarInt count_set, VarInt count_removed; затем для каждого
//! установленного — `DataComponentType` + typed value; для каждого
//! удалённого — только `DataComponentType`.

use crate::core::component::data_component_map::DataComponentMap;
use crate::core::component::data_component_type::{
    data_component_type_arc_by_id, data_component_type_id_by_identifier,
    data_component_type_stream_codec, ComponentValue, DataComponentType,
};
use crate::nbt::{CompoundTag, Tag};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::{CodecError, FriendlyByteBuf};
use crate::resources::identifier::Identifier;
use std::collections::HashSet;
use std::sync::Arc;

/// Значение в patch: `Set` (новое значение) или `Removed`.
#[derive(Clone)]
pub enum ComponentPatchValue {
    /// `Optional.of(value)`.
    Set(ComponentValue),
    /// `Optional.empty()` — удаление компонента.
    Removed,
}

impl std::fmt::Debug for ComponentPatchValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentPatchValue::Set(v) => write!(f, "Set({})", v.value_debug()),
            ComponentPatchValue::Removed => write!(f, "Removed"),
        }
    }
}

impl PartialEq for ComponentPatchValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ComponentPatchValue::Set(a), ComponentPatchValue::Set(b)) => a.value_eq(b.as_ref()),
            (ComponentPatchValue::Removed, ComponentPatchValue::Removed) => true,
            _ => false,
        }
    }
}

/// `DataComponentPatch`.
#[derive(Clone, Default)]
pub struct DataComponentPatch {
    /// `map` — `DataComponentType -> Optional<value>` (порядок вставки).
    entries: Vec<(Identifier, ComponentPatchValue)>,
}

/// Разрешение типа по registry-ключу → Arc<DataComponentType>.
fn type_arc(id: &Identifier) -> Result<Arc<DataComponentType>, CodecError> {
    let wire_id = data_component_type_id_by_identifier(id)
        .ok_or_else(|| CodecError(format!("unregistered component type: {id}")))?;
    data_component_type_arc_by_id(wire_id)
        .ok_or_else(|| CodecError(format!("unregistered component type: {id}")))
}

impl DataComponentPatch {
    /// `EMPTY`.
    pub fn empty() -> Self {
        Self { entries: Vec::new() }
    }

    /// `builder()`.
    pub fn builder() -> DataComponentPatchBuilder {
        DataComponentPatchBuilder { entries: Vec::new() }
    }

    /// `get(component)` — `Optional<? extends T>` как Option<&T>.
    pub fn get<T: 'static>(&self, component: &DataComponentType) -> Option<&T> {
        self.entries
            .iter()
            .find(|(id, _)| *id == component.id)
            .and_then(|(_, v)| match v {
                ComponentPatchValue::Set(val) => val.as_any().downcast_ref::<T>(),
                ComponentPatchValue::Removed => None,
            })
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.entries.len()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `entrySet`.
    pub fn entries(&self) -> &[(Identifier, ComponentPatchValue)] {
        &self.entries
    }

    /// `forget(predicate)`.
    pub fn forget(&mut self, predicate: impl Fn(&DataComponentType) -> bool) {
        self.entries.retain(|(id, _)| {
            crate::core::component::data_component_type::data_component_type_by_identifier(id)
                .map(|t| !predicate(&t))
                .unwrap_or(true)
        });
    }

    /// `split()` — (added map, removed types).
    pub fn split(&self) -> (DataComponentMap, HashSet<Identifier>) {
        let mut added = DataComponentMap::new();
        let mut removed = HashSet::new();
        for (id, value) in &self.entries {
            match value {
                ComponentPatchValue::Set(v) => {
                    added.set_boxed(id, v.clone_box());
                }
                ComponentPatchValue::Removed => {
                    removed.insert(id.clone());
                }
            }
        }
        (added, removed)
    }

    /// Persistent-формат Java. Удаление записывается как "!namespace:key": {}.
    pub fn to_nbt(&self) -> Result<CompoundTag, String> {
        let mut tag = CompoundTag::new();
        for (id, value) in &self.entries {
            let component = type_arc(id).map_err(|error| error.0)?;
            if component.transient {
                continue;
            }

            match value {
                ComponentPatchValue::Removed => {
                    tag.put(format!("!{id}"), Tag::Compound(CompoundTag::new()));
                }
                ComponentPatchValue::Set(value) => {
                    let codec = component.nbt_codec.as_ref()
                        .ok_or_else(|| format!("component '{id}' has no persistent codec"))?;
                    let data = codec.encode(value.as_any()).map_err(|error| format!("{id}: {error}"))?;
                    tag.put(id.to_string(), data);
                }
            }
        }
        Ok(tag)
    }

    pub fn from_nbt(tag: &CompoundTag) -> Result<Self, String> {
        let mut builder = Self::builder();
        let mut seen = HashSet::new();
        for (key, value) in tag.iter() {
            let removed = key.strip_prefix('!');
            let id = Identifier::parse(removed.unwrap_or(key))
                .map_err(|error| format!("invalid component key '{key}': {error}"))?;
            let component = type_arc(&id).map_err(|error| error.0)?;
            if component.transient || !seen.insert(id.clone()) {
                return Err(format!("transient or duplicate component '{id}'"));
            }

            if removed.is_some() {
                if value.as_compound().is_none_or(|tag| !tag.is_empty()) {
                    return Err(format!("removed component '{id}' requires an empty compound"));
                }
                builder.remove(&component);
            } else {
                let codec = component.nbt_codec.as_ref()
                    .ok_or_else(|| format!("component '{id}' has no persistent codec"))?;
                let data = codec.decode(value).map_err(|error| format!("{id}: {error}"))?;
                builder.set_boxed(id, data);
            }
        }
        Ok(builder.build())
    }

    fn encode_inner(
        &self,
        buf: &mut FriendlyByteBuf,
        delimited: bool,
    ) -> Result<(), CodecError> {
        if self.is_empty() {
            buf.write_var_int(0);
            buf.write_var_int(0);
            return Ok(());
        }
        let mut set_count = 0i32;
        let mut removed_count = 0i32;
        for (_, v) in &self.entries {
            match v {
                ComponentPatchValue::Set(_) => set_count += 1,
                ComponentPatchValue::Removed => removed_count += 1,
            }
        }
        buf.write_var_int(set_count);
        buf.write_var_int(removed_count);
        // Java: сначала все Set-записи, затем все Removed.
        for (id, value) in &self.entries {
            if let ComponentPatchValue::Set(v) = value {
                let ty = type_arc(id)?;
                data_component_type_stream_codec().encode(buf, &ty);
                if delimited {
                    let codec = ty.stream_codec.as_ref().ok_or_else(|| {
                        CodecError(format!("component {} has no stream codec", ty.id))
                    })?;
                    let mut tmp = FriendlyByteBuf::new();
                    codec.encode(&mut tmp, v.as_any());
                    let bytes = tmp.readable_bytes_vec();
                    buf.write_var_int(bytes.len() as i32);
                    buf.write_bytes(&bytes);
                } else {
                    ty.stream_codec
                        .as_ref()
                        .ok_or_else(|| {
                            CodecError(format!("component {} has no stream codec", ty.id))
                        })?
                        .encode(buf, v.as_any());
                }
            }
        }
        for (id, value) in &self.entries {
            if matches!(value, ComponentPatchValue::Removed) {
                let ty = type_arc(id)?;
                data_component_type_stream_codec().encode(buf, &ty);
            }
        }
        Ok(())
    }

    fn decode_inner(
        buf: &mut FriendlyByteBuf,
        delimited: bool,
    ) -> Result<Self, CodecError> {
        let set_count = buf.read_var_int();
        let removed_count = buf.read_var_int();
        if set_count == 0 && removed_count == 0 {
            return Ok(Self::empty());
        }
        let total = set_count.saturating_add(removed_count);
        let mut entries = Vec::with_capacity(total.max(0) as usize);
        for _ in 0..set_count {
            let ty = data_component_type_stream_codec().decode(buf);
            let codec = ty.stream_codec.as_ref().ok_or_else(|| {
                CodecError(format!("component {} has no stream codec", ty.id))
            })?;
            let value = if delimited {
                let len = buf.read_var_int();
                let bytes = buf.read_bytes_vec(len.max(0) as usize);
                let mut sub = FriendlyByteBuf::from_bytes(&bytes);
                codec.decode(&mut sub)
            } else {
                codec.decode(buf)
            };
            entries.push((ty.id.clone(), ComponentPatchValue::Set(value)));
        }
        for _ in 0..removed_count {
            let ty = data_component_type_stream_codec().decode(buf);
            entries.push((ty.id.clone(), ComponentPatchValue::Removed));
        }
        Ok(Self { entries })
    }

    /// `STREAM_CODEC`.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, DataComponentPatch> {
        sync_fn(
            |buf: &mut FriendlyByteBuf, p: &DataComponentPatch| {
                p.encode_inner(buf, false).expect("component patch encode");
            },
            |buf: &mut FriendlyByteBuf| {
                DataComponentPatch::decode_inner(buf, false).expect("component patch decode")
            },
        )
    }

    /// `DELIMITED_STREAM_CODEC` — каждое value обёрнуто
    /// `registryFriendlyLengthPrefixed(Integer.MAX_VALUE)` (VarInt len).
    pub fn delimited_stream_codec() -> impl StreamCodec<FriendlyByteBuf, DataComponentPatch> {
        sync_fn(
            |buf: &mut FriendlyByteBuf, p: &DataComponentPatch| {
                p.encode_inner(buf, true).expect("component patch encode");
            },
            |buf: &mut FriendlyByteBuf| {
                DataComponentPatch::decode_inner(buf, true).expect("component patch decode")
            },
        )
    }
}

impl std::fmt::Debug for DataComponentPatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataComponentPatch")
            .field("entries", &self.entries)
            .finish()
    }
}

impl PartialEq for DataComponentPatch {
    fn eq(&self, other: &Self) -> bool {
        if self.entries.len() != other.entries.len() {
            return false;
        }
        self.entries.iter().all(|(id, v)| {
            other
                .entries
                .iter()
                .find(|(oid, _)| oid == id)
                .map(|(_, ov)| v == ov)
                .unwrap_or(false)
        })
    }
}

/// `DataComponentPatch.Builder`.
#[derive(Clone, Default)]
pub struct DataComponentPatchBuilder {
    entries: Vec<(Identifier, ComponentPatchValue)>,
}

impl DataComponentPatchBuilder {
    /// `copy(orig)` — CraftBukkit.
    pub fn copy(&mut self, orig: &DataComponentPatch) {
        self.entries = orig.entries.clone();
    }

    /// `clear(type)` — CraftBukkit.
    pub fn clear(&mut self, component: &DataComponentType) {
        self.entries.retain(|(id, _)| *id != component.id);
    }

    /// `isSet(type)` — CraftBukkit.
    pub fn is_set(&self, component: &DataComponentType) -> bool {
        self.entries.iter().any(|(id, _)| *id == component.id)
    }

    /// `isEmpty` — CraftBukkit.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// `set_boxed` — установить значение по registry-ключу типа
    /// (аналог `setUnchecked`, используется `asPatch`).
    pub fn set_boxed(&mut self, component_id: Identifier, value: ComponentValue) -> &mut Self {
        self.entries.retain(|(id, _)| *id != component_id);
        self.entries
            .push((component_id, ComponentPatchValue::Set(value)));
        self
    }

    /// `set(component, value)`.
    pub fn set<T>(&mut self, component: &DataComponentType, value: T) -> &mut Self
    where
        T: crate::core::component::data_component_type::ComponentValueData + Clone + 'static,
    {
        self.entries.retain(|(id, _)| *id != component.id);
        self.entries.push((
            component.id.clone(),
            ComponentPatchValue::Set(Box::new(value)),
        ));
        self
    }

    /// `remove(component)`.
    pub fn remove(&mut self, component: &DataComponentType) -> &mut Self {
        self.entries.retain(|(id, _)| *id != component.id);
        self.entries
            .push((component.id.clone(), ComponentPatchValue::Removed));
        self
    }

    /// `build`.
    pub fn build(self) -> DataComponentPatch {
        if self.entries.is_empty() {
            DataComponentPatch::empty()
        } else {
            DataComponentPatch { entries: self.entries }
        }
    }
}
