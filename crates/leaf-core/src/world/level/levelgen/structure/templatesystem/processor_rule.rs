//! Перенос `ProcessorRule` + `RuleBlockEntityModifier`/`Passthrough`.

use crate::core::block_pos::BlockPos;
use crate::nbt::compound::CompoundTag;
use crate::random::RandomSource;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::levelgen::structure::templatesystem::pos_rule_tests::{
    PosAlwaysTrueTest, PosRuleTest,
};
use crate::world::level::levelgen::structure::templatesystem::rule_tests::RuleTest;
use std::rc::Rc;

/// `RuleBlockEntityModifier` — модификатор NBT block entity
/// (аналог Java `rule.blockentity.RuleBlockEntityModifier`).
pub trait RuleBlockEntityModifier: Send + Sync {
    /// `apply(random, tag)` → Option<CompoundTag>.
    fn apply(
        &self,
        random: &mut dyn RandomSource,
        tag: Option<&CompoundTag>,
    ) -> Option<CompoundTag>;
}

/// `Passthrough` — DEFAULT_BLOCK_ENTITY_MODIFIER, возвращает tag.
pub struct Passthrough;

impl RuleBlockEntityModifier for Passthrough {
    fn apply(
        &self,
        _random: &mut dyn RandomSource,
        tag: Option<&CompoundTag>,
    ) -> Option<CompoundTag> {
        tag.cloned()
    }
}

/// `AppendLoot` — `rule.blockentity.AppendLoot`: добавляет
/// `LootTable`/`LootTableSeed` в NBT block entity.
pub struct AppendLoot {
    /// `lootTable` — `ResourceKey<LootTable>` (храним Identifier).
    pub loot_table: crate::resources::identifier::Identifier,
}

impl AppendLoot {
    /// `new(lootTable)`.
    pub fn new(loot_table: crate::resources::identifier::Identifier) -> Self {
        Self { loot_table }
    }
}

impl RuleBlockEntityModifier for AppendLoot {
    fn apply(
        &self,
        random: &mut dyn RandomSource,
        tag: Option<&CompoundTag>,
    ) -> Option<CompoundTag> {
        let mut tag = tag.cloned().unwrap_or_default();
        tag.put(
            "LootTable",
            crate::nbt::tag::Tag::String(self.loot_table.to_string()),
        );
        tag.put(
            "LootTableSeed",
            crate::nbt::tag::Tag::Long(random.next_long()),
        );
        Some(tag)
    }
}

/// `ProcessorRule` — правило замены в `RuleProcessor`.
pub struct ProcessorRule {
    /// `inputPredicate`.
    pub input_predicate: Rc<dyn RuleTest>,
    /// `locPredicate`.
    pub loc_predicate: Rc<dyn RuleTest>,
    /// `posPredicate`.
    pub pos_predicate: Rc<dyn PosRuleTest>,
    /// `outputState`.
    pub output_state: BlockState,
    /// `blockEntityModifier`.
    pub block_entity_modifier: Rc<dyn RuleBlockEntityModifier>,
}

impl ProcessorRule {
    /// `new(input, loc, outputState)` — pos = AlwaysTrue,
    /// modifier = Passthrough.
    pub fn new(
        input_predicate: Rc<dyn RuleTest>,
        loc_predicate: Rc<dyn RuleTest>,
        output_state: BlockState,
    ) -> Self {
        Self {
            input_predicate,
            loc_predicate,
            pos_predicate: Rc::new(PosAlwaysTrueTest),
            output_state,
            block_entity_modifier: Rc::new(Passthrough),
        }
    }

    /// `new(input, loc, posPredicate, outputState)`.
    pub fn with_pos_predicate(
        input_predicate: Rc<dyn RuleTest>,
        loc_predicate: Rc<dyn RuleTest>,
        pos_predicate: Rc<dyn PosRuleTest>,
        output_state: BlockState,
    ) -> Self {
        Self {
            input_predicate,
            loc_predicate,
            pos_predicate,
            output_state,
            block_entity_modifier: Rc::new(Passthrough),
        }
    }

    /// `new(input, loc, posPredicate, outputState, modifier)`.
    pub fn with_modifier(
        input_predicate: Rc<dyn RuleTest>,
        loc_predicate: Rc<dyn RuleTest>,
        pos_predicate: Rc<dyn PosRuleTest>,
        output_state: BlockState,
        block_entity_modifier: Rc<dyn RuleBlockEntityModifier>,
    ) -> Self {
        Self {
            input_predicate,
            loc_predicate,
            pos_predicate,
            output_state,
            block_entity_modifier,
        }
    }

    /// `test(inputState, existingState, localPos, relativePos,
    /// structurePos, random)`.
    #[allow(clippy::too_many_arguments)]
    pub fn test(
        &self,
        input_state: &BlockState,
        existing_state: &BlockState,
        local_pos: BlockPos,
        relative_pos: BlockPos,
        structure_pos: BlockPos,
        random: &mut dyn RandomSource,
    ) -> bool {
        self.input_predicate.test(input_state, random)
            && self.loc_predicate.test(existing_state, random)
            && self.pos_predicate.test(local_pos, relative_pos, structure_pos, random)
    }

    /// `getOutputState`.
    pub fn get_output_state(&self) -> &BlockState {
        &self.output_state
    }

    /// `getOutputTag(random, tag)`.
    pub fn get_output_tag(
        &self,
        random: &mut dyn RandomSource,
        tag: Option<&CompoundTag>,
    ) -> Option<CompoundTag> {
        self.block_entity_modifier.apply(random, tag)
    }
}
