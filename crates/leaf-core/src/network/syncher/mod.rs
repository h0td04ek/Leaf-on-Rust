//! `net.minecraft.network.syncher` — синхронизация entity data.

pub mod entity_data_accessor;
pub mod entity_data_serializers;
pub mod synched_entity_data;

pub use entity_data_accessor::{EntityDataAccessor, EntityDataSerializer, ForValueTypeSerializer};
pub use entity_data_serializers::EntityDataSerializers;
pub use synched_entity_data::{SynchedEntityData, DataItem, DataValue};
