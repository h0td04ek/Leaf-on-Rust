use std::collections::HashMap;
use std::sync::OnceLock;

use leaf_core::resources::Identifier;

pub struct MobType {
    pub id: i32,
    pub name: &'static str,
    pub health: f32,
    pub height: f64,
    pub width: f64,
    pub hostile: bool,
    pub flying: bool,
    pub aquatic: bool,
    pub fire_immune: bool,
}

impl MobType {
    pub fn is_pushed_by_fluid(&self) -> bool {
        !self.aquatic && !matches!(self.name, "axolotl" | "turtle" | "zombie_nautilus")
    }
}

struct Registry {
    names: HashMap<&'static str, MobType>,
    eggs: HashMap<i32, &'static str>,
}

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = Registry {
            names: HashMap::new(),
            eggs: HashMap::new(),
        };
        for row in include_str!("../data/entities-774.txt").lines().skip(1) {
            let mut fields = row.split_ascii_whitespace();
            let id = fields.next().unwrap().parse().unwrap();
            let name = fields.next().unwrap();
            let egg: i32 = fields.next().unwrap().parse().unwrap();
            let health = fields.next().unwrap().parse().unwrap();
            let height = fields.next().unwrap().parse().unwrap();
            let width = fields.next().unwrap().parse().unwrap();
            let category = fields.next().unwrap();
            let fire_immune = fields.next().unwrap() == "1";
            let flying = matches!(
                name,
                "allay"
                    | "bat"
                    | "bee"
                    | "blaze"
                    | "breeze"
                    | "ender_dragon"
                    | "ghast"
                    | "happy_ghast"
                    | "parrot"
                    | "phantom"
                    | "vex"
                    | "wither"
            );
            let aquatic =
                category.contains("water") || matches!(name, "squid" | "glow_squid" | "tadpole");
            registry.names.insert(
                name,
                MobType {
                    id,
                    name,
                    health,
                    height,
                    width,
                    hostile: category == "monster",
                    flying,
                    aquatic,
                    fire_immune,
                },
            );
            if egg >= 0 {
                registry.eggs.insert(egg, name);
            }
        }
        registry
    })
}

pub fn named(name: &str) -> Option<&'static MobType> {
    registry().names.get(name)
}

pub fn get(id: &Identifier) -> Option<&'static MobType> {
    (id.get_namespace() == "minecraft")
        .then(|| named(id.get_path()))
        .flatten()
}

pub fn from_egg(item: i32) -> Option<&'static MobType> {
    named(registry().eggs.get(&item)?)
}

pub fn names() -> impl Iterator<Item = &'static str> {
    registry().names.keys().copied()
}
