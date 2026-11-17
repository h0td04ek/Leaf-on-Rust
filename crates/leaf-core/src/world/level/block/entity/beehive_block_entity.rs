//! Перенос `net.minecraft.world.level.block.entity.BeehiveBlockEntity.Occupant`
//! — запись занятого улья (для `bees` компонента).

use crate::core::holder::Holder;
use crate::entity::entity_type::EntityType;
use crate::nbt::compound::CompoundTag;
use crate::world::item::component::typed_entity_data::TypedEntityData;
use crate::network::codec::byte_buf_codecs;
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `BeehiveBlockEntity.Occupant` — (entityData, ticksInHive,
/// minTicksInHive).
#[derive(Clone, Debug, PartialEq)]
pub struct BeehiveOccupant {
    /// `entityData`.
    pub entity_data: TypedEntityData<Holder<EntityType>>,
    /// `ticksInHive`.
    pub ticks_in_hive: i32,
    /// `minTicksInHive`.
    pub min_ticks_in_hive: i32,
}

impl BeehiveOccupant {
    /// `new`.
    pub fn new(entity_data: TypedEntityData<Holder<EntityType>>, ticks_in_hive: i32, min_ticks_in_hive: i32) -> Self {
        Self { entity_data, ticks_in_hive, min_ticks_in_hive }
    }

    /// `Occupant.STREAM_CODEC` — TypedEntityData(EntityType) +
    /// VAR_INT + VAR_INT.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, BeehiveOccupant> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &BeehiveOccupant| {
                TypedEntityData::stream_codec(EntityType::stream_codec()).encode(buf, &v.entity_data);
                buf.write_var_int(v.ticks_in_hive);
                buf.write_var_int(v.min_ticks_in_hive);
            },
            |buf: &mut FriendlyByteBuf| {
                let entity_data = TypedEntityData::stream_codec(EntityType::stream_codec()).decode(buf);
                let ticks_in_hive = buf.read_var_int();
                let min_ticks_in_hive = buf.read_var_int();
                BeehiveOccupant::new(entity_data, ticks_in_hive, min_ticks_in_hive)
            },
        )
    }
}
