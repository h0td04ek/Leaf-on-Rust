use leaf_core::core::block_pos::BlockPos;
use leaf_core::network::protocol::game::clientbound_container_set_content_packet::ClientboundContainerSetContentPacket;
use leaf_core::world::inventory::ClickType;
use leaf_core::world::item::item_stack::ItemStack;

use crate::gameplay::insert_inventory;
use crate::items::item_id;
use crate::maps::{map_id, set_map_id, Maps};
use crate::player_data::Inventory;
use crate::recipes::{Crafted, Recipe, Recipes};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Slot {
    Grid(usize),
    Player(usize),
}

struct Drag {
    mode: u8,
    slots: Vec<Slot>,
}

pub struct Crafting {
    pub id: i32,
    pub table: Option<BlockPos>,
    next_id: i32,
    state: i32,
    grid: [ItemStack; 9],
    cursor: ItemStack,
    drag: Option<Drag>,
}

impl Default for Crafting {
    fn default() -> Self {
        Self {
            id: 0,
            table: None,
            next_id: 0,
            state: 0,
            grid: std::array::from_fn(|_| ItemStack::empty()),
            cursor: ItemStack::empty(),
            drag: None,
        }
    }
}

impl Crafting {
    pub fn open(&mut self, pos: BlockPos) {
        self.next_id = self.next_id % 100 + 1;
        self.id = self.next_id;
        self.table = Some(pos);
        self.state = 0;
    }

    fn width(&self) -> usize {
        if self.table.is_some() {
            3
        } else {
            2
        }
    }

    fn result(&self, maps: &Maps) -> Option<Crafted> {
        Recipes::vanilla().find(&self.grid, self.width(), maps)
    }

    fn slot(&self, index: i16) -> Option<Slot> {
        match (self.table.is_some(), index) {
            (false, 1..=4) | (true, 1..=9) => Some(Slot::Grid((index - 1) as usize)),
            (false, 5..=8) => Some(Slot::Player((44 - index) as usize)),
            (false, 9..=35) => Some(Slot::Player(index as usize)),
            (false, 36..=44) => Some(Slot::Player((index - 36) as usize)),
            (false, 45) => Some(Slot::Player(40)),
            (true, 10..=36) => Some(Slot::Player((index - 1) as usize)),
            (true, 37..=45) => Some(Slot::Player((index - 37) as usize)),
            _ => None,
        }
    }

    fn get<'a>(&'a self, inventory: &'a Inventory, slot: Slot) -> &'a ItemStack {
        match slot {
            Slot::Grid(index) => &self.grid[index],
            Slot::Player(index) => &inventory[index],
        }
    }

    fn get_mut<'a>(&'a mut self, inventory: &'a mut Inventory, slot: Slot) -> &'a mut ItemStack {
        match slot {
            Slot::Grid(index) => &mut self.grid[index],
            Slot::Player(index) => &mut inventory[index],
        }
    }

    pub fn contents(
        &mut self,
        inventory: &Inventory,
        maps: &Maps,
    ) -> ClientboundContainerSetContentPacket {
        self.state = (self.state + 1) & 32767;
        let output = self
            .result(maps)
            .map_or_else(ItemStack::empty, |result| result.output);
        let mut items = vec![output];
        items.extend((1..=45).map(|index| self.get(inventory, self.slot(index).unwrap()).clone()));
        ClientboundContainerSetContentPacket::new(self.id, self.state, items, self.cursor.clone())
    }

    pub fn recovery_items(&self) -> impl Iterator<Item = &ItemStack> {
        self.grid
            .iter()
            .chain(std::iter::once(&self.cursor))
            .filter(|item| !item.is_empty())
    }

    pub fn close(&mut self, inventory: &mut Inventory) -> Vec<ItemStack> {
        self.id = 0;
        self.table = None;
        self.state = 0;
        self.drag = None;
        let mut changed = Vec::new();
        self.grid
            .iter_mut()
            .chain(std::iter::once(&mut self.cursor))
            .filter_map(|item| {
                let mut item = std::mem::replace(item, ItemStack::empty());
                insert_inventory(&mut inventory[..36], &mut item, &mut changed);
                (!item.is_empty()).then_some(item)
            })
            .collect()
    }

    pub fn click(
        &mut self,
        inventory: &mut Inventory,
        index: i16,
        button: u8,
        action: ClickType,
        creative: bool,
        maps: &Maps,
    ) -> Vec<ItemStack> {
        let mut dropped = Vec::new();
        if action != ClickType::QuickCraft {
            self.drag = None;
        }
        let slot = self.slot(index);
        match (action, index, slot) {
            (ClickType::Pickup, -999, _) if button <= 1 => {
                let amount = if button == 0 { self.cursor.count } else { 1 };
                drop_part(&mut self.cursor, amount, &mut dropped);
            }
            (_, 0, _) => self.take_result(inventory, button, action, creative, maps, &mut dropped),
            (ClickType::Pickup, _, Some(slot)) if button <= 1 => {
                let mut cursor = std::mem::replace(&mut self.cursor, ItemStack::empty());
                pickup(
                    self.get_mut(inventory, slot),
                    &mut cursor,
                    slot,
                    button == 1,
                );
                self.cursor = cursor;
            }
            (ClickType::QuickMove, _, Some(slot)) => {
                let mut taken =
                    std::mem::replace(self.get_mut(inventory, slot), ItemStack::empty());
                if !taken.is_empty() {
                    let mut changed = Vec::new();
                    let armor =
                        armor_slot(&taken).filter(|&i| self.id == 0 && inventory[i].is_empty());
                    match (slot, armor) {
                        (Slot::Player(0..=35), Some(i)) => inventory[i] = taken.split(1),
                        (Slot::Player(0..=8), _) => {
                            insert_inventory(&mut inventory[9..36], &mut taken, &mut changed)
                        }
                        (Slot::Player(9..=35), _) => {
                            insert_inventory(&mut inventory[..9], &mut taken, &mut changed)
                        }
                        _ => insert_inventory(&mut inventory[..36], &mut taken, &mut changed),
                    }
                }
                *self.get_mut(inventory, slot) = taken;
            }
            (ClickType::Swap, _, Some(slot)) if button < 9 || button == 40 => {
                let hotbar = button as usize;
                if slot != Slot::Player(hotbar) && can_insert(slot, &inventory[hotbar]) {
                    let incoming = std::mem::replace(&mut inventory[hotbar], ItemStack::empty());
                    inventory[hotbar] = std::mem::replace(self.get_mut(inventory, slot), incoming);
                }
            }
            (ClickType::Clone, _, Some(slot)) if creative && self.cursor.is_empty() => {
                let original = self.get(inventory, slot);
                self.cursor = original.copy_with_count(original.get_max_stack_size());
            }
            (ClickType::Throw, _, Some(slot)) if self.cursor.is_empty() && button <= 1 => {
                let current = self.get_mut(inventory, slot);
                let amount = if button == 0 { 1 } else { current.count };
                drop_part(current, amount, &mut dropped);
            }
            (ClickType::QuickCraft, _, _) => self.drag(inventory, slot, button, creative),
            (ClickType::PickupAll, _, _) if button <= 1 => self.pickup_all(inventory, button == 1),
            _ => {}
        }
        dropped
    }

    fn take_result(
        &mut self,
        inventory: &mut Inventory,
        button: u8,
        action: ClickType,
        creative: bool,
        maps: &Maps,
        dropped: &mut Vec<ItemStack>,
    ) {
        let mut changed = Vec::new();
        while let Some(crafted) = self.result(maps) {
            let mut output = crafted.output;
            if crafted.expand_map && action != ClickType::Clone {
                let fits_new = match action {
                    ClickType::Pickup => button <= 1 && self.cursor.is_empty(),
                    ClickType::QuickMove => inventory[..36].iter().any(ItemStack::is_empty),
                    ClickType::Swap => {
                        (button < 9 || button == 40) && inventory[button as usize].is_empty()
                    }
                    ClickType::Throw => button <= 1 && self.cursor.is_empty(),
                    _ => false,
                };
                if !fits_new {
                    break;
                }
                match maps.expand(map_id(&output).unwrap()) {
                    Ok(id) => set_map_id(&mut output, id),
                    Err(error) => {
                        eprintln!("Could not expand map: {error}");
                        break;
                    }
                }
            }
            let grid_count = self.width().pow(2);
            match action {
                ClickType::Pickup if button <= 1 && fits(&self.cursor, &output) => {
                    let count = output.count;
                    transfer(&mut self.cursor, &mut output, count);
                }
                ClickType::QuickMove if capacity(&inventory[..36], &output) >= output.count => {
                    insert_inventory(&mut inventory[..36], &mut output, &mut changed);
                }
                ClickType::Swap
                    if (button < 9 || button == 40)
                        && fits(&inventory[button as usize], &output) =>
                {
                    let count = output.count;
                    transfer(&mut inventory[button as usize], &mut output, count);
                }
                ClickType::Throw if self.cursor.is_empty() && button <= 1 => dropped.push(output),
                ClickType::Clone if creative && self.cursor.is_empty() => {
                    self.cursor = output.copy_with_count(output.get_max_stack_size());
                    break;
                }
                _ => break,
            }
            for item in &mut self.grid[..grid_count] {
                item.shrink(1);
            }
            for (slot, mut item) in crafted.remainders {
                if fits(&self.grid[slot], &item) {
                    let count = item.count;
                    transfer(&mut self.grid[slot], &mut item, count);
                }
                insert_inventory(&mut inventory[..36], &mut item, &mut changed);
                if !item.is_empty() {
                    dropped.push(item);
                }
            }
            if action != ClickType::QuickMove {
                break;
            }
        }
    }

    fn drag(&mut self, inventory: &mut Inventory, slot: Option<Slot>, button: u8, creative: bool) {
        let mode = (button >> 2) & 3;
        let phase = button & 3;
        if self.cursor.is_empty() || mode > 2 || mode == 2 && !creative {
            self.drag = None;
            return;
        }
        match phase {
            0 => {
                self.drag = Some(Drag {
                    mode,
                    slots: Vec::new(),
                })
            }
            1 => {
                let Some(slot) = slot else { return };
                let usable = can_insert(slot, &self.cursor)
                    && space(self.get(inventory, slot), &self.cursor) > 0;
                if let Some(drag) = &mut self.drag {
                    if usable
                        && drag.mode == mode
                        && !drag.slots.contains(&slot)
                        && (mode == 2 || self.cursor.count as usize > drag.slots.len())
                    {
                        drag.slots.push(slot);
                    }
                }
            }
            2 => {
                let Some(drag) = self.drag.take() else { return };
                if drag.mode != mode || drag.slots.is_empty() {
                    return;
                }
                let mut cursor = std::mem::replace(&mut self.cursor, ItemStack::empty());
                let amount = match mode {
                    0 => cursor.count / drag.slots.len() as i32,
                    1 => 1,
                    _ => cursor.get_max_stack_size(),
                };
                for slot in drag.slots {
                    let current = self.get_mut(inventory, slot);
                    let amount = amount
                        .min(space(current, &cursor))
                        .min(slot_limit(slot) - current.count);
                    if mode == 2 {
                        transfer(current, &mut cursor.copy_with_count(amount), amount);
                    } else {
                        transfer(current, &mut cursor, amount);
                    }
                }
                self.cursor = cursor;
            }
            _ => self.drag = None,
        }
    }

    fn pickup_all(&mut self, inventory: &mut Inventory, reverse: bool) {
        if self.cursor.is_empty() {
            return;
        }
        let mut cursor = std::mem::replace(&mut self.cursor, ItemStack::empty());
        let slots = (1..=45)
            .map(|i| self.slot(if reverse { 46 - i } else { i }).unwrap())
            .collect::<Vec<_>>();
        for skip_full in [true, false] {
            for &slot in &slots {
                let current = self.get_mut(inventory, slot);
                if skip_full && current.count >= current.get_max_stack_size() {
                    continue;
                }
                let amount = space(&cursor, current);
                transfer(&mut cursor, current, amount);
            }
        }
        self.cursor = cursor;
    }

    pub fn fill(&mut self, inventory: &mut Inventory, recipe: &Recipe, maximum: bool) -> bool {
        let Some(mut cells) = recipe.grid_slots(self.width()) else {
            return false;
        };
        let mut available = inventory.clone();
        let mut changed = Vec::new();
        for previous in &self.grid {
            let mut item = previous.clone();
            insert_inventory(&mut available[..36], &mut item, &mut changed);
            if !item.is_empty() {
                return false;
            }
        }
        let existing = recipe
            .matches(&self.grid, self.width())
            .then(|| {
                self.grid
                    .iter()
                    .filter(|item| !item.is_empty())
                    .map(|item| item.count)
                    .min()
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        let start = if maximum { 1 } else { existing + 1 };
        let mut high = if maximum { 64 } else { start };
        let mut low = start;
        let mut best = None;
        cells.sort_by_key(|(_, ingredients)| ingredients.len());
        while low <= high {
            let amount = (low + high) / 2;
            let mut trial = available.clone();
            let mut grid = std::array::from_fn(|_| ItemStack::empty());
            if fill_slots(&mut trial[..36], &mut grid, &cells, amount) {
                best = Some((trial, grid));
                low = amount + 1;
            } else {
                high = amount - 1;
            }
        }
        if let Some((new_inventory, grid)) = best {
            self.grid = grid;
            *inventory = new_inventory;
            true
        } else {
            false
        }
    }
}

fn fill_slots(
    inventory: &mut [ItemStack],
    grid: &mut [ItemStack],
    cells: &[(usize, &[i32])],
    amount: i32,
) -> bool {
    let supplies = inventory
        .iter()
        .enumerate()
        .flat_map(|(slot, item)| {
            let count = (item.count / amount).min(cells.len() as i32);
            (0..count).filter_map(move |_| item_id(item).map(|id| (slot, id)))
        })
        .collect::<Vec<_>>();
    let mut matched = vec![None; supplies.len()];
    for target in 0..cells.len() {
        let mut visited = vec![false; supplies.len()];
        if !assign_slot(cells, &supplies, target, &mut matched, &mut visited) {
            return false;
        }
    }
    for ((slot, _), target) in supplies.into_iter().zip(matched) {
        if let Some(target) = target {
            grid[cells[target].0] = inventory[slot].split(amount);
        }
    }
    true
}

fn assign_slot(
    cells: &[(usize, &[i32])],
    supplies: &[(usize, i32)],
    target: usize,
    matched: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    for (candidate, &(_, item)) in supplies.iter().enumerate() {
        if visited[candidate] || cells[target].1.binary_search(&item).is_err() {
            continue;
        }
        visited[candidate] = true;
        if matched[candidate].is_none_or(|old| assign_slot(cells, supplies, old, matched, visited))
        {
            matched[candidate] = Some(target);
            return true;
        }
    }
    false
}

fn pickup(current: &mut ItemStack, cursor: &mut ItemStack, slot: Slot, right: bool) {
    if cursor.is_empty() {
        let amount = if right {
            (current.count + 1) / 2
        } else {
            current.count
        };
        transfer(cursor, current, amount);
    } else if can_insert(slot, cursor) {
        let limit = slot_limit(slot);
        if current.is_empty() || same(current, cursor) {
            let amount = if right { 1 } else { cursor.count };
            let amount = amount
                .min(space(current, cursor))
                .min(limit - current.count);
            transfer(current, cursor, amount);
        } else if cursor.count <= limit {
            std::mem::swap(current, cursor);
        }
    }
}

fn armor_slot(item: &ItemStack) -> Option<usize> {
    let id = item.item_identifier()?;
    let name = id.get_path();
    if name.ends_with("_boots") {
        Some(36)
    } else if name.ends_with("_leggings") {
        Some(37)
    } else if name.ends_with("_chestplate") || name == "elytra" {
        Some(38)
    } else if name.ends_with("_helmet")
        || name.ends_with("_head")
        || name.ends_with("_skull")
        || name == "carved_pumpkin"
    {
        Some(39)
    } else {
        None
    }
}

fn can_insert(slot: Slot, item: &ItemStack) -> bool {
    match slot {
        Slot::Player(index @ 36..=39) => item.is_empty() || armor_slot(item) == Some(index),
        _ => true,
    }
}

fn slot_limit(slot: Slot) -> i32 {
    if matches!(slot, Slot::Player(36..=39)) {
        1
    } else {
        99
    }
}

fn space(target: &ItemStack, source: &ItemStack) -> i32 {
    if source.is_empty() {
        0
    } else if target.is_empty() || same(target, source) {
        (source.get_max_stack_size() - target.count).max(0)
    } else {
        0
    }
}

fn capacity(inventory: &[ItemStack], stack: &ItemStack) -> i32 {
    inventory.iter().map(|slot| space(slot, stack)).sum()
}

fn fits(target: &ItemStack, source: &ItemStack) -> bool {
    space(target, source) >= source.count
}

fn transfer(target: &mut ItemStack, source: &mut ItemStack, count: i32) {
    if count <= 0 {
        return;
    }
    if target.is_empty() {
        *target = source.split(count);
    } else {
        let moved = source.split(count);
        target.grow(moved.count);
    }
}

fn same(a: &ItemStack, b: &ItemStack) -> bool {
    ItemStack::is_same_item_same_components(a, b)
}

fn drop_part(item: &mut ItemStack, count: i32, dropped: &mut Vec<ItemStack>) {
    if !item.is_empty() {
        dropped.push(item.split(count));
    }
}
