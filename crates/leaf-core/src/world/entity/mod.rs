//! `net.minecraft.world.entity` — сущности.

pub mod ai;
pub mod entity;
pub mod entity_type;
pub mod mob_category;
pub mod pose;

pub use entity::Entity;
pub use entity_type::EntityType;
pub use pose::Pose;
pub mod living_entity;
pub mod item_entity;
pub mod experience_orb;
pub use living_entity::LivingEntity;
