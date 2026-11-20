//! Перенос `net.minecraft.world.entity.item.ItemEntity` — выпавший предмет.

use crate::core::holder::Holder;
use crate::world::entity::entity::Entity;
use crate::world::entity::entity_type;
use crate::world::item::item_stack::ItemStack;
use std::sync::Arc;

/// `ItemEntity` — сущность-капля предмета.
#[derive(Clone)]
pub struct ItemEntity {
    /// базовая `Entity`.
    pub entity: Entity,
    /// `item` — стек в дропе.
    pub item: ItemStack,
    /// `age` — возраст в тиках.
    pub age: i32,
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
        let mut entity = Entity::new(
            Holder::Direct(entity_type::item_entity_type()),
            level,
        );
        entity.pos = crate::world::phys::vec3::Vec3::new(x, y, z);
        Self {
            entity,
            item,
            age: 0,
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

    /// `setUnlimitedLifetime()` — `age = -6000`.
    pub fn set_unlimited_lifetime(&mut self) {
        self.age = -6000;
    }

    /// `setNoPickUpDelay()`.
    pub fn set_no_pick_up_delay(&mut self) {
        self.pickup_delay = 0;
    }

    /// `getItem()`.
    pub fn get_item(&self) -> &ItemStack {
        &self.item
    }

    /// `setItem(stack)` — кламп к max stack size.
    pub fn set_item(&mut self, stack: ItemStack) {
        self.item = stack;
    }

    /// `isMergeable` — может ли влиться в другой дроп
    /// (нет thrower/target и стек <= 64).
    pub fn is_mergeable(&self) -> bool {
        !self.item.is_empty() && self.thrower.is_none() && self.target.is_none()
    }
}

/// `ItemEntity.areMergable(a, b)` — одинаковый предмет/компоненты
/// и сумма не превышает max stack size.
pub fn item_entity_are_mergable(
    a: &crate::world::item::item_stack::ItemStack,
    b: &crate::world::item::item_stack::ItemStack,
) -> bool {
    crate::world::item::item_stack::ItemStack::matches(a, b)
        && a.get_count() + b.get_count()
            <= a.get_max_stack_size().max(b.get_max_stack_size())
}

/// `ItemEntity.merge(a, b, maxCount)` — слить `b` в `a` до `maxCount`,
/// возвращает объединённый стек (мутирует `b` через уменьшение count).
pub fn item_entity_merge(
    a: &crate::world::item::item_stack::ItemStack,
    b: &mut crate::world::item::item_stack::ItemStack,
    max_count: i32,
) -> crate::world::item::item_stack::ItemStack {
    let mut result = a.copy();
    let to_move = (max_count - result.get_count()).min(b.get_count());
    if to_move > 0 {
        result.grow(to_move);
        b.shrink(to_move);
    }
    result
}

