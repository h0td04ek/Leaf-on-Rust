//! Перенос `net.minecraft.advancements`.

pub mod advancement;
pub mod advancement_holder;
pub mod advancement_progress;
pub mod advancement_requirements;
pub mod advancement_rewards;
pub mod advancement_type;
pub mod criterion;
pub mod criterion_progress;
pub mod display_info;

pub use advancement::Advancement;
pub use advancement_holder::AdvancementHolder;
pub use advancement_progress::AdvancementProgress;
pub use advancement_requirements::AdvancementRequirements;
pub use advancement_rewards::AdvancementRewards;
pub use advancement_type::AdvancementType;
pub use criterion_progress::CriterionProgress;
pub use display_info::DisplayInfo;
