//! Перенос `net.minecraft.world.level.pathfinder.AmphibiousNodeEvaluator`.

use crate::core::block_pos::BlockPos;
use crate::core::direction::Direction;
use crate::util::mth;
use crate::world::entity::mob::MobPathing;
use crate::world::level::pathfinder::node::{NodeRef};
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::pathfinding_context::PathfindingContext;
use crate::world::level::pathfinder::target::Target;
use crate::world::level::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use std::collections::HashMap;

/// `AmphibiousNodeEvaluator` — WalkNodeEvaluator с amphibious-режимом
/// и переназначенными malus (WATER=0, WALKABLE=6, WATER_BORDER=4).
pub struct AmphibiousNodeEvaluator {
    inner: WalkNodeEvaluator,
    /// `prefersShallowSwimming`.
    prefers_shallow_swimming: bool,
    /// `oldWalkableCost`/`oldWaterBorderCost`.
    old_walkable_cost: f32,
    old_water_border_cost: f32,
}

impl AmphibiousNodeEvaluator {
    /// `new AmphibiousNodeEvaluator(prefersShallowSwimming)`.
    pub fn new(prefers_shallow_swimming: bool) -> Self {
        let mut inner = WalkNodeEvaluator::new();
        inner.amphibious = true;
        Self {
            inner,
            prefers_shallow_swimming,
            old_walkable_cost: 0.0,
            old_water_border_cost: 0.0,
        }
    }

    /// `getPathTypeWithinMobBB` — amphibious-вариант (getPathType
    /// виртуальный → water-border проверка).
    fn within_mob_bb_amphibious(
        &mut self,
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
    ) -> std::collections::HashSet<PathType> {
        let mut set = std::collections::HashSet::new();
        let block_pos = ctx.mob_position;
        for i in 0..self.inner.entity_width() {
            for i1 in 0..self.inner.entity_height() {
                for i2 in 0..self.inner.entity_depth() {
                    let i3 = i + x;
                    let i4 = i1 + y;
                    let i5 = i2 + z;
                    let mut path_type = self.get_path_type(ctx, i3, i4, i5);
                    let can_pass_doors = self.inner.can_pass_doors();
                    if path_type == PathType::DoorWoodClosed && self.inner.can_open_doors() && can_pass_doors {
                        path_type = PathType::WalkableDoor;
                    }
                    if path_type == PathType::DoorOpen && !can_pass_doors {
                        path_type = PathType::Blocked;
                    }
                    if path_type == PathType::Rail
                        && self.get_path_type(ctx, block_pos.x, block_pos.y, block_pos.z) != PathType::Rail
                        && self.get_path_type(ctx, block_pos.x, block_pos.y - 1, block_pos.z) != PathType::Rail
                    {
                        path_type = PathType::UnpassableRail;
                    }
                    set.insert(path_type);
                }
            }
        }
        set
    }

    /// `getPathTypeOfMob` — amphibious-вариант.
    fn path_type_of_mob_amphibious(
        &mut self,
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
        mob: &dyn MobPathing,
    ) -> PathType {
        let path_type_within = self.within_mob_bb_amphibious(ctx, x, y, z);
        if path_type_within.contains(&PathType::Fence) {
            return PathType::Fence;
        }
        if path_type_within.contains(&PathType::UnpassableRail) {
            return PathType::UnpassableRail;
        }
        let mut path_type = PathType::Blocked;
        for &pt in &path_type_within {
            if mob.get_pathfinding_malus(pt) < 0.0 {
                return pt;
            }
            if mob.get_pathfinding_malus(pt) >= mob.get_pathfinding_malus(path_type) {
                path_type = pt;
            }
        }
        if self.inner.entity_width() <= 1
            && path_type != PathType::Open
            && mob.get_pathfinding_malus(path_type) == 0.0
            && self.get_path_type(ctx, x, y, z) == PathType::Open
        {
            PathType::Open
        } else {
            path_type
        }
    }

    /// `isVerticalNeighborValid`.
    fn is_vertical_neighbor_valid(&self, neighbor: Option<&NodeRef>, node: &NodeRef) -> bool {
        self.inner.is_neighbor_valid(neighbor, node)
            && neighbor
                .map(|n| n.borrow().path_type == PathType::Water)
                .unwrap_or(false)
    }
}

impl Default for AmphibiousNodeEvaluator {
    fn default() -> Self {
        Self::new(false)
    }
}

impl NodeEvaluator for AmphibiousNodeEvaluator {
    fn nodes(&mut self) -> &mut HashMap<i32, NodeRef> {
        self.inner.nodes()
    }
    fn entity_width(&self) -> i32 { self.inner.entity_width() }
    fn entity_height(&self) -> i32 { self.inner.entity_height() }
    fn entity_depth(&self) -> i32 { self.inner.entity_depth() }
    fn can_pass_doors(&self) -> bool { self.inner.can_pass_doors() }
    fn can_open_doors(&self) -> bool { self.inner.can_open_doors() }
    fn can_float(&self) -> bool { self.inner.can_float() }
    fn can_walk_over_fences(&self) -> bool { self.inner.can_walk_over_fences() }
    fn set_can_pass_doors(&mut self, v: bool) { self.inner.set_can_pass_doors(v); }
    fn set_can_open_doors(&mut self, v: bool) { self.inner.set_can_open_doors(v); }
    fn set_can_float(&mut self, v: bool) { self.inner.set_can_float(v); }
    fn set_can_walk_over_fences(&mut self, v: bool) { self.inner.set_can_walk_over_fences(v); }

    fn prepare(&mut self, mob: &dyn MobPathing) {
        self.inner.prepare(mob);
        mob.set_pathfinding_malus(PathType::Water, 0.0);
        self.old_walkable_cost = mob.get_pathfinding_malus(PathType::Walkable);
        mob.set_pathfinding_malus(PathType::Walkable, 6.0);
        self.old_water_border_cost = mob.get_pathfinding_malus(PathType::WaterBorder);
        mob.set_pathfinding_malus(PathType::WaterBorder, 4.0);
    }

    fn done(&mut self, mob: &dyn MobPathing) {
        mob.set_pathfinding_malus(PathType::Walkable, self.old_walkable_cost);
        mob.set_pathfinding_malus(PathType::WaterBorder, self.old_water_border_cost);
        self.inner.done(mob);
    }

    fn get_start(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing) -> Option<NodeRef> {
        if !mob.is_in_water() {
            return self.inner.get_start(ctx, mob);
        }
        let bb = mob.bounding_box();
        let pos = BlockPos::new(
            mth::floor(bb.min_x),
            mth::floor(bb.min_y + 0.5),
            mth::floor(bb.min_z),
        );
        Some(self.inner.get_start_node(ctx, mob, pos))
    }

    fn get_target(&mut self, x: f64, y: f64, z: f64) -> Target {
        self.inner.get_target_node_at(x, y + 0.5, z)
    }

    fn get_neighbors(
        &mut self,
        output: &mut Vec<NodeRef>,
        node: &NodeRef,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
    ) -> usize {
        let start_len = output.len();
        let i = <WalkNodeEvaluator as NodeEvaluator>::get_neighbors(&mut self.inner, output, node, ctx, mob);
        let (nx, ny, nz) = {
            let n = node.borrow();
            (n.x, n.y, n.z)
        };
        let cached_path_type = self.inner.get_cached_path_type_mob(ctx, mob, nx, ny + 1, nz);
        let cached_path_type1 = self.inner.get_cached_path_type_mob(ctx, mob, nx, ny, nz);
        let floor = if mob.get_pathfinding_malus(cached_path_type) >= 0.0
            && cached_path_type1 != PathType::StickyHoney
        {
            mth::floor((1.0f32).max(mob.max_up_step()) as f64) as i32
        } else {
            0
        };
        let floor_level = self.inner.get_floor_level(ctx, BlockPos::new(nx, ny, nz));
        let node1 = self.inner.find_accepted_node(
            ctx, mob, nx, ny + 1, nz,
            (0).max(floor - 1),
            floor_level,
            Direction::UP,
            cached_path_type1,
        );
        let node2 = self.inner.find_accepted_node(
            ctx, mob, nx, ny - 1, nz,
            floor,
            floor_level,
            Direction::DOWN,
            cached_path_type1,
        );
        let mut i = i;
        if self.is_vertical_neighbor_valid(node1.as_ref(), node) {
            if let Some(n) = node1 {
                output.push(n);
                i += 1;
            }
        }
        if self.is_vertical_neighbor_valid(node2.as_ref(), node) && cached_path_type1 != PathType::Trapdoor {
            if let Some(n) = node2 {
                output.push(n);
                i += 1;
            }
        }
        // prefersShallowSwimming: глубокая вода дороже.
        if self.prefers_shallow_swimming {
            if let Some(level) = mob.level() {
                let sea_level = level.sea_level();
                for n in output.iter_mut().skip(start_len) {
                    let mut nb = n.borrow_mut();
                    if nb.path_type == PathType::Water && nb.y < sea_level - 10 {
                        nb.cost_malus += 1.0;
                    }
                }
            }
        }
        let _ = i;
        output.len() - start_len
    }

    fn get_path_type_of_mob(
        &mut self,
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
        mob: &dyn MobPathing,
    ) -> PathType {
        self.path_type_of_mob_amphibious(ctx, x, y, z, mob)
    }

    fn get_path_type(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        let path_type_from_state = ctx.get_path_type_from_state(x, y, z);
        if path_type_from_state == PathType::Water {
            // Проверка 6 соседей: если рядом BLOCKED → WATER_BORDER.
            for direction in Direction::VALUES {
                let pos = BlockPos::new(
                    x + direction.get_step_x(),
                    y + direction.get_step_y(),
                    z + direction.get_step_z(),
                );
                if ctx.get_path_type_from_state(pos.x, pos.y, pos.z) == PathType::Blocked {
                    return PathType::WaterBorder;
                }
            }
            PathType::Water
        } else {
            WalkNodeEvaluator::get_path_type_static(ctx, x, y, z)
        }
    }
}
