use std::collections::HashMap;
use std::sync::OnceLock;

use leaf_core::core::block_pos::BlockPos;
use leaf_core::core::direction::Direction;

pub const GLOBAL_BITS: usize = 15;

pub struct Blocks {
    kinds: Vec<Block>,
    states: Vec<usize>,
    items: HashMap<i32, usize>,
    names: HashMap<&'static str, usize>,
}

pub struct Block {
    pub name: &'static str,
    pub default: i32,
    pub item: Option<i32>,
    wall: &'static str,
    pub hardness: i8,
    pub map_color: u8,
    pub replaceable: bool,
    variants: Vec<(i32, Vec<(&'static str, &'static str)>)>,
}

impl Blocks {
    pub fn vanilla() -> &'static Self {
        static BLOCKS: OnceLock<Blocks> = OnceLock::new();
        BLOCKS.get_or_init(|| {
            let mut blocks = Self {
                kinds: vec![],
                states: vec![],
                items: HashMap::new(),
                names: HashMap::new(),
            };
            for line in include_str!("../data/blocks-774.txt").lines().skip(1) {
                let mut fields = line.split_ascii_whitespace();
                let first = fields.next().unwrap();
                if let Some(state) = first.strip_prefix('+') {
                    blocks.kinds.last_mut().unwrap().variants.push((
                        state.parse().unwrap(),
                        fields.map(|p| p.split_once('=').unwrap()).collect(),
                    ));
                    continue;
                }
                let index = blocks.kinds.len();
                assert_eq!(first.parse::<usize>().unwrap(), blocks.states.len());
                let default = fields.next().unwrap().parse().unwrap();
                let last: usize = fields.next().unwrap().parse().unwrap();
                let item: i32 = fields.next().unwrap().parse().unwrap();
                let name = fields.next().unwrap();
                let wall = fields.next().unwrap();
                let hardness = fields.next().unwrap().parse().unwrap();
                let map_color = fields.next().unwrap().parse().unwrap();
                let replaceable = fields.next().unwrap() == "1";
                blocks.states.resize(last + 1, index);
                blocks.names.insert(name, index);
                blocks.kinds.push(Block {
                    name,
                    default,
                    item: (item != 0).then_some(item),
                    wall,
                    hardness,
                    map_color,
                    replaceable,
                    variants: vec![],
                });
                if item != 0 {
                    blocks.items.entry(item).or_insert(index);
                }
            }
            // Prefer the standing item variant even if the wall state has a lower ID.
            for block in &blocks.kinds {
                if block.wall != "-" {
                    blocks
                        .items
                        .insert(block.item.unwrap(), blocks.names[block.name]);
                }
            }
            assert_eq!(
                usize::BITS as usize - (blocks.states.len() - 1).leading_zeros() as usize,
                GLOBAL_BITS
            );
            blocks
        })
    }

    pub fn get(&self, state: i32) -> &Block {
        &self.kinds[self.states[state as usize]]
    }

    pub fn for_item(&self, item: i32, face: Direction) -> Option<&Block> {
        let block = &self.kinds[*self.items.get(&item)?];
        if matches!(
            face,
            Direction::NORTH | Direction::EAST | Direction::SOUTH | Direction::WEST
        ) {
            if let Some(index) = self.names.get(block.wall) {
                return Some(&self.kinds[*index]);
            }
        }
        Some(block)
    }
}

impl Block {
    pub fn state(&self, base: i32, overrides: &[(&str, &str)]) -> i32 {
        let Some((_, properties)) = self.variants.iter().find(|(state, _)| *state == base) else {
            return base;
        };
        self.variants
            .iter()
            .find(|(_, variant)| {
                properties
                    .iter()
                    .zip(variant)
                    .all(|((key, original), (_, value))| {
                        overrides
                            .iter()
                            .find(|(k, _)| k == key)
                            .map_or(original, |(_, updated)| updated)
                            == value
                    })
            })
            .map_or(base, |(state, _)| *state)
    }

    pub fn property(&self, state: i32, key: &str) -> Option<&'static str> {
        self.variants
            .iter()
            .find(|(id, _)| *id == state)?
            .1
            .iter()
            .find_map(|&(property, value)| (property == key).then_some(value))
    }

    pub fn door(&self) -> bool {
        self.name.ends_with("_door")
    }

    pub fn paired(
        &self,
        pos: BlockPos,
        state: i32,
    ) -> Option<(BlockPos, &'static str, &'static str)> {
        if self.door()
            || matches!(
                self.name,
                "sunflower"
                    | "lilac"
                    | "rose_bush"
                    | "peony"
                    | "tall_grass"
                    | "large_fern"
                    | "pitcher_plant"
                    | "tall_dry_grass"
            )
        {
            return match self.property(state, "half")? {
                "lower" => Some((pos.offset(0, 1, 0), "half", "upper")),
                _ => Some((pos.offset(0, -1, 0), "half", "lower")),
            };
        }
        if self.name.ends_with("_bed") {
            let direction = Direction::by_name(self.property(state, "facing")?)?;
            return if self.property(state, "part")? == "foot" {
                Some((pos.relative(direction), "part", "head"))
            } else {
                Some((pos.relative(direction.get_opposite()), "part", "foot"))
            };
        }
        None
    }
}

pub fn placement_state(block: &Block, face: Direction, yaw: f32, hit_y: f32) -> i32 {
    let toward = Direction::from_y_rot(f64::from(yaw));
    let facing = if block.name.ends_with("anvil") {
        toward.get_clock_wise()
    } else if block.name.contains("wall_") {
        face
    } else if block.door() || block.name.ends_with("_bed") || block.name.ends_with("_fence_gate") {
        toward
    } else if block.property(block.default, "facing") == Some("up") {
        face
    } else {
        toward.get_opposite()
    };
    let axis = match face {
        Direction::UP | Direction::DOWN => "y",
        Direction::EAST | Direction::WEST => "x",
        _ => "z",
    };
    let half = if face == Direction::DOWN || face != Direction::UP && hit_y > 0.5 {
        "top"
    } else {
        "bottom"
    };
    let rotation = ((f64::from(yaw) * 16.0 / 360.0).round() as i32 & 15).to_string();
    let mut overrides = vec![
        ("facing", facing.get_serialized_name()),
        ("axis", axis),
        ("rotation", &rotation),
    ];
    if matches!(
        block.property(block.default, "half"),
        Some("top" | "bottom")
    ) {
        overrides.push(("half", half));
    }
    if block.property(block.default, "face").is_some() {
        let surface = match face {
            Direction::UP => "floor",
            Direction::DOWN => "ceiling",
            _ => "wall",
        };
        overrides.push(("face", surface));
        if surface == "wall" {
            overrides[0].1 = face.get_serialized_name();
        }
    }
    if block.property(block.default, "type") == Some("bottom") {
        overrides.push(("type", half));
    }
    block.state(block.default, &overrides)
}
