//! Перенос `net.minecraft.world.level.pathfinder.NodeEvaluator`.
//!
//! Абстрактный оценщик узлов. `Mob` передаётся через трейт
//! `MobPathing` (чтобы не было циклической зависимости на
//! конкретную структуру `Mob`).

use crate::core::block_pos::BlockPos;
use crate::world::entity::mob::MobPathing;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::pathfinder::node::{Node, NodeRef};
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::pathfinding_context::PathfindingContext;
use crate::world::level::pathfinder::target::Target;
use std::collections::HashMap;

/// `NodeEvaluator` — базовый оценщик (поля `nodes`,
/// `entityWidth/Height/Depth`, флаги canPassDoors и т.д.).
pub trait NodeEvaluator {
    /// `nodes` — карта hash → Node.
    fn nodes(&mut self) -> &mut HashMap<i32, NodeRef>;

    /// `entityWidth`, `entityHeight`, `entityDepth`.
    fn entity_width(&self) -> i32;
    fn entity_height(&self) -> i32;
    fn entity_depth(&self) -> i32;

    /// `canPassDoors`, `canOpenDoors`, `canFloat`, `canWalkOverFences`.
    fn can_pass_doors(&self) -> bool;
    fn can_open_doors(&self) -> bool;
    fn can_float(&self) -> bool;
    fn can_walk_over_fences(&self) -> bool;

    /// `setCanPassDoors` и др.
    fn set_can_pass_doors(&mut self, _v: bool) {}
    fn set_can_open_doors(&mut self, _v: bool) {}
    fn set_can_float(&mut self, _v: bool) {}
    fn set_can_walk_over_fences(&mut self, _v: bool) {}

    /// `prepare` — инициализация размеров сущности.
    fn prepare(&mut self, mob: &dyn MobPathing) {
        self.nodes().clear();
    }

    /// `done` — mob передаётся для восстановления malus
    /// (AmphibiousNodeEvaluator.done восстанавливает WALKABLE/
    /// WATER_BORDER через mob.setPathfindingMalus).
    fn done(&mut self, _mob: &dyn MobPathing) {}

    /// `getNode(x, y, z)` — узел по координатам (создаёт при
    /// отсутствии).
    fn get_node(&mut self, x: i32, y: i32, z: i32) -> NodeRef {
        let hash = Node::create_hash(x, y, z);
        let nodes = self.nodes();
        if let Some(n) = nodes.get(&hash) {
            return n.clone();
        }
        let node = Node::node_ref(x, y, z);
        nodes.insert(hash, node.clone());
        node
    }

    /// `getStart`.
    fn get_start(
        &mut self,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
    ) -> Option<NodeRef>;

    /// `getTarget(x, y, z)`.
    fn get_target(&mut self, x: f64, y: f64, z: f64) -> Target;

    /// `getTargetNodeAt`.
    fn get_target_node_at(&mut self, x: f64, y: f64, z: f64) -> Target {
        let node = self.get_node(crate::util::mth::floor(x) as i32, crate::util::mth::floor(y) as i32, crate::util::mth::floor(z) as i32);
        let n = node.borrow();
        Target::new(n.x, n.y, n.z)
    }

    /// `getNeighbors(output, node)` — записывает соседей в
    /// `output` (очищается вызывающим).
    fn get_neighbors(
        &mut self,
        output: &mut Vec<NodeRef>,
        node: &NodeRef,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
    ) -> usize;

    /// `getPathTypeOfMob`.
    fn get_path_type_of_mob(
        &mut self,
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
        mob: &dyn MobPathing,
    ) -> PathType;

    /// `getPathType`.
    fn get_path_type(
        &mut self,
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
    ) -> PathType;

    /// `getPathType(Mob, BlockPos)` — контекст создаётся на
    /// лету (как в Java).
    fn get_path_type_at(
        &mut self,
        mob: &dyn MobPathing,
        pos: BlockPos,
    ) -> PathType {
        let level = mob.level();
        let level_ref: &dyn crate::world::level::level_reader::LevelReader =
            level.as_ref().map(|l| l.as_ref()).unwrap_or_else(|| {
                // У моба всегда есть уровень — Java `mob.level()`
                // не бывает null.
                panic!("MobPathing::level() returned None")
            });
        let cache = level_ref.path_type_cache();
        let mut ctx =
            PathfindingContext::new(level_ref, cache, mob.block_position());
        self.get_path_type(&mut ctx, pos.x, pos.y, pos.z)
    }
}

/// `NodeEvaluator.isBurningBlock`.
pub fn is_burning_block(state: &BlockState) -> bool {
    // `state.is(BlockTags.FIRE) || state.is(Blocks.LAVA) ||
    // state.is(Blocks.MAGMA_BLOCK) || CampfireBlock.isLitCampfire
    // || state.is(Blocks.LAVA_CAULDRON)`.
    let id = &state.block.id;
    *id == crate::resources::identifier::Identifier::minecraft("lava")
        || *id == crate::resources::identifier::Identifier::minecraft("magma_block")
        || *id == crate::resources::identifier::Identifier::minecraft("lava_cauldron")
        || state.is_named_tag(&crate::tags::block_tags::BlockTags::fire())
        || (id == &crate::resources::identifier::Identifier::minecraft("campfire")
            || id == &crate::resources::identifier::Identifier::minecraft("soul_campfire"))
            && state.is_lit_campfire()
}
