//! Перенос `net.minecraft.world.SimpleContainer` — простой
//! инвентарь-список `ItemStack` (villager inventory, horse
//! inventory, дроп-контейнеры).

use crate::resources::identifier::Identifier;
use crate::world::item::item_stack::ItemStack;

/// `ContainerListener` — колбэк на изменение содержимого.
pub type ContainerListener =
    std::rc::Rc<dyn Fn(&dyn ContainerView, usize) + Send + Sync>;

/// Минимальный view для listener'ов (как `Container`).
pub trait ContainerView {
    fn get_item(&self, index: usize) -> ItemStack;
    fn get_container_size(&self) -> usize;
    fn is_empty(&self) -> bool;
}

/// `SimpleContainer` — NonNullList<ItemStack> + listeners.
#[derive(Clone)]
pub struct SimpleContainer {
    /// `size`.
    pub size: usize,
    /// `items`.
    pub items: Vec<ItemStack>,
    /// `listeners`.
    pub listeners: Option<Vec<ContainerListener>>,
}

impl SimpleContainer {
    /// `new(size)`.
    pub fn new(size: usize) -> Self {
        Self {
            size,
            items: vec![ItemStack::empty(); size],
            listeners: None,
        }
    }

    /// `new(ItemStack...)`.
    pub fn of(items: Vec<ItemStack>) -> Self {
        Self {
            size: items.len(),
            items,
            listeners: None,
        }
    }

    /// `addListener`.
    pub fn add_listener(&mut self, listener: ContainerListener) {
        self.listeners.get_or_insert_with(Vec::new).push(listener);
    }

    /// `removeListener`.
    pub fn remove_listener(&mut self, listener: &ContainerListener) {
        if let Some(list) = &mut self.listeners {
            // Rc не PartialEq — удаляем по ptr-равенству.
            if let Some(pos) = list
                .iter()
                .position(|l| std::rc::Rc::ptr_eq(l, listener))
            {
                list.remove(pos);
            }
        }
    }

    /// `getItem(index)`.
    pub fn get_item(&self, index: usize) -> ItemStack {
        self.items.get(index).cloned().unwrap_or_else(ItemStack::empty)
    }

    /// `removeAllItems` — все непустые стеки + clear.
    pub fn remove_all_items(&mut self) -> Vec<ItemStack> {
        let list: Vec<ItemStack> =
            self.items.iter().filter(|s| !s.is_empty()).cloned().collect();
        self.clear_content();
        list
    }

    /// `removeItem(index, count)` — частичное удаление.
    pub fn remove_item(&mut self, index: usize, count: i32) -> ItemStack {
        let mut result = ItemStack::empty();
        if let Some(slot) = self.items.get_mut(index) {
            if !slot.is_empty() {
                if slot.get_count() <= count {
                    result = slot.clone();
                    *slot = ItemStack::empty();
                } else {
                    result = slot.split(count);
                }
            }
        }
        if !result.is_empty() {
            self.set_changed();
        }
        result
    }

    /// `removeItemType(item, amount)` — собрать `amount` штук
    /// предмета из инвентаря (с конца).
    pub fn remove_item_type(&mut self, item: &Identifier, amount: i32) -> ItemStack {
        let mut stack = ItemStack::empty();
        for i in (0..self.size).rev() {
            let needed = amount - stack.get_count();
            if needed <= 0 {
                break;
            }
            if self.items[i].is(item) && !self.items[i].is_empty() {
                let take = needed.min(self.items[i].get_count());
                let mut piece = self.items[i].split(take);
                piece.set_count(take);
                stack.grow(take);
            }
        }
        if !stack.is_empty() {
            self.set_changed();
        }
        stack
    }

    /// `addItem(stack)` — сначала в стаки того же типа, потом
    /// в пустые слоты. Возвращает остаток (пустой = всё влезло).
    pub fn add_item(&mut self, mut stack: ItemStack) -> ItemStack {
        if stack.is_empty() {
            return ItemStack::empty();
        }
        let mut item_stack = stack.copy();
        self.move_item_to_occupied_slots_with_same_type(&mut item_stack);
        if item_stack.is_empty() {
            return ItemStack::empty();
        }
        self.move_item_to_empty_slots(&mut item_stack);
        item_stack
    }

    /// `canAddItem`.
    pub fn can_add_item(&self, stack: &ItemStack) -> bool {
        self.items.iter().any(|s| {
            s.is_empty()
                || (ItemStack::matches(s, stack) && s.get_count() < s.get_max_stack_size())
        })
    }

    /// `removeItemNoUpdate`.
    pub fn remove_item_no_update(&mut self, index: usize) -> ItemStack {
        let item = self.items.get(index).cloned().unwrap_or_else(ItemStack::empty);
        if !item.is_empty() {
            if let Some(slot) = self.items.get_mut(index) {
                *slot = ItemStack::empty();
            }
        }
        item
    }

    /// `setItem`.
    pub fn set_item(&mut self, index: usize, stack: ItemStack) {
        if index < self.items.len() {
            self.items[index] = stack;
        }
        self.set_changed();
    }

    /// `getContainerSize`.
    pub fn get_container_size(&self) -> usize {
        self.size
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.items.iter().all(|s| s.is_empty())
    }

    /// `clearContent`.
    pub fn clear_content(&mut self) {
        for s in &mut self.items {
            *s = ItemStack::empty();
        }
        self.set_changed();
    }

    /// `countItem(item)` — суммарное количество предмета.
    pub fn count_item(&self, item: &Identifier) -> i32 {
        self.items
            .iter()
            .filter(|s| !s.is_empty() && s.is(item))
            .map(|s| s.get_count())
            .sum()
    }

    /// `hasAnyOf(items)` — есть ли хоть один из предметов.
    pub fn has_any_of(&self, items: &std::collections::HashSet<Identifier>) -> bool {
        self.items.iter().any(|s| {
            !s.is_empty()
                && s.item_identifier()
                    .map(|id| items.contains(&id))
                    .unwrap_or(false)
        })
    }

    /// `getMaxStackSize(stack)` — default 99 (как Container).
    pub fn get_max_stack_size(_stack: &ItemStack) -> i32 {
        99
    }

    /// `setChanged` — уведомить listeners.
    pub fn set_changed(&self) {
        if let Some(list) = &self.listeners {
            for l in list {
                l(self, usize::MAX);
            }
        }
    }

    fn move_item_to_occupied_slots_with_same_type(&mut self, stack: &mut ItemStack) {
        for slot in &mut self.items {
            if !stack.is_empty() && !slot.is_empty() && ItemStack::matches(slot, stack) {
                let max = slot.get_max_stack_size();
                let space = max - slot.get_count();
                if space > 0 {
                    let move_n = space.min(stack.get_count());
                    slot.grow(move_n);
                    stack.shrink(move_n);
                }
            }
        }
    }

    fn move_item_to_empty_slots(&mut self, stack: &mut ItemStack) {
        let max = stack.get_max_stack_size();
        for slot in &mut self.items {
            if stack.is_empty() {
                break;
            }
            if slot.is_empty() {
                let move_n = max.min(stack.get_count());
                *slot = stack.copy();
                slot.set_count(move_n);
                stack.shrink(move_n);
            }
        }
    }
}

impl ContainerView for SimpleContainer {
    fn get_item(&self, index: usize) -> ItemStack {
        SimpleContainer::get_item(self, index)
    }
    fn get_container_size(&self) -> usize {
        self.size
    }
    fn is_empty(&self) -> bool {
        SimpleContainer::is_empty(self)
    }
}

fn i1_unused(_s: ItemStack) {}

/// `net.minecraft.world.Container` — мутабельный контейнер
/// (chunk block entity inventory). Trait-объект для
/// `TransportItemsBetweenContainers` и будущих блок-сущностей.
pub trait Container: ContainerView {
    /// `removeItem(slot, count)`.
    fn remove_item(&mut self, index: usize, count: i32) -> ItemStack;
    /// `setItem(slot, stack)`.
    fn set_item(&mut self, index: usize, stack: ItemStack);
    /// `setChanged`.
    fn set_changed(&mut self);
    /// `BaseContainerBlockEntity.isLocked` — default false.
    fn is_locked(&self) -> bool {
        false
    }
    /// Итерация `for (ItemStack s : container)` — снимок стопок.
    fn item_snapshot(&self) -> Vec<ItemStack> {
        (0..self.get_container_size())
            .map(|i| self.get_item(i))
            .collect()
    }
}

impl Container for SimpleContainer {
    fn remove_item(&mut self, index: usize, count: i32) -> ItemStack {
        SimpleContainer::remove_item(self, index, count)
    }
    fn set_item(&mut self, index: usize, stack: ItemStack) {
        SimpleContainer::set_item(self, index, stack)
    }
    fn set_changed(&mut self) {
        SimpleContainer::set_changed(self)
    }
    fn item_snapshot(&self) -> Vec<ItemStack> {
        self.items.clone()
    }
}
