//! `BlockEntity`-интерфейс — `collectComponents`/`saveWithFullMetadata`
//! для `BlockPredicate.matchesComponents`.

use crate::core::component::data_component_getter::DataComponentGetter;
use crate::core::registry_access::RegistryAccess;
use crate::nbt::compound::CompoundTag;
use crate::nbt::tag::Tag;
use std::sync::Arc;

/// Доступ к block entity для предикатов (`DataComponentGetter` +
/// NBT-сохранение).
pub trait BlockEntityAccess: Send + Sync {
    /// `collectComponents` — эффективные компоненты BE.
    fn components(&self) -> &dyn DataComponentGetter;

    /// `saveWithFullMetadata(registryAccess)` — полный NBT
    /// (с id и координатами).
    fn save_full_metadata(&self, registry_access: &RegistryAccess) -> CompoundTag;

    /// Обрабатывает событие блока. Возвращает true, если событие обработано.
    fn trigger_event(&self, _event_id: i32, _event_param: i32) -> bool {
        false
    }

    /// Тип BE (`BlockEntityType`).
    fn block_entity_type_id(&self) -> &crate::resources::identifier::Identifier;
}

/// Type-erased ссылка на block entity.
pub type BlockEntityRef = Arc<dyn BlockEntityAccess>;
