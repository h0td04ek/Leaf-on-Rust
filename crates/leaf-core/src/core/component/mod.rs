//! Перенос `net.minecraft.core.component` — data components,
//! патчи и карты компонентов для ItemStack/EntityData.

pub mod data_component_map;
pub mod data_component_patch;
pub mod data_component_type;
pub mod data_components;
pub mod patched_data_component_map;
pub mod typed_data_component;

pub use data_component_map::DataComponentMap;
pub use data_component_patch::{ComponentPatchValue, DataComponentPatch};
pub use data_component_type::{data_component_type_stream_codec, DataComponentType};
pub use patched_data_component_map::PatchedDataComponentMap;
pub use typed_data_component::TypedDataComponent;
pub mod data_component_exact_predicate;
pub mod data_component_getter;
pub mod predicates;
