//! `net.minecraft.world.entity.npc.villager` — житель и его данные.

pub mod villager_data;
pub mod villager_profession;
pub mod villager_type;
pub mod villager;

pub use villager_data::VillagerData;
pub use villager_profession::VillagerProfession;
pub use villager_type::VillagerType;
pub use villager::{init_villager, is_villager, poi_memories};
