//! Перенос `net.minecraft.world.entity.Leashable.LeashData`.

use crate::core::block_pos::BlockPos;
use crate::util::either::Either;
use crate::util::uuid::Uuid;

/// `LeashData` — данные поводка моба.
#[derive(Clone, Debug)]
pub struct LeashData {
    /// `delayedLeashHolderId`.
    pub delayed_leash_holder_id: i32,
    /// `leashHolder` — id сущности-держателя (в Java — ссылка
    /// на Entity; здесь entity id, резолв через уровень).
    pub leash_holder_id: Option<i32>,
    /// `delayedLeashInfo` — `Either<UUID, BlockPos>`.
    pub delayed_leash_info: Option<Either<Uuid, BlockPos>>,
    /// `angularMomentum`.
    pub angular_momentum: f64,
}

impl LeashData {
    /// `new LeashData(Entity)`.
    pub fn new_leash_holder(entity_id: i32) -> Self {
        Self {
            delayed_leash_holder_id: -1,
            leash_holder_id: Some(entity_id),
            delayed_leash_info: None,
            angular_momentum: 0.0,
        }
    }

    /// `new LeashData(int delayedLeashHolderId)`.
    pub fn new_delayed(id: i32) -> Self {
        Self {
            delayed_leash_holder_id: id,
            leash_holder_id: None,
            delayed_leash_info: None,
            angular_momentum: 0.0,
        }
    }

    /// `new LeashData(Either<UUID, BlockPos>)`.
    pub fn new_delayed_info(info: Either<Uuid, BlockPos>) -> Self {
        Self {
            delayed_leash_holder_id: -1,
            leash_holder_id: None,
            delayed_leash_info: Some(info),
            angular_momentum: 0.0,
        }
    }
}
