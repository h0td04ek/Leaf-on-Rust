//! `net.minecraft.world.level.gamerules.GameRuleCategory`.

use std::sync::{Mutex, OnceLock};

use crate::resources::identifier::Identifier;

/// `GameRuleCategory` — категория правил (record(Identifier)).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GameRuleCategory {
    /// `id`.
    pub id: Identifier,
}

impl GameRuleCategory {
    /// `new GameRuleCategory(id)` + регистрация в SORT_ORDER.
    pub fn register(id: Identifier) -> Self {
        let cat = Self { id };
        let mut order = sort_order().lock().unwrap();
        if order.contains(&cat) {
            panic!("Category '{}' is already registered.", cat.id);
        }
        order.push(cat.clone());
        cat
    }

    /// `register(name)` — `Identifier.withDefaultNamespace`.
    pub fn register_named(name: &str) -> Self {
        Self::register(Identifier::minecraft(name))
    }

    /// `getDescriptionId()`.
    pub fn get_description_id(&self) -> &Identifier {
        &self.id
    }

    /// `SORT_ORDER` — снимок порядка регистрации.
    pub fn sort_order() -> Vec<GameRuleCategory> {
        sort_order().lock().unwrap().clone()
    }
}

fn sort_order() -> &'static Mutex<Vec<GameRuleCategory>> {
    static ORDER: OnceLock<Mutex<Vec<GameRuleCategory>>> = OnceLock::new();
    ORDER.get_or_init(|| Mutex::new(Vec::new()))
}

/// Ленивые константы категорий (Java static fields).
macro_rules! category {
    ($name:ident, $field:literal) => {
        /// Java `GameRuleCategory::$name`.
        pub fn $name() -> GameRuleCategory {
            static CAT: OnceLock<GameRuleCategory> = OnceLock::new();
            CAT.get_or_init(|| GameRuleCategory::register_named($field)).clone()
        }
    };
}

category!(player, "player");
category!(mobs, "mobs");
category!(spawning, "spawning");
category!(drops, "drops");
category!(updates, "updates");
category!(chat, "chat");
category!(misc, "misc");
