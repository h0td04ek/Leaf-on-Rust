//! Перенос `net.minecraft.world.level.storage.loot.parameters.LootContextParams`.

use crate::resources::identifier::Identifier;
use crate::util::context::context_key::ContextKey;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::damagesource::damage_source::DamageSource;
use crate::world::entity::entity::Entity;
use crate::world::item::item_stack::ItemStack;
use crate::world::phys::vec3::Vec3;

/// `LootContextParams` — все `ContextKey` параметров.
pub mod params {
    use super::*;

    /// `THIS_ENTITY`.
    pub fn this_entity() -> &'static ContextKey<Entity> {
        static K: std::sync::OnceLock<ContextKey<Entity>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("this_entity"))
    }
    /// `INTERACTING_ENTITY`.
    pub fn interacting_entity() -> &'static ContextKey<Entity> {
        static K: std::sync::OnceLock<ContextKey<Entity>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("interacting_entity"))
    }
    /// `TARGET_ENTITY`.
    pub fn target_entity() -> &'static ContextKey<Entity> {
        static K: std::sync::OnceLock<ContextKey<Entity>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("target_entity"))
    }
    /// `LAST_DAMAGE_PLAYER` — Player (здесь Entity).
    pub fn last_damage_player() -> &'static ContextKey<Entity> {
        static K: std::sync::OnceLock<ContextKey<Entity>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("last_damage_player"))
    }
    /// `DAMAGE_SOURCE`.
    pub fn damage_source() -> &'static ContextKey<DamageSource> {
        static K: std::sync::OnceLock<ContextKey<DamageSource>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("damage_source"))
    }
    /// `ATTACKING_ENTITY`.
    pub fn attacking_entity() -> &'static ContextKey<Entity> {
        static K: std::sync::OnceLock<ContextKey<Entity>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("attacking_entity"))
    }
    /// `DIRECT_ATTACKING_ENTITY`.
    pub fn direct_attacking_entity() -> &'static ContextKey<Entity> {
        static K: std::sync::OnceLock<ContextKey<Entity>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("direct_attacking_entity"))
    }
    /// `ORIGIN`.
    pub fn origin() -> &'static ContextKey<Vec3> {
        static K: std::sync::OnceLock<ContextKey<Vec3>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("origin"))
    }
    /// `BLOCK_STATE`.
    pub fn block_state() -> &'static ContextKey<BlockState> {
        static K: std::sync::OnceLock<ContextKey<BlockState>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("block_state"))
    }
    /// `TOOL`.
    pub fn tool() -> &'static ContextKey<ItemStack> {
        static K: std::sync::OnceLock<ContextKey<ItemStack>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("tool"))
    }
    /// `EXPLOSION_RADIUS`.
    pub fn explosion_radius() -> &'static ContextKey<f32> {
        static K: std::sync::OnceLock<ContextKey<f32>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("explosion_radius"))
    }
    /// `ENCHANTMENT_LEVEL`.
    pub fn enchantment_level() -> &'static ContextKey<i32> {
        static K: std::sync::OnceLock<ContextKey<i32>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("enchantment_level"))
    }
    /// `ENCHANTMENT_ACTIVE`.
    pub fn enchantment_active() -> &'static ContextKey<bool> {
        static K: std::sync::OnceLock<ContextKey<bool>> = std::sync::OnceLock::new();
        K.get_or_init(|| ContextKey::vanilla("enchantment_active"))
    }
}

pub use params::*;

/// `LootContextParams`-поля для NBT/отладки.
pub fn all_param_names() -> Vec<Identifier> {
    vec![
        Identifier::minecraft("this_entity"),
        Identifier::minecraft("interacting_entity"),
        Identifier::minecraft("target_entity"),
        Identifier::minecraft("last_damage_player"),
        Identifier::minecraft("damage_source"),
        Identifier::minecraft("attacking_entity"),
        Identifier::minecraft("direct_attacking_entity"),
        Identifier::minecraft("origin"),
        Identifier::minecraft("block_state"),
        Identifier::minecraft("tool"),
        Identifier::minecraft("explosion_radius"),
        Identifier::minecraft("enchantment_level"),
        Identifier::minecraft("enchantment_active"),
    ]
}
