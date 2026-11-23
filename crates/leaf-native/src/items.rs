use std::collections::{HashMap, HashSet};
use std::io;
use std::sync::OnceLock;

use leaf_core::core::component::data_component_type::data_component_type_by_identifier;
use leaf_core::core::component::data_components::crafting_registry_len;
use leaf_core::core::component::{DataComponentMap, DataComponentPatch};
use leaf_core::core::holder::Holder;
use leaf_core::resources::Identifier;
use leaf_core::world::effect::mob_effect_instance::{MobEffectDetails, MobEffectInstance};
use leaf_core::world::item::component::{
    dyed_item_color::DyedItemColor,
    firework_explosion::{FireworkExplosion, FireworkExplosionShape},
    fireworks::Fireworks,
    potion_contents::PotionContents,
};
use leaf_core::world::item::dye_color::DyeColor;
use leaf_core::world::item::{item, item_stack::ItemStack};
use leaf_core::world::level::{
    block::entity::banner_pattern_layers::{BannerPatternLayer, BannerPatternLayers},
    saveddata::maps::MapId,
};

use crate::wire::{invalid, Cursor};

pub const ELYTRA: i32 = 862;
pub const FIREWORK: i32 = 1242;
pub const ITEM_ENTITY: i32 = 71;
pub const FIREWORK_ENTITY: i32 = 53;
const DAMAGE: i32 = 3;
const DYED_COLOR: i32 = 42;
const MAP_ID: i32 = 44;
const POTION_CONTENTS: i32 = 49;
const FIREWORK_EXPLOSION: i32 = 66;
const BANNER_PATTERNS: i32 = 70;
const BASE_COLOR: i32 = 71;
const FIREWORKS: i32 = 67;

pub struct Items {
    maxima: Vec<i32>,
    damageables: HashMap<i32, i32>,
}

impl Items {
    pub fn vanilla() -> &'static Self {
        static ITEMS: OnceLock<Items> = OnceLock::new();
        ITEMS.get_or_init(|| {
            let mut items = Self {
                maxima: vec![],
                damageables: include_str!("../data/damageables-774.txt")
                    .lines()
                    .skip(1)
                    .map(|row| {
                        let (id, damage) = row.split_once(' ').unwrap();
                        (id.parse().unwrap(), damage.parse().unwrap())
                    })
                    .collect(),
            };
            let stack_size =
                data_component_type_by_identifier(&Identifier::minecraft("max_stack_size"))
                    .unwrap();
            let max_damage =
                data_component_type_by_identifier(&Identifier::minecraft("max_damage")).unwrap();
            let fireworks =
                data_component_type_by_identifier(&Identifier::minecraft("fireworks")).unwrap();
            for line in include_str!("../data/items-774.txt").lines().skip(1) {
                let mut fields = line.split_ascii_whitespace();
                let id = fields.next().unwrap().parse::<i32>().unwrap();
                let name = fields.next().unwrap();
                let max = fields.next().unwrap().parse::<i32>().unwrap();
                let mut defaults = DataComponentMap::new();
                defaults.set(&stack_size, max);
                if let Some(&damage) = items.damageables.get(&id) {
                    defaults.set(&max_damage, damage);
                    defaults.set(
                        &data_component_type_by_identifier(&Identifier::minecraft("damage"))
                            .unwrap(),
                        0_i32,
                    );
                }
                if id == FIREWORK {
                    defaults.set(&fireworks, Fireworks::new(1, vec![]));
                }
                let actual = item::register_item_with_components(
                    item::Item::new(Identifier::minecraft(name)),
                    defaults,
                );
                assert_eq!(id, actual, "vanilla item {name}");
                items.maxima.push(max);
            }
            items
        })
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
                DAMAGE if self.damageables.contains_key(&id) => {
                    let damage = data.count(self.damageables[&id] as usize)? as i32;
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft("damage"))
                            .unwrap(),
                        damage,
                    );
                }
                FIREWORK_EXPLOSION if id == named_id("firework_star") => {
                    let explosion = read_explosion(&mut data)?;
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft(
                            "firework_explosion",
                        ))
                        .unwrap(),
                        explosion,
                    );
                }
                POTION_CONTENTS => {
                    let potion = read_potion(&mut data)?;
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft(
                            "potion_contents",
                        ))
                        .unwrap(),
                        potion,
                    );
                }
                DYED_COLOR => {
                    let rgb = i32::from_be_bytes(data.fixed()?);
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft("dyed_color"))
                            .unwrap(),
                        DyedItemColor { rgb },
                    );
                }
                MAP_ID if id == named_id("filled_map") => {
                    let map = MapId(data.varint()?);
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft("map_id"))
                            .unwrap(),
                        map,
                    );
                }
                BANNER_PATTERNS => {
                    let layers = (0..data.count(256)?)
                        .map(|_| {
                            let pattern = data.count(crafting_registry_len("banner_pattern"))?;
                            if pattern == 0 {
                                return Err(invalid("expected a registered banner pattern"));
                            }
                            let color = DyeColor::by_id(data.count(15)? as i32);
                            Ok(BannerPatternLayer::new(
                                Holder::Reference(pattern as i32 - 1),
                                color,
                            ))
                        })
                        .collect::<io::Result<Vec<_>>>()?;
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft(
                            "banner_patterns",
                        ))
                        .unwrap(),
                        BannerPatternLayers::new(layers),
                    );
                }
                BASE_COLOR => {
                    let color = DyeColor::by_id(data.count(15)? as i32);
                    data.finish()?;
                    patch.set(
                        &data_component_type_by_identifier(&Identifier::minecraft("base_color"))
                            .unwrap(),
                        color,
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

fn read_potion(data: &mut Cursor) -> io::Result<PotionContents> {
    let potion = boolean(data)?
        .then(|| {
            Ok::<_, io::Error>(Holder::Reference(
                data.count(crafting_registry_len("potion") - 1)? as i32,
            ))
        })
        .transpose()?;
    let color = boolean(data)?
        .then(|| Ok::<_, io::Error>(i32::from_be_bytes(data.fixed()?)))
        .transpose()?;
    let effects = (0..data.count(256)?)
        .map(|_| {
            let effect =
                Holder::Reference(data.count(crafting_registry_len("mob_effect") - 1)? as i32);
            Ok(MobEffectInstance::new(effect, read_effect(data, 0)?))
        })
        .collect::<io::Result<Vec<_>>>()?;
    let name = boolean(data)?.then(|| data.string(256)).transpose()?;
    Ok(PotionContents::new(potion, color, effects, name))
}

fn read_effect(data: &mut Cursor, depth: usize) -> io::Result<MobEffectDetails> {
    if depth >= 16 {
        return Err(invalid("too many hidden effects"));
    }
    let amplifier = data.count(255)? as i32;
    let duration = data.varint()?;
    let ambient = boolean(data)?;
    let show_particles = boolean(data)?;
    let show_icon = boolean(data)?;
    let hidden = boolean(data)?
        .then(|| read_effect(data, depth + 1).map(Box::new))
        .transpose()?;
    Ok(MobEffectDetails::new(
        amplifier,
        duration,
        ambient,
        show_particles,
        show_icon,
        hidden,
    ))
}

fn boolean(data: &mut Cursor) -> io::Result<bool> {
    Ok(data.fixed::<1>()?[0] != 0)
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

pub fn named_id(path: &str) -> i32 {
    let item =
        item::item_by_identifier(&Identifier::minecraft(path)).expect("registered vanilla item");
    item::item_registry_id(&item).unwrap()
}

pub fn stack(id: i32, count: i32) -> ItemStack {
    ItemStack::new(Holder::Reference(id), count)
}
