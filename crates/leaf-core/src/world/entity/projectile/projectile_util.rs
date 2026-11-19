//! Перенос `net.minecraft.world.entity.projectile.ProjectileUtil`.

use crate::entity::humanoid_arm::HumanoidArm;
use crate::resources::identifier::Identifier;
use crate::world::entity::living_entity::LivingEntity;
use crate::world::interaction_hand::InteractionHand;

/// `ProjectileUtil.getWeaponHoldingHand(shooter, weapon)` —
/// рука, в которой держится оружие: `MAIN_HAND`, если предмет
/// в основной руке совпадает, иначе `OFF_HAND`.
pub fn get_weapon_holding_hand(
    shooter: &LivingEntity,
    weapon: &Identifier,
) -> InteractionHand {
    if shooter.get_main_hand_item().is(weapon) {
        InteractionHand::MainHand
    } else {
        InteractionHand::OffHand
    }
}

/// `InteractionHand` → `HumanoidArm` руки сущности
/// (`LivingEntity.startUsingItem(InteractionHand)` конвертирует
/// через `getMainArm()`/`getOpposite()`).
pub fn hand_to_arm(shooter: &LivingEntity, hand: InteractionHand) -> HumanoidArm {
    let main = shooter.get_main_arm();
    match hand {
        InteractionHand::MainHand => main,
        InteractionHand::OffHand => main.get_opposite(),
    }
}
