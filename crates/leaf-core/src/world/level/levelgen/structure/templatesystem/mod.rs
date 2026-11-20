//! Перенос `net.minecraft.world.level.levelgen.structure.templatesystem`.

pub mod liquid_settings;
pub mod pos_rule_tests;
pub mod processor_rule;
pub mod processors;
pub mod rule_tests;
pub mod structure_place_settings;
pub mod structure_template;
pub mod structure_template_manager;

pub use liquid_settings::LiquidSettings;
pub use pos_rule_tests::{AxisAlignedLinearPosTest, LinearPosTest, PosAlwaysTrueTest, PosRuleTest};
pub use processor_rule::{Passthrough, ProcessorRule, RuleBlockEntityModifier};
pub use processors::*;
pub use rule_tests::*;
pub use structure_place_settings::StructurePlaceSettings;
pub use structure_template_manager::StructureTemplateManager;
pub use structure_template::{
    transform, transform_vec3, JigsawBlockInfo, Palette, SimplePalette, StructureBlockInfo,
    StructureEntityInfo, StructureTemplate,
};
