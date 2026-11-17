//! Перенос `net.minecraft.core.component.PatchedDataComponentMap`.
//!
//! `prototype` (DataComponentMap) + `patch` (type -> Optional<value>).
//! copy-on-write семантика сохранена (patch клонируется при записи).
//! Lithium `ChangeSubscriber`-хуки Leaf — не переносятся (нет
//! equipment-tracking в Rust-порте).

use crate::core::component::data_component_map::DataComponentMap;
use crate::core::component::data_component_patch::{ComponentPatchValue, DataComponentPatch};
use crate::core::component::data_component_type::{ComponentValue, DataComponentType};
use crate::core::component::typed_data_component::TypedDataComponent;
use crate::resources::identifier::Identifier;
use std::collections::HashSet;
use std::sync::Arc;

/// `PatchedDataComponentMap`.
#[derive(Clone)]
pub struct PatchedDataComponentMap {
    /// `prototype`.
    prototype: DataComponentMap,
    /// `patch` — `type -> Optional<value>` (порядок вставки).
    patch: Vec<(Identifier, Option<ComponentValue>)>,
    /// `copyOnWrite`.
    copy_on_write: bool,
}

impl PatchedDataComponentMap {
    /// `new PatchedDataComponentMap(prototype)`.
    pub fn new(prototype: DataComponentMap) -> Self {
        Self { prototype, patch: Vec::new(), copy_on_write: true }
    }

    /// `new()` — пустой prototype.
    pub fn empty() -> Self {
        Self::new(DataComponentMap::new())
    }

    /// Приватный конструктор.
    fn with_patch(
        prototype: DataComponentMap,
        patch: Vec<(Identifier, Option<ComponentValue>)>,
        copy_on_write: bool,
    ) -> Self {
        Self { prototype, patch, copy_on_write }
    }

    /// `fromPatch(prototype, patch)`.
    pub fn from_patch(prototype: DataComponentMap, patch: &DataComponentPatch) -> Self {
        // `isPatchSanitized` — все Set-значения отличаются от prototype
        // и Removed-типы присутствуют в prototype.
        let mut sanitized = true;
        for (id, v) in patch.entries() {
            let proto = prototype_get(&prototype, id);
            match v {
                ComponentPatchValue::Set(val) => {
                    if let Some(pv) = proto {
                        if pv.value_eq(val.as_ref()) {
                            sanitized = false;
                            break;
                        }
                    }
                }
                ComponentPatchValue::Removed => {
                    if proto.is_none() {
                        sanitized = false;
                        break;
                    }
                }
            }
        }
        if sanitized {
            Self::with_patch(
                prototype,
                patch
                    .entries()
                    .iter()
                    .map(|(id, v)| match v {
                        ComponentPatchValue::Set(val) => (id.clone(), Some(val.clone_box())),
                        ComponentPatchValue::Removed => (id.clone(), None),
                    })
                    .collect(),
                true,
            )
        } else {
            let mut m = Self::new(prototype);
            m.apply_patch(patch);
            m
        }
    }

    /// `ensureMapOwnership`.
    fn ensure_map_ownership(&mut self) {
        if self.copy_on_write {
            self.patch = self.patch.clone();
            self.copy_on_write = false;
        }
    }

    /// `get(component)` — patch перекрывает prototype.
    pub fn get<T: 'static>(&self, component: &DataComponentType) -> Option<&T> {
        self.get_by_id(&component.id)
    }

    /// `get` по registry-ключу.
    pub fn get_by_id<T: 'static>(&self, component_id: &Identifier) -> Option<&T> {
        if let Some(entry) = self.patch.iter().find(|(id, _)| id == component_id) {
            return entry.1.as_ref().and_then(|v| v.as_any().downcast_ref::<T>());
        }
        self.prototype.get_by_id(component_id)
    }

    /// `has(component)` — есть ли эффективное значение.
    pub fn has(&self, component: &DataComponentType) -> bool {
        self.has_by_id(&component.id)
    }

    /// `has` по registry-ключу.
    pub fn has_by_id(&self, component_id: &Identifier) -> bool {
        if let Some(entry) = self.patch.iter().find(|(id, _)| id == component_id) {
            entry.1.is_some()
        } else {
            self.prototype.entries().iter().any(|(id, _)| id == component_id)
        }
    }

    /// `hasNonDefault(component)`.
    pub fn has_non_default(&self, component: &DataComponentType) -> bool {
        self.patch.iter().any(|(id, _)| *id == component.id)
    }

    /// `set(component, value)` — `None` равносильно `remove`.
    /// Возвращает прежнее эффективное значение (как Java).
    pub fn set<T>(&mut self, component: &DataComponentType, value: Option<T>) -> Option<ComponentValue>
    where
        T: crate::core::component::data_component_type::ComponentValueData + 'static,
    {
        let old = self.effective_get_raw(&component.id);
        match value {
            Some(v) => {
                let boxed: ComponentValue = Box::new(v);
                let same = old
                    .as_ref()
                    .map(|o| o.value_eq(boxed.as_ref()))
                    .unwrap_or(false);
                if same {
                    self.patch_remove_entry(&component.id);
                } else {
                    self.ensure_map_ownership();
                    self.patch_upsert(&component.id, Some(boxed));
                }
            }
            None => {
                self.remove(component);
            }
        }
        old
    }

    /// `set(TypedDataComponent)`.
    pub fn set_typed(&mut self, component: &TypedDataComponent) {
        let id = component.component_type.id.clone();
        let proto = self.prototype_get_raw(&id);
        let new_val = component.value.clone_box();
        let same = proto
            .as_ref()
            .map(|p| p.value_eq(new_val.as_ref()))
            .unwrap_or(false);
        if same {
            self.patch_remove_entry(&id);
        } else {
            self.ensure_map_ownership();
            self.patch_upsert(&id, Some(new_val));
        }
    }

    /// `remove(component)`.
    pub fn remove(&mut self, component: &DataComponentType) {
        let id = component.id.clone();
        let proto = self.prototype_get_raw(&id);
        if proto.is_some() {
            self.ensure_map_ownership();
            self.patch_upsert(&id, None);
        } else {
            self.patch_remove_entry(&id);
        }
    }

    /// `applyPatch(patch)`.
    pub fn apply_patch(&mut self, patch: &DataComponentPatch) {
        for (id, v) in patch.entries() {
            self.apply_patch_entry(id, v);
        }
    }

    fn apply_patch_entry(&mut self, id: &Identifier, value: &ComponentPatchValue) {
        let proto = self.prototype_get_raw(id);
        match value {
            ComponentPatchValue::Set(val) => {
                let same = proto
                    .as_ref()
                    .map(|p| p.value_eq(val.as_ref()))
                    .unwrap_or(false);
                if same {
                    self.patch_remove_entry(id);
                } else {
                    self.ensure_map_ownership();
                    self.patch_upsert(id, Some(val.clone_box()));
                }
            }
            ComponentPatchValue::Removed => {
                if proto.is_some() {
                    self.ensure_map_ownership();
                    self.patch_upsert(id, None);
                } else {
                    self.patch_remove_entry(id);
                }
            }
        }
    }

    /// `restorePatch(patch)`.
    pub fn restore_patch(&mut self, patch: &DataComponentPatch) {
        self.ensure_map_ownership();
        self.patch.clear();
        for (id, v) in patch.entries() {
            match v {
                ComponentPatchValue::Set(val) => {
                    self.patch.push((id.clone(), Some(val.clone_box())));
                }
                ComponentPatchValue::Removed => {
                    self.patch.push((id.clone(), None));
                }
            }
        }
    }

    /// `clearPatch`.
    pub fn clear_patch(&mut self) {
        self.ensure_map_ownership();
        self.patch.clear();
    }

    /// `setAll(map)`.
    pub fn set_all(&mut self, map: &DataComponentMap) {
        for (id, v) in map.entries() {
            self.ensure_map_ownership();
            self.patch_upsert(id, Some(v.clone_box()));
        }
    }

    /// `keySet` — эффективные типы.
    pub fn key_set(&self) -> HashSet<Identifier> {
        let mut set: HashSet<Identifier> = self
            .prototype
            .set_types()
            .into_iter()
            .collect();
        for (id, v) in &self.patch {
            match v {
                Some(_) => {
                    set.insert(id.clone());
                }
                None => {
                    set.remove(id);
                }
            }
        }
        set
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.key_set().len()
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.size() == 0
    }

    /// `asPatch`.
    pub fn as_patch(&self) -> DataComponentPatch {
        let mut builder = DataComponentPatch::builder();
        for (id, v) in &self.patch {
            match v {
                Some(val) => {
                    builder.set_boxed(id.clone(), val.clone_box());
                }
                None => {
                    if let Some(ty) =
                        crate::core::component::data_component_type::data_component_type_by_identifier(id)
                    {
                        builder.remove(&ty);
                    }
                }
            }
        }
        builder.build()
    }

    /// `copy`.
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// `toImmutableMap`.
    pub fn to_immutable_map(&self) -> DataComponentMap {
        let mut map = self.prototype.clone();
        for (id, v) in &self.patch {
            match v {
                Some(val) => map.set_boxed(id, val.clone_box()),
                None => map.remove_by_id(id),
            }
        }
        map
    }

    /// Итерация typed-компонентов.
    pub fn iter(&self) -> Vec<TypedDataComponent> {
        let mut out = Vec::new();
        for id in self.key_set() {
            if let Some(ty) =
                crate::core::component::data_component_type::data_component_type_by_identifier(&id)
            {
                let value = match self.patch.iter().find(|(pid, _)| *pid == id) {
                    Some((_, Some(v))) => Some(v.clone_box()),
                    Some((_, None)) => None,
                    None => self
                        .prototype
                        .entries()
                        .iter()
                        .find(|(pid, _)| *pid == id)
                        .map(|(_, v)| v.clone_box()),
                };
                if let Some(v) = value {
                    out.push(TypedDataComponent::create_unchecked(Arc::new(ty), v));
                }
            }
        }
        out
    }

    /// Доступ к patch для codec'ов.
    pub fn patch_entries(&self) -> &[(Identifier, Option<ComponentValue>)] {
        &self.patch
    }

    /// `prototype`.
    pub fn prototype(&self) -> &DataComponentMap {
        &self.prototype
    }

    /// Эффективное значение без типа — для codec'ов.
    pub fn effective_raw(&self, component_id: &Identifier) -> Option<ComponentValue> {
        self.effective_get_raw(component_id)
    }

    // ---- внутренние helpers -------------------------------------------

    fn prototype_get_raw(&self, id: &Identifier) -> Option<ComponentValue> {
        self.prototype
            .entries()
            .iter()
            .find(|(pid, _)| pid == id)
            .map(|(_, v)| v.clone_box())
    }

    fn effective_get_raw(&self, id: &Identifier) -> Option<ComponentValue> {
        if let Some(entry) = self.patch.iter().find(|(pid, _)| pid == id) {
            entry.1.clone()
        } else {
            self.prototype_get_raw(id)
        }
    }

    fn patch_upsert(&mut self, id: &Identifier, value: Option<ComponentValue>) {
        if let Some(entry) = self.patch.iter_mut().find(|(pid, _)| pid == id) {
            entry.1 = value;
        } else {
            self.patch.push((id.clone(), value));
        }
    }

    fn patch_remove_entry(&mut self, id: &Identifier) {
        if let Some(pos) = self.patch.iter().position(|(pid, _)| pid == id) {
            self.patch.remove(pos);
        }
    }
}

fn prototype_get<'a>(map: &'a DataComponentMap, id: &Identifier) -> Option<&'a ComponentValue> {
    map.entries().iter().find(|(pid, _)| pid == id).map(|(_, v)| v)
}

impl PartialEq for PatchedDataComponentMap {
    fn eq(&self, other: &Self) -> bool {
        if self.prototype != other.prototype || self.patch.len() != other.patch.len() {
            return false;
        }
        self.patch.iter().all(|(id, v)| {
            other
                .patch
                .iter()
                .find(|(oid, _)| oid == id)
                .map(|(_, ov)| match (v, ov) {
                    (Some(a), Some(b)) => a.value_eq(b.as_ref()),
                    (None, None) => true,
                    _ => false,
                })
                .unwrap_or(false)
        })
    }
}

impl std::fmt::Debug for PatchedDataComponentMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PatchedDataComponentMap")
            .field(
                "prototype",
                &self.prototype,
            )
            .field(
                "patch",
                &self
                    .patch
                    .iter()
                    .map(|(id, v)| (id.to_string(), v.as_ref().map(|x| x.value_debug())))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}
