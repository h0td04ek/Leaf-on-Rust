use std::sync::OnceLock;

use leaf_core::core::holder::Holder;
use leaf_core::network::protocol::game::clientbound_recipe_book_add_packet::{
    ClientboundRecipeBookAddPacket, Entry,
};
use leaf_core::resources::Identifier;
use leaf_core::world::item::component::suspicious_stew_effects::{
    SuspiciousStewEffects, SuspiciousStewEntry,
};
use leaf_core::world::item::crafting::display::{
    recipe_display::RecipeDisplay, recipe_display_entry::RecipeDisplayEntry,
    recipe_display_id::RecipeDisplayId, slot_display::SlotDisplay,
};
use leaf_core::world::item::crafting::{
    ingredient::Ingredient, recipe_book_category::RecipeBookCategory,
};
use leaf_core::world::item::item_stack::ItemStack;

use crate::items::{item_id, named_id, stack};
use crate::maps::Maps;
use crate::special_recipes;

pub struct Crafted {
    pub output: ItemStack,
    pub remainders: Vec<(usize, ItemStack)>,
    pub expand_map: bool,
}

pub struct Recipes {
    recipes: Vec<Recipe>,
    by_count: [Vec<usize>; 10],
    pub book: ClientboundRecipeBookAddPacket,
}

pub struct Recipe {
    kind: Kind,
    width: usize,
    height: usize,
    category: RecipeBookCategory,
    group: Option<i32>,
    ingredients: Vec<Vec<i32>>,
    output: ItemStack,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Pattern,
    Shapeless,
    Transmute,
}

impl Recipes {
    pub fn vanilla() -> &'static Recipes {
        static RECIPES: OnceLock<Recipes> = OnceLock::new();
        RECIPES.get_or_init(|| {
            let mut recipes = Vec::new();
            let mut by_count: [Vec<usize>; 10] = Default::default();
            let mut entries = Vec::new();
            for line in include_str!("../data/recipes-774.txt").lines().skip(1) {
                let mut fields = line.split_ascii_whitespace();
                let kind = match fields.next().unwrap() {
                    "pattern" => Kind::Pattern,
                    "shapeless" => Kind::Shapeless,
                    "transmute" => Kind::Transmute,
                    _ => unreachable!("invalid recipe kind"),
                };
                let width = fields.next().unwrap().parse().unwrap();
                let height = fields.next().unwrap().parse().unwrap();
                let category =
                    RecipeBookCategory::from_id(fields.next().unwrap().parse().unwrap()).unwrap();
                let group: i32 = fields.next().unwrap().parse().unwrap();
                let mut output = stack(
                    fields.next().unwrap().parse().unwrap(),
                    fields.next().unwrap().parse().unwrap(),
                );
                if let Some((effect, duration)) = fields.next().unwrap().split_once('/') {
                    output.set_component(
                        &Identifier::minecraft("suspicious_stew_effects"),
                        Some(SuspiciousStewEffects::new(vec![SuspiciousStewEntry::new(
                            Holder::Reference(effect.parse().unwrap()),
                            duration.parse().unwrap(),
                        )])),
                    );
                }
                let ingredients: Vec<Vec<i32>> = fields
                    .map(|cell| {
                        if cell == "-" {
                            vec![]
                        } else {
                            cell.split(',').map(|item| item.parse().unwrap()).collect()
                        }
                    })
                    .collect();
                let count = ingredients.iter().filter(|cell| !cell.is_empty()).count();
                let recipe = Recipe {
                    kind,
                    width,
                    height,
                    ingredients,
                    output,
                    category,
                    group: (group > 0).then_some(group - 1),
                };
                let id = recipes.len();
                by_count[count].push(id);
                entries.push(Entry::new(recipe.display_entry(id as i32), 0));
                recipes.push(recipe);
            }
            Recipes {
                recipes,
                by_count,
                book: ClientboundRecipeBookAddPacket::new(entries, true),
            }
        })
    }

    pub fn by_id(&self, id: usize) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    pub fn find(&self, grid: &[ItemStack], size: usize, maps: &Maps) -> Option<Crafted> {
        let input = Grid::new(grid, size)?;
        for &index in &self.by_count[input.filled.len()] {
            if let Some(crafted) = self.recipes[index].apply(&input) {
                return Some(crafted);
            }
        }
        special_recipes::craft(&input, maps)
    }
}

pub struct Grid<'a> {
    pub slots: &'a [ItemStack],
    pub filled: Vec<usize>,
    pub size: usize,
    left: usize,
    top: usize,
    pub width: usize,
    pub height: usize,
}

impl<'a> Grid<'a> {
    fn new(slots: &'a [ItemStack], size: usize) -> Option<Self> {
        let filled: Vec<_> = (0..size * size).filter(|&i| !slots[i].is_empty()).collect();
        let left = filled.iter().map(|&i| i % size).min()?;
        let right = filled.iter().map(|&i| i % size).max()?;
        let top = filled.first()? / size;
        let bottom = filled.last()? / size;
        Some(Self {
            slots,
            filled,
            size,
            left,
            top,
            width: right - left + 1,
            height: bottom - top + 1,
        })
    }

    pub fn remainder(&self, output: ItemStack) -> Crafted {
        let remainders = self
            .filled
            .iter()
            .filter_map(|&i| {
                let name = self.slots[i].item_identifier()?;
                let leftover = match name.get_path() {
                    "water_bucket" | "lava_bucket" | "milk_bucket" => "bucket",
                    "dragon_breath" | "honey_bottle" => "glass_bottle",
                    _ => return None,
                };
                Some((i, stack(named_id(leftover), 1)))
            })
            .collect();
        Crafted {
            output,
            remainders,
            expand_map: false,
        }
    }
}

impl Recipe {
    pub fn matches(&self, inventory: &[ItemStack], size: usize) -> bool {
        Grid::new(inventory, size)
            .filter(|input| {
                input.filled.len()
                    == self
                        .ingredients
                        .iter()
                        .filter(|cell| !cell.is_empty())
                        .count()
            })
            .is_some_and(|input| self.apply(&input).is_some())
    }

    fn apply(&self, input: &Grid<'_>) -> Option<Crafted> {
        if self.kind == Kind::Pattern {
            if input.width != self.width || input.height != self.height {
                return None;
            }
            let matches = |mirror: bool| {
                self.ingredients.iter().enumerate().all(|(i, cell)| {
                    let x = i % self.width;
                    let x = if mirror { self.width - 1 - x } else { x };
                    let index = (i / self.width + input.top) * input.size + input.left + x;
                    match item_id(&input.slots[index]) {
                        Some(id) => cell.binary_search(&id).is_ok(),
                        None => cell.is_empty(),
                    }
                })
            };
            return (matches(false) || matches(true)).then(|| input.remainder(self.output.clone()));
        }
        let source = input
            .filled
            .iter()
            .map(|&i| item_id(&input.slots[i]).unwrap())
            .collect::<Vec<_>>();
        let mut matched = vec![None; source.len()];
        for (index, _) in self.ingredients.iter().enumerate() {
            if !match_ingredient(
                &self.ingredients,
                &source,
                index,
                &mut matched,
                &mut [false; 9],
            ) {
                return None;
            }
        }
        let mut output = self.output.clone();
        if self.kind == Kind::Transmute {
            let first = matched.iter().position(|&i| i == Some(0)).unwrap();
            let original = &input.slots[input.filled[first]];
            output
                .components
                .apply_patch(&original.components.as_patch());
            if ItemStack::is_same_item_same_components(&output, original) {
                return None;
            }
        }
        Some(input.remainder(output))
    }

    pub fn grid_slots(&self, size: usize) -> Option<Vec<(usize, &[i32])>> {
        if self.width > size || self.height > size || self.ingredients.len() > size * size {
            return None;
        }
        let columns = if self.width == 0 { size } else { self.width };
        Some(
            self.ingredients
                .iter()
                .enumerate()
                .filter(|(_, c)| !c.is_empty())
                .map(|(i, cell)| (i / columns * size + i % columns, cell.as_slice()))
                .collect(),
        )
    }

    pub fn display(&self) -> RecipeDisplay {
        let station = SlotDisplay::Item(Holder::Reference(named_id("crafting_table")));
        let result = SlotDisplay::ItemStack(self.output.clone());
        let ingredients = self
            .ingredients
            .iter()
            .map(|cell| match cell.as_slice() {
                [] => SlotDisplay::Empty,
                [id] => SlotDisplay::Item(Holder::Reference(*id)),
                ids => SlotDisplay::Composite(
                    ids.iter()
                        .map(|&id| SlotDisplay::Item(Holder::Reference(id)))
                        .collect(),
                ),
            })
            .collect();
        if self.kind == Kind::Pattern {
            RecipeDisplay::ShapedCrafting {
                width: self.width as i32,
                height: self.height as i32,
                ingredients,
                result,
                crafting_station: station,
            }
        } else {
            RecipeDisplay::ShapelessCrafting {
                ingredients,
                result,
                crafting_station: station,
            }
        }
    }

    fn display_entry(&self, id: i32) -> RecipeDisplayEntry {
        let requirements = self
            .ingredients
            .iter()
            .filter(|cell| !cell.is_empty())
            .map(|cell| {
                Ingredient::of_items(cell.iter().map(|&id| Holder::Reference(id)).collect())
            })
            .collect();
        RecipeDisplayEntry::new(
            RecipeDisplayId::new(id),
            self.display(),
            self.group,
            self.category,
            Some(requirements),
        )
    }
}

fn match_ingredient(
    choices: &[Vec<i32>],
    items: &[i32],
    ingredient: usize,
    matched: &mut [Option<usize>],
    visited: &mut [bool],
) -> bool {
    for (i, id) in items.iter().enumerate() {
        if visited[i] || choices[ingredient].binary_search(id).is_err() {
            continue;
        }
        visited[i] = true;
        if matched[i].is_none_or(|old| match_ingredient(choices, items, old, matched, visited)) {
            matched[i] = Some(ingredient);
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::items::Items;

    #[test]
    fn catalog_has_every_data_driven_vanilla_grid_recipe() {
        Items::vanilla();
        let recipes = Recipes::vanilla();
        assert_eq!(recipes.recipes.len(), 706 + 320 + 32);
        let mut stew_recipes = 0;
        for recipe in &recipes.recipes {
            let mut grid = std::array::from_fn::<_, 9, _>(|_| ItemStack::empty());
            for (position, cells) in recipe.grid_slots(3).unwrap() {
                let item = cells
                    .iter()
                    .copied()
                    .find(|&id| {
                        recipe.kind != Kind::Transmute
                            || position != 0
                            || Some(id) != item_id(&recipe.output)
                    })
                    .unwrap();
                grid[position] = stack(item, 1);
            }
            assert!(
                recipe.matches(&grid, 3),
                "unmatchable recipe: {:?}",
                recipe.output
            );
            if recipe
                .output
                .has_component(&Identifier::minecraft("suspicious_stew_effects"))
            {
                let saved = recipe
                    .output
                    .save(leaf_core::nbt::CompoundTag::new())
                    .unwrap();
                let restored = ItemStack::from_nbt(saved.as_compound().unwrap()).unwrap();
                assert!(ItemStack::is_same_item_same_components(
                    &restored,
                    &recipe.output
                ));
                stew_recipes += 1;
            }
        }
        assert!(stew_recipes >= 10);
    }
}
