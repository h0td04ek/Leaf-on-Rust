//! Перенос `net.minecraft.world.entity.item.ItemEntity` — выпавший предмет.

use crate::core::holder::Holder;
use crate::world::entity::entity::{Entity, RemovalReason};
use crate::world::entity::entity_type;
use crate::world::item::item_stack::ItemStack;
use std::sync::Arc;

const INFINITE_PICKUP_DELAY: i32 = i16::MAX as i32;
const INFINITE_LIFETIME_AGE: i32 = i16::MIN as i32;
const DEFAULT_DESPAWN_RATE: i32 = 6000;

/// `ItemEntity` — выпавший предмет.
#[derive(Clone)]
pub struct ItemEntity {
    /// базовая `Entity`.
    pub entity: Entity,
    /// `item` — стек в дропе.
    pub item: ItemStack,
    /// `age` — возраст в тиках.
    pub age: i32,
    /// Скорость исчезновения предмета (Paper).
    pub despawn_rate: i32,
    /// `pickupDelay` — задержка подбора.
    pub pickup_delay: i32,
    /// `health` — "прочность" дропа (5.0).
    pub health: f32,
    /// `thrower` — UUID бросившего.
    pub thrower: Option<crate::util::uuid::Uuid>,
    /// `target` — UUID цели подбора.
    pub target: Option<crate::util::uuid::Uuid>,
}

impl ItemEntity {
    /// `new(level, x, y, z, stack)`.
    pub fn new(
        level: Option<Arc<dyn crate::world::level::level_reader::LevelReader>>,
        x: f64,
        y: f64,
        z: f64,
        item: ItemStack,
    ) -> Self {
        let mut entity = Entity::new(Holder::Direct(entity_type::item_entity_type()), level);
        entity.set_pos(x, y, z);
        Self {
            entity,
            item,
            age: 0,
            despawn_rate: DEFAULT_DESPAWN_RATE,
            pickup_delay: 10,
            health: 5.0,
            thrower: None,
            target: None,
        }
    }

    /// `setDefaultPickUpDelay()` — 10 тиков.
    pub fn set_default_pick_up_delay(&mut self) {
        self.pickup_delay = 10;
    }

    /// `setUnlimitedLifetime()`.
    pub fn set_unlimited_lifetime(&mut self) {
        self.age = INFINITE_LIFETIME_AGE;
    }

    /// `setNoPickUpDelay()`.
    pub fn set_no_pick_up_delay(&mut self) {
        self.pickup_delay = 0;
    }

    /// `getItem()`.
    pub fn get_item(&self) -> &ItemStack {
        &self.item
    }

    /// `setItem(stack)`.
    pub fn set_item(&mut self, stack: ItemStack) {
        self.item = stack;
    }

    /// `isMergable` — состояние дропа допускает слияние.
    pub fn is_mergeable(&self) -> bool {
        self.entity.is_alive()
            && self.pickup_delay != INFINITE_PICKUP_DELAY
            && self.age != INFINITE_LIFETIME_AGE
            && self.age < self.despawn_rate
            && !self.item.is_empty()
            && self.item.get_count() < self.item.get_max_stack_size()
    }

    /// Объединяет совместимые дропы с одинаковой целью подбора.
    /// При включённой настройке Leaf входящий предмет сливается в текущий.
    pub fn try_to_merge(&mut self, other: &mut Self, spigot_item_merging: bool) -> bool {
        if self.target != other.target
            || !self.is_mergeable()
            || !other.is_mergeable()
            || !item_entity_are_mergable(&self.item, &other.item)
        {
            return false;
        }

        let (destination, origin) =
            if spigot_item_merging || other.item.get_count() < self.item.get_count() {
                (self, other)
            } else {
                (other, self)
            };
        let merged = item_entity_merge(&destination.item, &mut origin.item, 64);
        if merged.get_count() == destination.item.get_count() {
            return false;
        }
        destination.item = merged;
        destination.pickup_delay = destination.pickup_delay.max(origin.pickup_delay);
        destination.age = destination.age.min(origin.age);
        if origin.item.is_empty() {
            origin.entity.remove(RemovalReason::Discarded);
        }
        true
    }
}

/// `ItemEntity.areMergable(a, b)` — одинаковый предмет/компоненты
/// и сумма не превышает max stack size.
pub fn item_entity_are_mergable(
    a: &crate::world::item::item_stack::ItemStack,
    b: &crate::world::item::item_stack::ItemStack,
) -> bool {
    !a.is_empty()
        && !b.is_empty()
        && i64::from(a.get_count()) + i64::from(b.get_count()) <= i64::from(b.get_max_stack_size())
        && ItemStack::is_same_item_same_components(a, b)
}

/// `ItemEntity.merge(a, b, maxCount)` — слить `b` в `a` до `maxCount`,
/// возвращает объединённый стек (мутирует `b` через уменьшение count).
pub fn item_entity_merge(
    a: &crate::world::item::item_stack::ItemStack,
    b: &mut crate::world::item::item_stack::ItemStack,
    max_count: i32,
) -> crate::world::item::item_stack::ItemStack {
    let capacity = a
        .get_max_stack_size()
        .min(max_count)
        .saturating_sub(a.get_count());
    let to_move = capacity.min(b.get_count()).max(0);
    if to_move > 0 {
        b.shrink(to_move);
    }
    a.copy_with_count(a.get_count() + to_move)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::Identifier;
    use crate::world::item::item::Item;

    fn stack(count: i32) -> ItemStack {
        ItemStack::new(
            Holder::Direct(Item::new(Identifier::minecraft("stone"))),
            count,
        )
    }

    #[test]
    fn merges_different_counts_for_same_pickup_target() {
        let target = crate::util::uuid::Uuid::random();
        let mut destination = ItemEntity::new(None, 0.0, 0.0, 0.0, stack(20));
        destination.target = Some(target);
        destination.age = 200;
        let mut origin = ItemEntity::new(None, 0.0, 0.0, 0.0, stack(2));
        origin.target = Some(target);
        origin.pickup_delay = 30;
        origin.age = 17;

        assert!(destination.try_to_merge(&mut origin, false));
        assert_eq!(destination.item.get_count(), 22);
        assert_eq!(destination.pickup_delay, 30);
        assert_eq!(destination.age, 17);
        assert!(origin.item.is_empty());
        assert_eq!(
            origin.entity.get_removal_reason(),
            Some(RemovalReason::Discarded)
        );
    }

    #[test]
    fn respects_components_and_lifecycle() {
        let mut a = stack(2);
        let mut b = stack(2);
        let damage = Identifier::minecraft("damage");
        a.set_component(&damage, Some(1i32));
        b.set_component(&damage, Some(2i32));
        assert!(!item_entity_are_mergable(&a, &b));

        let mut origin = ItemEntity::new(None, 0.0, 0.0, 0.0, a);
        origin.set_unlimited_lifetime();
        assert!(!origin.is_mergeable());
        assert_eq!(origin.age, -32768);
    }
}
