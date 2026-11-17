//! Перенос `net.minecraft.world.entity.EquipmentSlot`.

use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `EquipmentSlot.Type` — группа слотов.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlotType {
    /// `HAND`.
    Hand,
    /// `HUMANOID_ARMOR`.
    HumanoidArmor,
    /// `ANIMAL_ARMOR`.
    AnimalArmor,
    /// `SADDLE`.
    Saddle,
}

/// `EquipmentSlot` — слот экипировки (id 0-7 = wire id).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlot {
    /// `MAINHAND`.
    Mainhand,
    /// `OFFHAND`.
    Offhand,
    /// `FEET`.
    Feet,
    /// `LEGS`.
    Legs,
    /// `CHEST`.
    Chest,
    /// `HEAD`.
    Head,
    /// `BODY`.
    Body,
    /// `SADDLE`.
    Saddle,
}

impl EquipmentSlot {
    /// `VALUES_ARRAY` — порядок = id.
    pub const VALUES: [EquipmentSlot; 8] = [
        EquipmentSlot::Mainhand,
        EquipmentSlot::Offhand,
        EquipmentSlot::Feet,
        EquipmentSlot::Legs,
        EquipmentSlot::Chest,
        EquipmentSlot::Head,
        EquipmentSlot::Body,
        EquipmentSlot::Saddle,
    ];

    /// `ARMOR_SLOTS`.
    pub const ARMOR_SLOTS: [EquipmentSlot; 4] = [
        EquipmentSlot::Head,
        EquipmentSlot::Chest,
        EquipmentSlot::Legs,
        EquipmentSlot::Feet,
    ];

    /// `NO_COUNT_LIMIT`.
    pub const NO_COUNT_LIMIT: i32 = 0;

    /// `type` поля Java-конструктора.
    pub fn get_type(&self) -> EquipmentSlotType {
        match self {
            EquipmentSlot::Mainhand | EquipmentSlot::Offhand => EquipmentSlotType::Hand,
            EquipmentSlot::Feet | EquipmentSlot::Legs | EquipmentSlot::Chest | EquipmentSlot::Head => {
                EquipmentSlotType::HumanoidArmor
            }
            EquipmentSlot::Body => EquipmentSlotType::AnimalArmor,
            EquipmentSlot::Saddle => EquipmentSlotType::Saddle,
        }
    }

    /// `index` — индекс внутри типа.
    pub fn index(&self) -> i32 {
        match self {
            EquipmentSlot::Mainhand => 0,
            EquipmentSlot::Offhand => 1,
            EquipmentSlot::Feet => 0,
            EquipmentSlot::Legs => 1,
            EquipmentSlot::Chest => 2,
            EquipmentSlot::Head => 3,
            EquipmentSlot::Body => 0,
            EquipmentSlot::Saddle => 0,
        }
    }

    /// `countLimit`.
    pub fn count_limit(&self) -> i32 {
        match self {
            EquipmentSlot::Mainhand | EquipmentSlot::Offhand => EquipmentSlot::NO_COUNT_LIMIT,
            _ => 1,
        }
    }

    /// `getId` — wire id.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `getName`.
    pub fn get_name(&self) -> &'static str {
        match self {
            EquipmentSlot::Mainhand => "mainhand",
            EquipmentSlot::Offhand => "offhand",
            EquipmentSlot::Feet => "feet",
            EquipmentSlot::Legs => "legs",
            EquipmentSlot::Chest => "chest",
            EquipmentSlot::Head => "head",
            EquipmentSlot::Body => "body",
            EquipmentSlot::Saddle => "saddle",
        }
    }

    /// `isArmor`.
    pub fn is_armor(&self) -> bool {
        matches!(self, EquipmentSlot::Head | EquipmentSlot::Chest | EquipmentSlot::Legs | EquipmentSlot::Feet)
    }

    /// `BY_ID` — вне диапазона → index 0 (`OutOfBoundsStrategy.ZERO`).
    pub fn by_id(id: i32) -> EquipmentSlot {
        Self::VALUES.get(id.max(0) as usize).copied().unwrap_or(EquipmentSlot::Mainhand)
    }

    /// `STREAM_CODEC` — idMapper.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, EquipmentSlot> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &EquipmentSlot| {
                buf.write_var_int(v.get_id());
            },
            |buf: &mut FriendlyByteBuf| EquipmentSlot::by_id(buf.read_var_int()),
        )
    }
}
