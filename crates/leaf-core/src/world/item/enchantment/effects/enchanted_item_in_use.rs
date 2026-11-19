//! Перенос `net.minecraft.world.item.enchantment.EnchantedItemInUse`.

use crate::entity::equipment_slot::EquipmentSlot;
use crate::world::entity::entity::Entity;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::item::item_stack::ItemStack;

/// `EnchantedItemInUse` — предмет в использовании зачарованием.
///
/// В Java `onBreak` — `Consumer<Item>` (`owner.onEquippedItemBroken`);
/// здесь `on_break` применяется при разрушении через
/// `ItemStack::hurt_and_break`.
#[derive(Clone)]
pub struct EnchantedItemInUse {
    /// `itemStack`.
    pub item_stack: ItemStack,
    /// `inSlot` — слот экипировки (`None` — не экипирован).
    pub in_slot: Option<EquipmentSlot>,
    /// `owner` — владелец (`LivingEntity`).
    pub owner: Option<LivingEntity>,
}

impl EnchantedItemInUse {
    /// `new(itemStack, inSlot, owner)` — `onBreak` =
    /// `owner.onEquippedItemBroken(item, slot)` (реализуется
    /// через `hurt_and_break` на owner).
    pub fn new(
        item_stack: ItemStack,
        in_slot: Option<EquipmentSlot>,
        owner: Option<LivingEntity>,
    ) -> Self {
        Self { item_stack, in_slot, owner }
    }

    /// `new(itemStack, slot, owner)` без Option-обёрток (удобство).
    pub fn in_slot(
        item_stack: ItemStack,
        slot: EquipmentSlot,
        owner: Option<LivingEntity>,
    ) -> Self {
        Self { item_stack, in_slot: Some(slot), owner }
    }

    /// `owner()` — `Entity`-вид владельца.
    pub fn owner_entity(&self) -> Option<&Entity> {
        self.owner.as_ref().map(|l| &l.entity)
    }
}
