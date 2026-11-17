//! `net.minecraft.core.component.DataComponentGetter` — доступ к
//! значениям компонентов (DataComponentMap/ItemStack/Entity).
//!
//! Object-safe: все методы non-generic, значения — type-erased
//! `ComponentValue` (клон boxed `Any`). Типизированный доступ —
//! через `ComponentValue::as_any().downcast_ref::<T>()`.

use crate::core::component::data_component_type::{ComponentValue, DataComponentType};
use crate::resources::identifier::Identifier;

/// `DataComponentGetter`.
pub trait DataComponentGetter {
    /// `get(component)` — эффективное type-erased значение.
    fn get_raw(&self, component: &DataComponentType) -> Option<ComponentValue>;

    /// `get(component)` типизированно — `T` по `TypeId` значения.
    /// `None` если тип не совпал.
    fn get_typed<T: 'static>(&self, component: &DataComponentType) -> Option<T>
    where
        T: Clone + Sized,
        Self: Sized,
    {
        self.get_raw(component)
            .and_then(|v| v.as_any().downcast_ref::<T>().cloned())
    }

    /// `getOrDefault(component, default)`.
    fn get_or_default<T: 'static + Clone>(&self, component: &DataComponentType, default: T) -> T
    where
        Self: Sized,
    {
        self.get_typed(component).unwrap_or(default)
    }

    /// `has(component)` — наличие эффективного значения.
    fn has_component(&self, component: &DataComponentType) -> bool;

    /// Итератор `(componentId, Option<value>)` эффективных
    /// компонентов (value=None для удалённых патчем).
    fn iter_raw(&self) -> Vec<(Identifier, Option<ComponentValue>)>;
}

impl DataComponentGetter for crate::core::component::patched_data_component_map::PatchedDataComponentMap {
    fn get_raw(&self, component: &DataComponentType) -> Option<ComponentValue> {
        crate::core::component::patched_data_component_map::PatchedDataComponentMap::effective_raw(self, &component.id)
    }

    fn has_component(&self, component: &DataComponentType) -> bool {
        crate::core::component::patched_data_component_map::PatchedDataComponentMap::has(self, component)
    }

    fn iter_raw(&self) -> Vec<(Identifier, Option<ComponentValue>)> {
        self.iter()
            .into_iter()
            .map(|t| (t.component_type.id.clone(), Some(t.value)))
            .collect()
    }
}
