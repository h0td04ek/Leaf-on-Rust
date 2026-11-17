//! Перенос `net.minecraft.world.level.block.entity` — типы данных
//! для wire-формата (полные BE — с портом level).

pub mod command_block_entity;
pub mod jigsaw_block_entity;
pub mod structure_block_entity;
pub mod test_instance_block_entity;
pub use command_block_entity::CommandBlockEntity;
pub use jigsaw_block_entity::JigsawBlockEntity;
pub use structure_block_entity::StructureBlockEntity;
pub use test_instance_block_entity::TestInstanceBlockEntity;
pub mod block_entity_type;
pub use block_entity_type::BlockEntityType;
pub mod banner_pattern;
pub mod banner_pattern_layers;
pub mod pot_decorations;
pub mod beehive_block_entity;
pub mod block_entity;
