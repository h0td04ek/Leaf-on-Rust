use leaf_core::core::holder::Holder;
use leaf_core::resources::Identifier;
use leaf_core::world::item::component::{
    dyed_item_color::DyedItemColor,
    firework_explosion::{FireworkExplosion, FireworkExplosionShape as Shape},
    fireworks::Fireworks,
    potion_contents::PotionContents,
    written_book_content::WrittenBookContent,
};
use leaf_core::world::item::{dye_color::DyeColor, item_stack::ItemStack};
use leaf_core::world::level::{
    block::entity::{banner_pattern_layers::BannerPatternLayers, pot_decorations::PotDecorations},
    saveddata::maps::MapId,
};

use crate::items::{named_id, stack, FIREWORK};
use crate::maps::{map_id, Maps};
use crate::recipes::{Crafted, Grid};

pub fn craft(input: &Grid<'_>, maps: &Maps) -> Option<Crafted> {
    let names = input
        .filled
        .iter()
        .map(|&i| input.slots[i].item_identifier())
        .collect::<Option<Vec<_>>>()?;
    let names = names.iter().map(Identifier::get_path).collect::<Vec<_>>();
    rockets(input, &names)
        .or_else(|| firework_star(input, &names))
        .or_else(|| dye_armor(input, &names))
        .or_else(|| decorated_pot(input, &names))
        .or_else(|| tipped_arrows(input, &names))
        .or_else(|| repair(input))
        .or_else(|| clone_map(input, &names))
        .or_else(|| expand_map(input, &names, maps))
        .or_else(|| clone_book(input, &names))
        .or_else(|| banners(input, &names))
}

fn rockets(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    let gunpowder = names.iter().filter(|&&name| name == "gunpowder").count();
    if !(1..=3).contains(&gunpowder)
        || names.iter().filter(|&&name| name == "paper").count() != 1
        || names
            .iter()
            .any(|&name| !matches!(name, "paper" | "gunpowder" | "firework_star"))
    {
        return None;
    }
    let explosions = input
        .filled
        .iter()
        .filter_map(|&i| {
            input.slots[i]
                .get_component::<FireworkExplosion>(&Identifier::minecraft("firework_explosion"))
        })
        .collect();
    let mut output = stack(FIREWORK, 3);
    output.set_component(
        &Identifier::minecraft("fireworks"),
        Some(Fireworks::new(gunpowder as i32, explosions)),
    );
    Some(input.remainder(output))
}

fn firework_star(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    let key = Identifier::minecraft("firework_explosion");
    let colors = names
        .iter()
        .filter_map(|name| dye(name))
        .map(|dye| dye.firework_color())
        .collect::<Vec<_>>();
    if colors.is_empty() {
        return None;
    }
    if colors.len() + 1 == names.len() {
        if let Some(index) = names.iter().position(|&name| name == "firework_star") {
            let item = &input.slots[input.filled[index]];
            let mut effect = item
                .get_component::<FireworkExplosion>(&key)
                .unwrap_or_else(FireworkExplosion::default_explosion);
            effect.fade_colors = colors;
            let mut output = item.copy_with_count(1);
            output.set_component(&key, Some(effect));
            return Some(input.remainder(output));
        }
    }

    let mut has_powder = false;
    let mut effect = FireworkExplosion::default_explosion();
    for &name in names.iter().filter(|name| dye(name).is_none()) {
        match name {
            "gunpowder" if !has_powder => has_powder = true,
            "diamond" if !effect.has_trail => effect.has_trail = true,
            "glowstone_dust" if !effect.has_twinkle => effect.has_twinkle = true,
            _ if effect.shape == Shape::SMALL_BALL => {
                effect.shape = match name {
                    "fire_charge" => Shape::LARGE_BALL,
                    "feather" => Shape::BURST,
                    "gold_nugget" => Shape::STAR,
                    "skeleton_skull"
                    | "wither_skeleton_skull"
                    | "creeper_head"
                    | "player_head"
                    | "dragon_head"
                    | "zombie_head"
                    | "piglin_head" => Shape::CREEPER,
                    _ => return None,
                }
            }
            _ => return None,
        }
    }
    if !has_powder {
        return None;
    }
    effect.colors = colors;
    let mut output = stack(named_id("firework_star"), 1);
    output.set_component(&key, Some(effect));
    Some(input.remainder(output))
}

fn dye(name: &str) -> Option<DyeColor> {
    DyeColor::by_name(name.strip_suffix("_dye")?)
}

fn dye_armor(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    let key = Identifier::minecraft("dyed_color");
    let armor = names.iter().position(|name| {
        matches!(
            *name,
            "leather_helmet"
                | "leather_chestplate"
                | "leather_leggings"
                | "leather_boots"
                | "leather_horse_armor"
                | "wolf_armor"
        )
    })?;
    let mut colors = names
        .iter()
        .enumerate()
        .filter(|(index, _)| *index != armor)
        .map(|(_, name)| dye(name).map(|dye| dye.col()))
        .collect::<Option<Vec<_>>>()?;
    if colors.is_empty() {
        return None;
    }
    let current = &input.slots[input.filled[armor]];
    if let Some(color) = current.get_component::<DyedItemColor>(&key) {
        colors.push(color.rgb);
    }
    let mut totals = [0; 3];
    let mut brightness = 0;
    for &rgb in &colors {
        let parts = [(rgb >> 16) & 255, (rgb >> 8) & 255, rgb & 255];
        for (total, part) in totals.iter_mut().zip(parts) {
            *total += part;
        }
        brightness += *parts.iter().max().unwrap();
    }
    let count = colors.len() as i32;
    let average = totals.map(|total| total / count);
    let scale = brightness as f32 / count as f32 / *average.iter().max().unwrap() as f32;
    let rgb = average.into_iter().fold(0, |packed, part| {
        (packed << 8) | (part as f32 * scale) as i32
    });
    let mut output = current.copy_with_count(1);
    output.set_component(&key, Some(DyedItemColor { rgb }));
    Some(input.remainder(output))
}

fn decorated_pot(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    if input.size != 3
        || input.filled.as_slice() != [1, 3, 5, 7]
        || names
            .iter()
            .any(|name| *name != "brick" && !name.ends_with("_pottery_sherd"))
    {
        return None;
    }
    let side = |i: usize| input.slots[input.filled[i]].item_holder().cloned();
    let mut output = stack(named_id("decorated_pot"), 1);
    if names.iter().any(|&name| name != "brick") {
        output.set_component(
            &Identifier::minecraft("pot_decorations"),
            Some(PotDecorations {
                back: side(0),
                left: side(1),
                right: side(2),
                front: side(3),
                brick_holder: Some(Holder::Reference(named_id("brick"))),
            }),
        );
    }
    Some(input.remainder(output))
}

fn tipped_arrows(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    if names.len() != 9
        || names
            .iter()
            .enumerate()
            .any(|(i, &name)| name != if i == 4 { "lingering_potion" } else { "arrow" })
    {
        return None;
    }
    let key = Identifier::minecraft("potion_contents");
    let mut output = stack(named_id("tipped_arrow"), 8);
    output.set_component(&key, input.slots[4].get_component::<PotionContents>(&key));
    Some(input.remainder(output))
}

fn repair(input: &Grid<'_>) -> Option<Crafted> {
    let &[first, second] = input.filled.as_slice() else {
        return None;
    };
    let (a, b) = (&input.slots[first], &input.slots[second]);
    if !ItemStack::is_same_item(a, b)
        || a.count != 1
        || b.count != 1
        || !a.is_damageable_item()
        || !b.is_damageable_item()
    {
        return None;
    }
    let maximum = a.get_max_damage().max(b.get_max_damage());
    let remaining =
        (a.get_max_damage() - a.get_damage_value()) + (b.get_max_damage() - b.get_damage_value());
    let mut output = ItemStack::new(a.item_holder()?.clone(), 1);
    if output.get_max_damage() != maximum {
        output.set_component(&Identifier::minecraft("max_damage"), Some(maximum));
    }
    output.set_damage_value((maximum - remaining - maximum / 20).max(0));
    Some(input.remainder(output))
}

fn clone_map(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    let index = names.iter().position(|&name| name == "filled_map")?;
    if names.len() < 2
        || names
            .iter()
            .enumerate()
            .any(|(i, &name)| i != index && name != "map")
    {
        return None;
    }
    let original = &input.slots[input.filled[index]];
    original.get_component::<MapId>(&Identifier::minecraft("map_id"))?;
    Some(input.remainder(original.copy_with_count(names.len() as i32)))
}

fn expand_map(input: &Grid<'_>, names: &[&str], maps: &Maps) -> Option<Crafted> {
    if names.len() != 9
        || names
            .iter()
            .enumerate()
            .any(|(i, &name)| name != if i == 4 { "filled_map" } else { "paper" })
    {
        return None;
    }
    let source = &input.slots[4];
    if !maps.can_expand(map_id(source)?) {
        return None;
    }
    let mut crafted = input.remainder(source.copy_with_count(1));
    crafted.expand_map = true;
    Some(crafted)
}

fn clone_book(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    let index = names.iter().position(|&name| name == "written_book")?;
    if names.len() < 2
        || names
            .iter()
            .enumerate()
            .any(|(i, &name)| i != index && name != "writable_book")
    {
        return None;
    }
    let key = Identifier::minecraft("written_book_content");
    let position = input.filled[index];
    let original = &input.slots[position];
    let mut content = original.get_component::<WrittenBookContent>(&key)?;
    if content.generation >= 2 {
        return None;
    }
    content.generation += 1;
    let mut output = original.copy_with_count(names.len() as i32 - 1);
    output.set_component(&key, Some(content));
    Some(Crafted {
        output,
        remainders: vec![(position, original.copy_with_count(1))],
        expand_map: false,
    })
}

fn banners(input: &Grid<'_>, names: &[&str]) -> Option<Crafted> {
    let &[first, second] = input.filled.as_slice() else {
        return None;
    };
    let key = Identifier::minecraft("banner_patterns");
    let first_banner = names[0].strip_suffix("_banner").and_then(DyeColor::by_name);
    let second_banner = names[1].strip_suffix("_banner").and_then(DyeColor::by_name);
    let a = &input.slots[first];
    let b = &input.slots[second];
    let layers = |item: &ItemStack| {
        item.get_component::<BannerPatternLayers>(&key)
            .unwrap_or_default()
    };
    if first_banner.is_some() && first_banner == second_banner {
        let original = match (layers(a).layers.len(), layers(b).layers.len()) {
            (1..=6, 0) => first,
            (0, 1..=6) => second,
            _ => return None,
        };
        let output = input.slots[original].copy_with_count(1);
        return Some(Crafted {
            remainders: vec![(original, output.clone())],
            output,
            expand_map: false,
        });
    }
    let (banner, color, shield) = match (first_banner, second_banner, names) {
        (Some(color), None, [_, "shield"]) => (a, color, b),
        (None, Some(color), ["shield", _]) => (b, color, a),
        _ => return None,
    };
    let base = Identifier::minecraft("base_color");
    if shield.has_component(&base) {
        return None;
    }
    let mut output = shield.copy_with_count(1);
    output.set_component(&base, Some(color));
    output.set_component(&key, Some(layers(banner)));
    Some(input.remainder(output))
}
