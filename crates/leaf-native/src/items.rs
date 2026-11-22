use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::OnceLock;

use leaf_core::core::component::data_component_type::data_component_type_by_identifier;
use leaf_core::core::component::{DataComponentMap, DataComponentPatch};
use leaf_core::core::holder::Holder;
use leaf_core::resources::Identifier;
use leaf_core::world::item::component::{
    firework_explosion::{FireworkExplosion, FireworkExplosionShape},
    fireworks::Fireworks,
};
use leaf_core::world::item::{item, item_stack::ItemStack};

use crate::wire::{invalid, Cursor};

pub const ELYTRA: i32 = 862;
pub const FIREWORK: i32 = 1242;
pub const ITEM_ENTITY: i32 = 71;
pub const FIREWORK_ENTITY: i32 = 53;
const DAMAGE: i32 = 3;
const FIREWORKS: i32 = 67;

pub struct Items {
    maxima: Vec<i32>,
    place: HashMap<i32, i32>,
    drops: HashMap<i32, i32>,
}

impl Items {
    pub fn vanilla() -> &'static Self {
        static ITEMS: OnceLock<Items> = OnceLock::new();
        ITEMS.get_or_init(|| {
            let mut items = Self {
                maxima: vec![],
                place: HashMap::new(),
                drops: HashMap::new(),
            };
            let stack_size =
                data_component_type_by_identifier(&Identifier::minecraft("max_stack_size"))
                    .unwrap();
            let fireworks =
                data_component_type_by_identifier(&Identifier::minecraft("fireworks")).unwrap();
            for line in include_str!("../data/items-774.txt").lines().skip(1) {
                let mut fields = line.split_ascii_whitespace();
                let id = fields.next().unwrap().parse::<i32>().unwrap();
                let name = fields.next().unwrap();
                let max = fields.next().unwrap().parse::<i32>().unwrap();
                let mut defaults = DataComponentMap::new();
                defaults.set(&stack_size, max);
                if id == FIREWORK {
                    defaults.set(&fireworks, Fireworks::new(1, vec![]));
                }
                let actual = item::register_item_with_components(
                    item::Item::new(Identifier::minecraft(name)),
                    defaults,
                );
                assert_eq!(id, actual, "vanilla item {name}");
                items.maxima.push(max);
                if let Ok(state) = fields.next().unwrap().parse() {
                    items.place.insert(id, state);
                    items.drops.insert(state, id);
                }
            }
            items
        })
    }

    pub fn place_state(&self, stack: &ItemStack) -> Option<i32> {
        self.place.get(&item_id(stack)?).copied()
    }

    pub fn block_drop(&self, state: i32) -> Option<ItemStack> {
        Some(stack(*self.drops.get(&state)?, 1))
    }

    pub fn read_creative_stack(&self, body: &mut Cursor) -> io::Result<ItemStack> {
        let count = body.varint()?;
        if count == 0 {
            return Ok(ItemStack::empty());
        }
        let id = body.varint()?;
        let limit = self
            .maxima
            .get(id as usize)
            .copied()
            .ok_or_else(|| invalid("unknown creative item"))?;
        if id == 0 || !(1..=99).contains(&count) {
            return Err(invalid("invalid creative stack count"));
        }
        let values = body.count(104)?;
        let removals = body.count(104)?;
        let mut patch = DataComponentPatch::builder();
        let mut seen = HashSet::new();
        for _ in 0..values {
            let component = body.varint()?;
            if !seen.insert(component) {
                return Err(invalid("duplicate creative component"));
            }
            let len = body.count(128 * 1024)?;
            let mut data = body.subcursor(len)?;
            match component {
                FIREWORKS if id == FIREWORK => {
                    let duration = data.count(255)? as i32;
                    let explosions = (0..data.count(256)?)
                        .map(|_| read_explosion(&mut data))
                        .collect::<io::Result<Vec<_>>>()?;
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft("fireworks"))
                            .unwrap(),
                        Fireworks::new(duration, explosions),
                    );
                }
                DAMAGE if id == ELYTRA => {
                    let damage = data.count(432)? as i32;
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft("damage"))
                            .unwrap(),
                        damage,
                    );
                }
                _ => {}
            }
        }
        for _ in 0..removals {
            let component = body.varint()?;
            if !seen.insert(component) {
                return Err(invalid("duplicate creative component"));
            }
        }
        Ok(ItemStack::with_patch(
            Holder::Reference(id),
            count.min(limit),
            patch.build(),
        ))
    }
}

fn read_explosion(data: &mut Cursor) -> io::Result<FireworkExplosion> {
    let shape = FireworkExplosionShape::by_id(data.varint()?);
    let read_colors = |data: &mut Cursor| -> io::Result<Vec<i32>> {
        (0..data.count(256)?)
            .map(|_| Ok(i32::from_be_bytes(data.fixed()?)))
            .collect()
    };
    Ok(FireworkExplosion {
        shape,
        colors: read_colors(data)?,
        fade_colors: read_colors(data)?,
        has_trail: data.fixed::<1>()?[0] != 0,
        has_twinkle: data.fixed::<1>()?[0] != 0,
    })
}

pub fn item_id(stack: &ItemStack) -> Option<i32> {
    match stack.item_holder() {
        Some(Holder::Reference(id)) if !stack.is_empty() => Some(*id),
        _ => None,
    }
}

pub fn stack(id: i32, count: i32) -> ItemStack {
    ItemStack::new(Holder::Reference(id), count)
}
