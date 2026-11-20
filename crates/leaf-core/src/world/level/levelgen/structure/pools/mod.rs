//! `net.minecraft.world.level.levelgen.structure.pools` —
//! Jigsaw-пулы (StructureTemplatePool/PoolElement/JigsawPlacement).

pub mod empty_pool_element;
pub mod feature_pool_element;
pub mod jigsaw_placement;
pub mod legacy_single_pool_element;
pub mod list_pool_element;
pub mod pool_alias;
pub mod single_pool_element;
pub mod structure_pool_element;
pub mod structure_template_pool;

pub use empty_pool_element::{empty_element_rc, EmptyPoolElement};
pub use feature_pool_element::FeaturePoolElement;
pub use jigsaw_placement::{
    add_pieces, run_placer, DimensionPadding, EmptyPoolAliasLookup, JigsawContext,
    JigsawMaxDistance, PoolAliasLookup, PoolRegistryAccess, RcPoolPiece,
    template_pool_registry_key,
};
pub use legacy_single_pool_element::LegacySinglePoolElement;
pub use list_pool_element::ListPoolElement;
pub use single_pool_element::{single, SinglePoolElement, TemplateSource};
pub use structure_pool_element::{PoolGeneratorAccess, StructurePoolElement};
pub use structure_template_pool::{
    JigsawJunction, PoolElementWeight, Projection, StructureTemplatePool,
};
