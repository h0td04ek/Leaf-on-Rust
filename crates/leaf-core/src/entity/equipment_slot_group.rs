//! Перенос `net.minecraft.world.entity.EquipmentSlotGroup`.

use crate::entity::equipment_slot::{EquipmentSlot, EquipmentSlotType};
use crate::network::codec::stream_codec::{sync_fn, StreamCodec};
use crate::network::friendly_byte_buf::FriendlyByteBuf;

/// `EquipmentSlotGroup` — группа слотов (id 0-10 = wire id).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EquipmentSlotGroup {
    /// `ANY`.
    Any,
    /// `MAINHAND`.
    Mainhand,
    /// `OFFHAND`.
    Offhand,
    /// `HAND`.
    Hand,
    /// `FEET`.
    Feet,
    /// `LEGS`.
    Legs,
    /// `CHEST`.
    Chest,
    /// `HEAD`.
    Head,
    /// `ARMOR`.
    Armor,
    /// `BODY`.
    Body,
    /// `SADDLE`.
    Saddle,
}

impl EquipmentSlotGroup {
    /// `VALUES_ARRAY`.
    pub const VALUES: [EquipmentSlotGroup; 11] = [
        EquipmentSlotGroup::Any,
        EquipmentSlotGroup::Mainhand,
        EquipmentSlotGroup::Offhand,
        EquipmentSlotGroup::Hand,
        EquipmentSlotGroup::Feet,
        EquipmentSlotGroup::Legs,
        EquipmentSlotGroup::Chest,
        EquipmentSlotGroup::Head,
        EquipmentSlotGroup::Armor,
        EquipmentSlotGroup::Body,
        EquipmentSlotGroup::Saddle,
    ];

    /// `getSerializedName`.
    pub fn get_serialized_name(&self) -> &'static str {
        match self {
            EquipmentSlotGroup::Any => "any",
            EquipmentSlotGroup::Mainhand => "mainhand",
            EquipmentSlotGroup::Offhand => "offhand",
            EquipmentSlotGroup::Hand => "hand",
            EquipmentSlotGroup::Feet => "feet",
            EquipmentSlotGroup::Legs => "legs",
            EquipmentSlotGroup::Chest => "chest",
            EquipmentSlotGroup::Head => "head",
            EquipmentSlotGroup::Armor => "armor",
            EquipmentSlotGroup::Body => "body",
            EquipmentSlotGroup::Saddle => "saddle",
        }
    }

    /// `getId` — wire id.
    pub fn get_id(&self) -> i32 {
        *self as i32
    }

    /// `test(EquipmentSlot)` — predicate Java-конструктора.
    pub fn test(&self, slot: EquipmentSlot) -> bool {
        match self {
            EquipmentSlotGroup::Any => true,
            EquipmentSlotGroup::Mainhand => slot == EquipmentSlot::Mainhand,
            EquipmentSlotGroup::Offhand => slot == EquipmentSlot::Offhand,
            EquipmentSlotGroup::Hand => slot.get_type() == EquipmentSlotType::Hand,
            EquipmentSlotGroup::Feet => slot == EquipmentSlot::Feet,
            EquipmentSlotGroup::Legs => slot == EquipmentSlot::Legs,
            EquipmentSlotGroup::Chest => slot == EquipmentSlot::Chest,
            EquipmentSlotGroup::Head => slot == EquipmentSlot::Head,
            EquipmentSlotGroup::Armor => slot.is_armor(),
            EquipmentSlotGroup::Body => slot == EquipmentSlot::Body,
            EquipmentSlotGroup::Saddle => slot == EquipmentSlot::Saddle,
        }
    }

    /// `bySlot`.
    pub fn by_slot(slot: EquipmentSlot) -> EquipmentSlotGroup {
        match slot {
            EquipmentSlot::Mainhand => EquipmentSlotGroup::Mainhand,
            EquipmentSlot::Offhand => EquipmentSlotGroup::Offhand,
            EquipmentSlot::Feet => EquipmentSlotGroup::Feet,
            EquipmentSlot::Legs => EquipmentSlotGroup::Legs,
            EquipmentSlot::Chest => EquipmentSlotGroup::Chest,
            EquipmentSlot::Head => EquipmentSlotGroup::Head,
            EquipmentSlot::Body => EquipmentSlotGroup::Body,
            EquipmentSlot::Saddle => EquipmentSlotGroup::Saddle,
        }
    }

    /// `BY_ID` — `OutOfBoundsStrategy.ZERO`.
    pub fn by_id(id: i32) -> EquipmentSlotGroup {
        Self::VALUES.get(id.max(0) as usize).copied().unwrap_or(EquipmentSlotGroup::Any)
    }

    /// `STREAM_CODEC` — idMapper.
    pub fn stream_codec() -> impl StreamCodec<FriendlyByteBuf, EquipmentSlotGroup> + Send + Sync + 'static {
        sync_fn(
            |buf: &mut FriendlyByteBuf, v: &EquipmentSlotGroup| {
                buf.write_var_int(v.get_id());
            },
            |buf: &mut FriendlyByteBuf| EquipmentSlotGroup::by_id(buf.read_var_int()),
        )
    }
}
