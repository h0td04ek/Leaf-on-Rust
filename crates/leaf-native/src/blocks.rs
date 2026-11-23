use std::collections::HashMap;
use std::sync::OnceLock;

use leaf_core::core::block_pos::BlockPos;
use leaf_core::core::direction::Direction;
use leaf_core::nbt::{CompoundTag, Tag};

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
    pub blast_resistance: f32,
    pub replaceable: bool,
    first: i32,
    properties: Vec<BlockProperty>,
}

struct BlockProperty {
    name: &'static str,
    values: Vec<&'static str>,
    stride: i32,
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
                let first: i32 = fields.next().unwrap().parse().unwrap();
                let index = blocks.kinds.len();
                assert_eq!(first as usize, blocks.states.len());
                let default = fields.next().unwrap().parse().unwrap();
                let last: usize = fields.next().unwrap().parse().unwrap();
                let item: i32 = fields.next().unwrap().parse().unwrap();
                let name = fields.next().unwrap();
                let wall = fields.next().unwrap();
                let hardness = fields.next().unwrap().parse().unwrap();
                let map_color = fields.next().unwrap().parse().unwrap();
                let replaceable = fields.next().unwrap() == "1";
                let mut properties = fields
                    .map(|field| {
                        let (name, values) = field.split_once('=').unwrap();
                        BlockProperty {
                            name,
                            values: values.split(',').collect(),
                            stride: 0,
                        }
                    })
                    .collect::<Vec<_>>();
                let mut stride = 1;
                for property in properties.iter_mut().rev() {
                    property.stride = stride;
                    stride *= property.values.len() as i32;
                }
                assert_eq!(first as usize + stride as usize, last + 1, "{name}");
                blocks.states.resize(last + 1, index);
                blocks.names.insert(name, index);
                blocks.kinds.push(Block {
                    name,
                    default,
                    item: (item != 0).then_some(item),
                    wall,
                    hardness,
                    map_color,
                    blast_resistance: 0.0,
                    replaceable,
                    first,
                    properties,
                });
                if item != 0 {
                    blocks.items.entry(item).or_insert(index);
                }
            }
            for row in include_str!("../data/blast-resistance-774.txt")
                .lines()
                .skip(1)
            {
                let (name, resistance) = row.split_once(' ').unwrap();
                let index = blocks.names[name];
                blocks.kinds[index].blast_resistance = resistance.parse().unwrap();
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

    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    pub fn decode_state(&self, tag: &CompoundTag) -> Option<i32> {
        let name = tag.get_string("Name")?.strip_prefix("minecraft:")?;
        let block = self.named(name)?;
        let mut state = block.default;
        if let Some(properties) = tag.get("Properties") {
            for (name, value) in properties.as_compound()?.iter() {
                let property = block.properties.iter().find(|p| p.name == name)?;
                state = property.with_value(block.first, state, value.as_string()?)?;
            }
        }
        Some(state)
    }

    pub fn encode_state(&self, state: i32) -> CompoundTag {
        let block = self.get(state);
        let mut tag = CompoundTag::new();
        tag.put_string("Name", &format!("minecraft:{}", block.name));
        if !block.properties.is_empty() {
            let mut properties = CompoundTag::new();
            for property in &block.properties {
                properties.put_string(property.name, property.value(block.first, state));
            }
            tag.put("Properties", Tag::Compound(properties));
        }
        tag
    }

    pub fn named(&self, name: &str) -> Option<&Block> {
        self.names.get(name).map(|&index| &self.kinds[index])
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.names.keys().copied()
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

impl BlockProperty {
    fn index(&self, first: i32, state: i32) -> i32 {
        (state - first) / self.stride % self.values.len() as i32
    }

    fn value(&self, first: i32, state: i32) -> &'static str {
        self.values[self.index(first, state) as usize]
    }

    fn with_value(&self, first: i32, state: i32, value: &str) -> Option<i32> {
        let index = self.values.iter().position(|&entry| entry == value)? as i32;
        Some(state + (index - self.index(first, state)) * self.stride)
    }
}

impl Block {
    pub fn state(&self, base: i32, overrides: &[(&str, &str)]) -> i32 {
        let mut state = base;
        for &(name, value) in overrides {
            if let Some(property) = self.properties.iter().find(|p| p.name == name) {
                let Some(next) = property.with_value(self.first, state, value) else {
                    return base;
                };
                state = next;
            }
        }
        state
    }

    pub fn property(&self, state: i32, key: &str) -> Option<&'static str> {
        self.properties
            .iter()
            .find(|property| property.name == key)
            .map(|property| property.value(self.first, state))
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
