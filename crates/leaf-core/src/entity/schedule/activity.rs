//! Перенос `net.minecraft.world.entity.schedule.Activity` —
//! реестр `Registries.ACTIVITY`.

use crate::resources::identifier::Identifier;
use std::sync::{Arc, OnceLock, RwLock};

/// `Activity` — запись реестра `Registries.ACTIVITY`.
#[derive(Clone)]
pub struct Activity {
    /// Registry-ключ.
    pub id: Identifier,
    /// Порядковый id (Leaf `id` — размер реестра при регистрации).
    pub activity_id: i32,
}

impl Activity {
    fn new(name: &'static str, id: i32) -> Self {
        Self { id: Identifier::minecraft(name), activity_id: id }
    }

    /// `getName`.
    pub fn get_name(&self) -> &str {
        &self.id.path
    }
}

impl PartialEq for Activity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Activity {}
impl std::hash::Hash for Activity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
impl std::fmt::Debug for Activity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Activity").field(&self.id).finish()
    }
}

const NAMES: &[&str] = &[
    "core", "idle", "work", "play", "rest", "meet", "panic", "raid", "pre_raid", "hide",
    "fight", "celebrate", "admire_item", "avoid", "ride", "play_dead", "long_jump", "ram",
    "tongue", "swim", "lay_spawn", "sniff", "investigate", "roar", "emerge", "dig",
];

fn registry() -> &'static RwLock<Vec<Arc<Activity>>> {
    static REG: OnceLock<RwLock<Vec<Arc<Activity>>>> = OnceLock::new();
    REG.get_or_init(|| {
        RwLock::new(
            NAMES
                .iter()
                .enumerate()
                .map(|(i, name)| Arc::new(Activity::new(name, i as i32)))
                .collect(),
        )
    })
}

/// Все активности по registry-id.
pub fn activities() -> Vec<Arc<Activity>> {
    registry().read().unwrap().clone()
}

/// `Activity` по registry-id.
pub fn activity_by_id(id: i32) -> Option<Arc<Activity>> {
    registry().read().unwrap().get(id as usize).cloned()
}

/// `Activity` по имени.
pub fn activity_by_name(name: &str) -> Option<Arc<Activity>> {
    registry().read().unwrap().iter().find(|a| a.get_name() == name).cloned()
}
