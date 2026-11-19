//! Перенос статических методов `net.minecraft.world.item.CrossbowItem`.

use crate::util::mth;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::item::enchantment::enchantment_helper;
use crate::world::item::item_stack::ItemStack;

/// `CrossbowItem.getChargeDuration(stack, shooter)` —
/// `floor(modifyCrossbowChargingTime(stack, shooter, 1.25) * 20)`.
pub fn get_charge_duration(stack: &ItemStack, shooter: &mut LivingEntity) -> i32 {
    let f = enchantment_helper::modify_crossbow_charging_time(stack, shooter, 1.25);
    mth::floor_f32(f * 20.0)
}
