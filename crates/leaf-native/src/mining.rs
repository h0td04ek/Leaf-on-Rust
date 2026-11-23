use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use leaf_core::core::component::data_components::enchantment_id;
use leaf_core::core::holder::Holder;
use leaf_core::random::RandomSource;
use leaf_core::resources::Identifier;
use leaf_core::world::item::{
    enchantment::item_enchantments::ItemEnchantments, item_stack::ItemStack,
};

use crate::blocks::Block;
use crate::items::{item_id, named_id};

pub struct Miner {
    speed: f32,
    correct: bool,
    pub can_break_in_creative: bool,
    pub damage: i32,
    silk: bool,
    fortune: i32,
    shears: bool,
}

impl Miner {
    pub fn new(tool: &ItemStack, block: &Block) -> Self {
        let data = Mining::vanilla();
        let stats = &data.blocks[&block.default];
        let tool_id = item_id(tool).unwrap_or(0);
        let mut speed = 1.0;
        let mut correct = !stats.requires_tool;
        let mut can_break_in_creative = true;
        let mut damage = 0;
        if let Some(tool) = data.tools.get(&tool_id) {
            let matching = || {
                tool.rules
                    .iter()
                    .filter(|rule| data.sets[rule.set].contains(&block.default))
            };
            speed = matching().find_map(|rule| rule.speed).unwrap_or(1.0);
            correct |= matching().find_map(|rule| rule.correct).unwrap_or(false);
            can_break_in_creative = tool.creative;
            damage = tool.damage;
        }
        let shears = tool_id == named_id("shears");
        if shears {
            // These blocks use the ShearsItem override rather than its Tool component.
            if block.name.ends_with("_leaves")
                || matches!(
                    block.name,
                    "cobweb"
                        | "short_grass"
                        | "fern"
                        | "dead_bush"
                        | "hanging_roots"
                        | "vine"
                        | "tripwire"
                        | "glow_lichen"
                )
            {
                damage = 1;
            } else {
                damage = 0;
            }
            correct |= matches!(block.name, "cobweb" | "redstone_wire" | "tripwire");
        }
        if speed > 1.0 {
            let efficiency = enchantment_level(tool, "efficiency");
            if efficiency != 0 {
                speed += (efficiency * efficiency + 1) as f32;
            }
        }
        Self {
            speed,
            correct,
            can_break_in_creative,
            damage,
            silk: enchantment_level(tool, "silk_touch") != 0,
            fortune: enchantment_level(tool, "fortune"),
            shears,
        }
    }

    pub fn progress(&self, block: &Block, on_ground: bool, underwater: bool) -> f32 {
        let hardness = Mining::vanilla().blocks[&block.default].hardness;
        if hardness < 0.0 {
            return 0.0;
        }
        if hardness == 0.0 {
            return 1.0;
        }
        let mut speed = self.speed;
        if !on_ground {
            speed /= 5.0;
        }
        if underwater {
            speed /= 5.0;
        }
        speed / hardness / if self.correct { 30.0 } else { 100.0 }
    }

    pub fn drops(
        &self,
        block: &Block,
        state: i32,
        random: &mut impl RandomSource,
    ) -> Vec<(i32, i32)> {
        if !self.correct {
            return Vec::new();
        }
        match block.name {
            "short_grass" | "tall_grass" | "fern" | "large_fern" => {
                let (item, count) = match block.name {
                    "tall_grass" => ("short_grass", 2),
                    "large_fern" => ("fern", 2),
                    name => (name, 1),
                };
                if self.shears {
                    return vec![(named_id(item), count)];
                }
                if random.next_float() >= 0.125 {
                    return Vec::new();
                }
                let bonus = if count == 1 {
                    random.next_int_bound(self.fortune * 2 + 1)
                } else {
                    0
                };
                return vec![(named_id("wheat_seeds"), 1 + bonus)];
            }
            "short_dry_grass" | "tall_dry_grass" => {
                return if self.shears {
                    vec![(named_id(block.name), 1)]
                } else {
                    Vec::new()
                };
            }
            _ => {}
        }
        if let Some(tree) = block.name.strip_suffix("_leaves") {
            return self.leaf_drops(block, tree, random);
        }
        let stats = &Mining::vanilla().blocks[&block.default];
        match stats.loot[usize::from(self.silk)] {
            Loot::Empty => Vec::new(),
            Loot::Item {
                item,
                minimum,
                maximum,
                bonus,
            } => {
                let count = minimum
                    + if minimum == maximum {
                        0
                    } else {
                        random.next_int_bound(maximum - minimum + 1)
                    };
                vec![(item, bonus.apply(count, self.fortune, random))]
            }
            Loot::Fallback => {
                let count = if block.name.ends_with("_slab")
                    && block.property(state, "type") == Some("double")
                {
                    2
                } else {
                    1
                };
                block.item.into_iter().map(|item| (item, count)).collect()
            }
        }
    }

    fn leaf_drops(
        &self,
        block: &Block,
        tree: &str,
        random: &mut impl RandomSource,
    ) -> Vec<(i32, i32)> {
        if self.shears || self.silk {
            return block.item.into_iter().map(|item| (item, 1)).collect();
        }
        let fortune = self.fortune as usize;
        let mut drops = Vec::new();
        let saplings: &[f32] = if tree == "jungle" {
            &[0.025, 0.027777778, 0.03125, 0.041666668, 0.1]
        } else {
            &[0.05, 0.0625, 0.083333336, 0.1]
        };
        if tree != "mangrove" && random.next_float() < saplings[fortune.min(saplings.len() - 1)] {
            let sapling = match tree {
                "azalea" | "flowering_azalea" => tree.to_owned(),
                _ => format!("{tree}_sapling"),
            };
            drops.push((named_id(&sapling), 1));
        }
        const STICKS: [f32; 5] = [0.02, 0.022222223, 0.025, 0.033333335, 0.1];
        const APPLES: [f32; 5] = [0.005, 0.005555556, 0.00625, 0.008333334, 0.025];
        if random.next_float() < STICKS[fortune.min(4)] {
            drops.push((named_id("stick"), 1 + random.next_int_bound(2)));
        }
        if matches!(tree, "oak" | "dark_oak") && random.next_float() < APPLES[fortune.min(4)] {
            drops.push((named_id("apple"), 1));
        }
        drops
    }
}

pub fn enchantment_level(stack: &ItemStack, name: &str) -> i32 {
    let key = Identifier::minecraft(name);
    let Some(enchantments) =
        stack.get_component::<ItemEnchantments>(&Identifier::minecraft("enchantments"))
    else {
        return 0;
    };
    let id = enchantment_id(&key);
    enchantments
        .enchantments
        .iter()
        .find_map(|(holder, level)| {
            match holder {
                Holder::Reference(reference) => id == Some(*reference),
                Holder::Key(holder) => holder.id() == &key,
                Holder::Direct(_) => false,
            }
            .then_some(*level)
        })
        .unwrap_or(0)
}

#[derive(Clone, Copy)]
enum Bonus {
    None,
    Ore,
    Uniform(i32),
    Binomial(i32, f32),
}

impl Bonus {
    fn apply(self, count: i32, fortune: i32, random: &mut impl RandomSource) -> i32 {
        match self {
            Self::None => count,
            Self::Ore if fortune == 0 => count,
            Self::Ore => {
                count
                    * (random.next_int_bound(fortune + 2) - 1)
                        .max(0)
                        .saturating_add(1)
            }
            Self::Uniform(multiplier) => count + random.next_int_bound(multiplier * fortune + 1),
            Self::Binomial(extra, probability) => {
                count
                    + (0..fortune + extra)
                        .filter(|_| random.next_float() < probability)
                        .count() as i32
            }
        }
    }
}

#[derive(Clone, Copy)]
enum Loot {
    Empty,
    Fallback,
    Item {
        item: i32,
        minimum: i32,
        maximum: i32,
        bonus: Bonus,
    },
}

impl Loot {
    fn parse(value: &str) -> Self {
        match value {
            "-" => Self::Empty,
            "?" => Self::Fallback,
            value => {
                let mut fields = value.split(':');
                let item = fields.next().unwrap().parse().unwrap();
                let minimum = fields.next().unwrap().parse().unwrap();
                let maximum = fields.next().unwrap().parse().unwrap();
                let mut bonus = fields.next().unwrap().split('_');
                let bonus = match bonus.next().unwrap() {
                    "-" => Bonus::None,
                    "ore" => Bonus::Ore,
                    "uniform" => Bonus::Uniform(bonus.next().unwrap().parse().unwrap()),
                    "binomial" => Bonus::Binomial(
                        bonus.next().unwrap().parse().unwrap(),
                        bonus.next().unwrap().parse().unwrap(),
                    ),
                    name => panic!("unknown fortune bonus {name}"),
                };
                Self::Item {
                    item,
                    minimum,
                    maximum,
                    bonus,
                }
            }
        }
    }
}

struct BlockMining {
    hardness: f32,
    requires_tool: bool,
    loot: [Loot; 2],
}

struct Tool {
    damage: i32,
    creative: bool,
    rules: Vec<ToolRule>,
}

struct ToolRule {
    set: usize,
    speed: Option<f32>,
    correct: Option<bool>,
}

pub struct Mining {
    blocks: HashMap<i32, BlockMining>,
    tools: HashMap<i32, Tool>,
    sets: Vec<HashSet<i32>>,
}

impl Mining {
    pub fn vanilla() -> &'static Self {
        static REGISTRY: OnceLock<Mining> = OnceLock::new();
        REGISTRY.get_or_init(|| {
            let mut registry = Mining {
                blocks: HashMap::new(),
                tools: HashMap::new(),
                sets: Vec::new(),
            };
            let mut last_tool = 0;
            for row in include_str!("../data/mining-774.txt").lines().skip(1) {
                let mut fields = row.split_ascii_whitespace();
                let kind = fields.next().unwrap();
                let id: i32 = fields.next().unwrap().parse().unwrap();
                match kind {
                    "B" => {
                        let hardness = fields.next().unwrap().parse().unwrap();
                        let requires_tool = fields.next().unwrap() == "1";
                        let loot = std::array::from_fn(|_| Loot::parse(fields.next().unwrap()));
                        registry.blocks.insert(
                            id,
                            BlockMining {
                                hardness,
                                requires_tool,
                                loot,
                            },
                        );
                    }
                    "T" => {
                        let damage = fields.next().unwrap().parse().unwrap();
                        let creative = fields.next().unwrap() == "1";
                        registry.tools.insert(
                            id,
                            Tool {
                                damage,
                                creative,
                                rules: Vec::new(),
                            },
                        );
                        last_tool = id;
                    }
                    "R" => {
                        let speed = fields.next().unwrap().parse().ok();
                        let correct = match fields.next().unwrap() {
                            "0" => Some(false),
                            "1" => Some(true),
                            _ => None,
                        };
                        registry
                            .tools
                            .get_mut(&last_tool)
                            .unwrap()
                            .rules
                            .push(ToolRule {
                                set: id as usize,
                                speed,
                                correct,
                            });
                    }
                    "S" => {
                        assert_eq!(registry.sets.len(), id as usize);
                        registry
                            .sets
                            .push(fields.map(|block| block.parse().unwrap()).collect());
                    }
                    _ => panic!("unknown block mining profile row {kind}"),
                }
            }
            registry
        })
    }

    pub fn hardness(&self, block: &Block) -> f32 {
        self.blocks[&block.default].hardness
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::Blocks;
    use crate::items::{stack, Items};
    use leaf_core::random::LegacyRandomSource;

    #[test]
    fn uses_mining_tools_and_keeps_enchantments_in_saved_items() {
        Items::vanilla();
        let hand = ItemStack::empty();
        let stone = Blocks::vanilla().named("stone").unwrap();
        let pickaxe = stack(named_id("iron_pickaxe"), 1);
        let mut random = LegacyRandomSource::new(9);
        let bare = Miner::new(&hand, stone);
        let effective = Miner::new(&pickaxe, stone);
        assert!(effective.progress(stone, true, false) > bare.progress(stone, true, false) * 10.0);
        assert!(bare.drops(stone, stone.default, &mut random).is_empty());
        assert_eq!(
            effective.drops(stone, stone.default, &mut random),
            [(named_id("cobblestone"), 1)]
        );

        let mut pickaxe = pickaxe;
        let key = Identifier::minecraft("silk_touch");
        pickaxe.set_component(
            &Identifier::minecraft("enchantments"),
            Some(ItemEnchantments::new(vec![(
                Holder::Reference(enchantment_id(&key).unwrap()),
                1,
            )])),
        );
        let saved = pickaxe.save(leaf_core::nbt::CompoundTag::new()).unwrap();
        let pickaxe = ItemStack::from_nbt(saved.as_compound().unwrap()).unwrap();
        assert_eq!(enchantment_level(&pickaxe, "silk_touch"), 1);
        assert_eq!(
            Miner::new(&pickaxe, stone).drops(stone, stone.default, &mut random),
            [(named_id("stone"), 1)]
        );

        let grass = Blocks::vanilla().named("short_grass").unwrap();
        let miner = Miner::new(&hand, grass);
        assert_eq!(miner.progress(grass, true, false), 1.0);
        let seeds = named_id("wheat_seeds");
        let loot = (0..100)
            .flat_map(|_| miner.drops(grass, grass.default, &mut random))
            .collect::<Vec<_>>();
        assert!(!loot.is_empty());
        assert!(loot
            .iter()
            .all(|&(item, count)| item == seeds && count == 1));
    }
}
