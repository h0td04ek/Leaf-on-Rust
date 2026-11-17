//! Перенос `net.minecraft.world.level.gameevent.PositionSource`,
//! `BlockPositionSource`, `EntityPositionSource`.

use super::position_source_type::PositionSourceTypeKind;
use crate::core::block_pos::BlockPos;
use crate::network::codec::stream_codec::StreamCodec;
use crate::network::friendly_byte_buf::FriendlyByteBuf;
use crate::util::uuid::Uuid;
use crate::world::phys::vec3::Vec3;

/// `EntityPositionSource.entityOrUuidOrId` — `Either<Entity, Either<UUID, Integer>>`.
/// Ссылка на сущность на проводе — только entityId (VAR_INT) + yOffset (FLOAT);
/// uuid-форма существует для NBT/кода, но не сериализуется в stream.
#[derive(Clone, Debug, PartialEq)]
pub enum EntitySource {
    /// `Either.left(Entity)` — ссылка на объект сущности (рантайм).
    Entity(i32),
    /// `Either.right(Either.left(UUID))` — uuid (NBT-путь).
    Uuid(Uuid),
    /// `Either.right(Either.right(Integer))` — entity id.
    EntityId(i32),
}

/// `EntityPositionSource` — позиция сущности + yOffset.
#[derive(Clone, Debug, PartialEq)]
pub struct EntityPositionSource {
    /// `entityOrUuidOrId`.
    pub source: EntitySource,
    /// `yOffset`.
    pub y_offset: f32,
}

impl EntityPositionSource {
    /// Конструктор от entity id (stream-путь).
    pub fn new(source: EntitySource, y_offset: f32) -> Self {
        Self { source, y_offset }
    }

    /// `getId` — entity id для stream-кодека.
    fn get_id(&self) -> i32 {
        match self.source {
            EntitySource::Entity(id) | EntitySource::EntityId(id) => id,
            EntitySource::Uuid(_) => panic!("Unable to get entityId from uuid"),
        }
    }

    /// `getUuid` — uuid (только для NBT-пути).
    pub fn get_uuid(&self) -> Option<Uuid> {
        match self.source {
            EntitySource::Uuid(u) => Some(u),
            _ => None,
        }
    }

    /// `STREAM_CODEC` — VAR_INT entityId + FLOAT yOffset.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, EntityPositionSource> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &EntityPositionSource| {
                buf.write_var_int(v.get_id());
                buf.write_float(v.y_offset);
            },
            |buf: &mut FriendlyByteBuf| {
                let id = buf.read_var_int();
                let y = buf.read_float();
                EntityPositionSource::new(EntitySource::EntityId(id), y)
            },
        )
    }
}

/// `PositionSource` — источник позиции (block или entity).
#[derive(Clone, Debug, PartialEq)]
pub enum PositionSource {
    /// `BlockPositionSource`.
    Block(BlockPos),
    /// `EntityPositionSource`.
    Entity(EntityPositionSource),
}

impl PositionSource {
    /// `getType`.
    pub fn get_type(&self) -> PositionSourceTypeKind {
        match self {
            PositionSource::Block(_) => PositionSourceTypeKind::Block,
            PositionSource::Entity(_) => PositionSourceTypeKind::Entity,
        }
    }

    /// `PositionSource.STREAM_CODEC` — registry(POSITION_SOURCE_TYPE)
    /// .dispatch(type, type.streamCodec): VarInt type-id + payload.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, PositionSource> + Send + Sync + 'static {
        crate::network::codec::stream_codec::sync_fn(
            |buf: &mut FriendlyByteBuf, v: &PositionSource| match v {
                PositionSource::Block(pos) => {
                    buf.write_var_int(PositionSourceTypeKind::Block.id());
                    buf.write_block_pos(pos);
                }
                PositionSource::Entity(e) => {
                    buf.write_var_int(PositionSourceTypeKind::Entity.id());
                    EntityPositionSource::stream_codec().encode(buf, e);
                }
            },
            |buf: &mut FriendlyByteBuf| {
                let type_id = buf.read_var_int();
                match PositionSourceTypeKind::by_id(type_id) {
                    PositionSourceTypeKind::Block => {
                        PositionSource::Block(buf.read_block_pos())
                    }
                    PositionSourceTypeKind::Entity => {
                        PositionSource::Entity(EntityPositionSource::stream_codec().decode(buf))
                    }
                }
            },
        )
    }

    /// `getPosition(level)` — для Block: центр блока; для Entity: позиция
    /// сущности + yOffset (резолв через level/entity store — позже).
    pub fn position_hint(&self) -> Vec3 {
        match self {
            PositionSource::Block(pos) => Vec3::new(
                pos.x as f64 + 0.5,
                pos.y as f64 + 0.5,
                pos.z as f64 + 0.5,
            ),
            PositionSource::Entity(e) => Vec3::new(0.0, e.y_offset as f64, 0.0),
        }
    }
}
