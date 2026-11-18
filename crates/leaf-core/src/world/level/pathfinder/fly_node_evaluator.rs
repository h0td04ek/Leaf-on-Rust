//! Перенос `net.minecraft.world.level.pathfinder.FlyNodeEvaluator`.

use crate::core::block_pos::BlockPos;
use crate::util::mth;
use crate::world::entity::mob::MobPathing;
use crate::world::level::pathfinder::node::NodeRef;
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::pathfinding_context::PathfindingContext;
use crate::world::level::pathfinder::target::Target;
use crate::world::level::pathfinder::walk_node_evaluator::WalkNodeEvaluator;
use std::collections::{HashMap, HashSet};

/// `FlyNodeEvaluator`.
pub struct FlyNodeEvaluator {
    inner: WalkNodeEvaluator,
    /// `pathTypeByPosCache`.
    path_type_by_pos_cache: HashMap<i64, PathType>,
}

impl FlyNodeEvaluator {
    /// `new`.
    pub fn new() -> Self {
        Self {
            inner: WalkNodeEvaluator::new(),
            path_type_by_pos_cache: HashMap::new(),
        }
    }

    /// `SMALL_MOB_SIZE`.
    const SMALL_MOB_SIZE: f32 = 1.0;
    /// `SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX`.
    const SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX: f32 = 1.1;
    /// `MAX_START_NODE_CANDIDATES`.
    const MAX_START_NODE_CANDIDATES: i32 = 10;
}

impl Default for FlyNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl FlyNodeEvaluator {
    fn can_start_at_fly(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, pos: BlockPos) -> bool {
        let cached = self.get_cached_path_type(ctx, mob, pos.x, pos.y, pos.z);
        mob.get_pathfinding_malus(cached) >= 0.0
    }


    /// `findAcceptedNode(x, y, z)` — fly-вариант.
    fn find_accepted_node(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, x: i32, y: i32, z: i32) -> Option<NodeRef> {
        let cached = self.get_cached_path_type(ctx, mob, x, y, z);
        let malus = mob.get_pathfinding_malus(cached);
        if malus < 0.0 {
            return None;
        }
        let n = self.get_node(x, y, z);
        {
            let mut nb = n.borrow_mut();
            nb.path_type = cached;
            nb.cost_malus = nb.cost_malus.max(malus);
            if cached == PathType::Walkable {
                nb.cost_malus += 1.0;
            }
        }
        Some(n)
    }

    /// `getCachedPathType` — fly-кэш.
    fn get_cached_path_type(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, x: i32, y: i32, z: i32) -> PathType {
        let key = BlockPos::as_long_static(x, y, z);
        if let Some(t) = self.path_type_by_pos_cache.get(&key) {
            return *t;
        }
        let t = self.path_type_of_mob_fly(ctx, x, y, z, mob);
        self.path_type_by_pos_cache.insert(key, t);
        t
    }

    /// `isOpen`/`hasMalus`.
    fn is_open(&self, node: &Option<NodeRef>) -> bool {
        node.as_ref().map(|n| !n.borrow().closed).unwrap_or(false)
    }
    fn has_malus(&self, node: &Option<NodeRef>) -> bool {
        node.as_ref().map(|n| n.borrow().cost_malus >= 0.0).unwrap_or(false)
    }

    /// `getPathType` — fly-вариант (переопределяет Walk).
    fn path_type_fly(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        let mut path_type_from_state = WalkNodeEvaluator::get_path_type_static(ctx, x, y, z);
        if path_type_from_state == PathType::Open && y >= ctx.level.min_build_height() + 1 {
            let block_pos = BlockPos::new(x, y - 1, z);
            let below = WalkNodeEvaluator::get_path_type_static(ctx, block_pos.x, block_pos.y, block_pos.z);
            path_type_from_state = match below {
                PathType::DamageFire | PathType::Lava => PathType::DamageFire,
                PathType::DamageOther => PathType::DamageOther,
                PathType::Cocoa => PathType::Cocoa,
                PathType::Fence => {
                    if block_pos != ctx.mob_position {
                        PathType::Fence
                    } else {
                        path_type_from_state
                    }
                }
                PathType::Walkable | PathType::Open | PathType::Water => PathType::Open,
                _ => PathType::Walkable,
            };
        }
        if path_type_from_state == PathType::Walkable || path_type_from_state == PathType::Open {
            path_type_from_state = WalkNodeEvaluator::check_neighbour_blocks(ctx, x, y, z, path_type_from_state);
        }
        path_type_from_state
    }

    /// `getPathTypeOfMob` — fly-вариант (наследуется от Walk,
    /// но `getPathType` виртуальный → fly-версия выше).
    fn path_type_of_mob_fly(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32, mob: &dyn MobPathing) -> PathType {
        let path_type_within = self.within_mob_bb_fly(ctx, x, y, z);
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
        if self.entity_width() <= 1
            && path_type != PathType::Open
            && mob.get_pathfinding_malus(path_type) == 0.0
            && self.path_type_fly(ctx, x, y, z) == PathType::Open
        {
            PathType::Open
        } else {
            path_type
        }
    }

    /// `getPathTypeWithinMobBB` — fly-вариант (getPathType → fly).
    fn within_mob_bb_fly(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> HashSet<PathType> {
        let mut set = HashSet::new();
        let block_pos = ctx.mob_position;
        for i in 0..self.entity_width() {
            for i1 in 0..self.entity_height() {
                for i2 in 0..self.entity_depth() {
                    let mut path_type = self.path_type_fly(ctx, i + x, i1 + y, i2 + z);
                    let can_pass_doors = self.can_pass_doors();
                    if path_type == PathType::DoorWoodClosed && self.can_open_doors() && can_pass_doors {
                        path_type = PathType::WalkableDoor;
                    }
                    if path_type == PathType::DoorOpen && !can_pass_doors {
                        path_type = PathType::Blocked;
                    }
                    if path_type == PathType::Rail
                        && self.path_type_fly(ctx, block_pos.x, block_pos.y, block_pos.z) != PathType::Rail
                        && self.path_type_fly(ctx, block_pos.x, block_pos.y - 1, block_pos.z) != PathType::Rail
                    {
                        path_type = PathType::UnpassableRail;
                    }
                    set.insert(path_type);
                }
            }
        }
        set
    }

    /// `iteratePathfindingStartNodeCandidatePositions`.
    fn iterate_start_candidates(&self, mob: &dyn MobPathing) -> Vec<BlockPos> {
        let bb = mob.bounding_box();
        if bb.get_size() >= Self::SMALL_MOB_SIZE as f64 {
            vec![
                BlockPos::containing(bb.min_x, mob.get_block_y() as f64, bb.min_z),
                BlockPos::containing(bb.min_x, mob.get_block_y() as f64, bb.max_z),
                BlockPos::containing(bb.max_x, mob.get_block_y() as f64, bb.min_z),
                BlockPos::containing(bb.max_x, mob.get_block_y() as f64, bb.max_z),
            ]
        } else {
            let max = 0.0f64.max(Self::SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX as f64 - bb.get_zsize());
            let max1 = 0.0f64.max(Self::SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX as f64 - bb.get_xsize());
            let max2 = 0.0f64.max(Self::SMALL_MOB_INFLATED_START_NODE_BOUNDING_BOX as f64 - bb.get_ysize());
            let aabb = bb.inflate(max1, max2, max);
            let mut out = Vec::new();
            for _ in 0..Self::MAX_START_NODE_CANDIDATES {
                out.push(BlockPos::new(
                    mob.next_int_between(mth::floor(aabb.min_x), mth::floor(aabb.max_x)),
                    mob.next_int_between(mth::floor(aabb.min_y), mth::floor(aabb.max_y)),
                    mob.next_int_between(mth::floor(aabb.min_z), mth::floor(aabb.max_z)),
                ));
            }
            out
        }
    }
}

impl NodeEvaluator for FlyNodeEvaluator {
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
        self.path_type_by_pos_cache.clear();
    }

    fn done(&mut self, mob: &dyn MobPathing) {
        self.inner.done(mob);
        self.path_type_by_pos_cache.clear();
    }

    fn get_start(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing) -> Option<NodeRef> {
        let mut block_y;
        if self.inner.can_float() && mob.is_in_water() {
            block_y = mob.get_block_y();
            loop {
                let is_water = ctx
                    .get_block_state(BlockPos::containing(mob.get_x(), block_y as f64, mob.get_z()))
                    .map(|s| s.block.id == crate::resources::identifier::Identifier::minecraft("water"))
                    .unwrap_or(false);
                if !is_water {
                    break;
                }
                block_y += 1;
            }
        } else {
            block_y = mth::floor(mob.get_y() + 0.5);
        }
        let block_pos = BlockPos::containing(mob.get_x(), block_y as f64, mob.get_z());
        let mut found = None;
        if self.can_start_at_fly(ctx, mob, block_pos) {
            found = Some(block_pos);
        } else {
            for cand in self.iterate_start_candidates(mob) {
                if self.can_start_at_fly(ctx, mob, cand) {
                    found = Some(cand);
                    break;
                }
            }
        }
        let pos = found.unwrap_or(block_pos);
        Some(self.inner.get_start_node(ctx, mob, pos))
    }

    fn get_target(&mut self, x: f64, y: f64, z: f64) -> Target {
        self.inner.get_target_node_at(x, y, z)
    }

    fn get_neighbors(
        &mut self,
        output: &mut Vec<NodeRef>,
        node: &NodeRef,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
    ) -> usize {
        let start_len = output.len();
        let (nx, ny, nz) = {
            let n = node.borrow();
            (n.x, n.y, n.z)
        };
        macro_rules! emit {
            ($opt:expr) => {
                if self.is_open(&$opt) {
                    if let Some(nn) = $opt {
                        output.push(nn);
                    }
                }
            };
        }
        // 6 осевых
        let node1 = self.find_accepted_node(ctx, mob, nx, ny, nz + 1);
        emit!(node1.clone());
        let node2 = self.find_accepted_node(ctx, mob, nx - 1, ny, nz);
        emit!(node2.clone());
        let node3 = self.find_accepted_node(ctx, mob, nx + 1, ny, nz);
        emit!(node3.clone());
        let node4 = self.find_accepted_node(ctx, mob, nx, ny, nz - 1);
        emit!(node4.clone());
        let node5 = self.find_accepted_node(ctx, mob, nx, ny + 1, nz);
        emit!(node5.clone());
        let node6 = self.find_accepted_node(ctx, mob, nx, ny - 1, nz);
        emit!(node6.clone());

        // Верхние Y-диагонали
        let n7 = self.find_accepted_node(ctx, mob, nx, ny + 1, nz + 1);
        if self.is_open(&n7) && self.has_malus(&node1) && self.has_malus(&node5) { emit!(n7.clone()); }
        let n8 = self.find_accepted_node(ctx, mob, nx - 1, ny + 1, nz);
        if self.is_open(&n8) && self.has_malus(&node2) && self.has_malus(&node5) { emit!(n8.clone()); }
        let n9 = self.find_accepted_node(ctx, mob, nx + 1, ny + 1, nz);
        if self.is_open(&n9) && self.has_malus(&node3) && self.has_malus(&node5) { emit!(n9.clone()); }
        let n10 = self.find_accepted_node(ctx, mob, nx, ny + 1, nz - 1);
        if self.is_open(&n10) && self.has_malus(&node4) && self.has_malus(&node5) { emit!(n10.clone()); }

        // Нижние Y-диагонали
        let n11 = self.find_accepted_node(ctx, mob, nx, ny - 1, nz + 1);
        if self.is_open(&n11) && self.has_malus(&node1) && self.has_malus(&node6) { emit!(n11.clone()); }
        let n12 = self.find_accepted_node(ctx, mob, nx - 1, ny - 1, nz);
        if self.is_open(&n12) && self.has_malus(&node2) && self.has_malus(&node6) { emit!(n12.clone()); }
        let n13 = self.find_accepted_node(ctx, mob, nx + 1, ny - 1, nz);
        if self.is_open(&n13) && self.has_malus(&node3) && self.has_malus(&node6) { emit!(n13.clone()); }
        let n14 = self.find_accepted_node(ctx, mob, nx, ny - 1, nz - 1);
        if self.is_open(&n14) && self.has_malus(&node4) && self.has_malus(&node6) { emit!(n14.clone()); }

        // Горизонтальные диагонали
        let n15 = self.find_accepted_node(ctx, mob, nx + 1, ny, nz - 1);
        if self.is_open(&n15) && self.has_malus(&node4) && self.has_malus(&node3) { emit!(n15.clone()); }
        let n16 = self.find_accepted_node(ctx, mob, nx + 1, ny, nz + 1);
        if self.is_open(&n16) && self.has_malus(&node1) && self.has_malus(&node3) { emit!(n16.clone()); }
        let n17 = self.find_accepted_node(ctx, mob, nx - 1, ny, nz - 1);
        if self.is_open(&n17) && self.has_malus(&node4) && self.has_malus(&node2) { emit!(n17.clone()); }
        let n18 = self.find_accepted_node(ctx, mob, nx - 1, ny, nz + 1);
        if self.is_open(&n18) && self.has_malus(&node1) && self.has_malus(&node2) { emit!(n18.clone()); }

        // 3D-диагонали
        let n19 = self.find_accepted_node(ctx, mob, nx + 1, ny + 1, nz - 1);
        if self.is_open(&n19)
            && self.has_malus(&n15) && self.has_malus(&node4) && self.has_malus(&node3)
            && self.has_malus(&node5) && self.has_malus(&n10) && self.has_malus(&n9)
        { emit!(n19); }
        let n20 = self.find_accepted_node(ctx, mob, nx + 1, ny + 1, nz + 1);
        if self.is_open(&n20)
            && self.has_malus(&n16) && self.has_malus(&node1) && self.has_malus(&node3)
            && self.has_malus(&node5) && self.has_malus(&n7) && self.has_malus(&n9)
        { emit!(n20); }
        let n21 = self.find_accepted_node(ctx, mob, nx - 1, ny + 1, nz - 1);
        if self.is_open(&n21)
            && self.has_malus(&n17) && self.has_malus(&node4) && self.has_malus(&node2)
            && self.has_malus(&node5) && self.has_malus(&n10) && self.has_malus(&n8)
        { emit!(n21); }
        let n22 = self.find_accepted_node(ctx, mob, nx - 1, ny + 1, nz + 1);
        if self.is_open(&n22)
            && self.has_malus(&n18) && self.has_malus(&node1) && self.has_malus(&node2)
            && self.has_malus(&node5) && self.has_malus(&n7) && self.has_malus(&n8)
        { emit!(n22); }
        let n23 = self.find_accepted_node(ctx, mob, nx + 1, ny - 1, nz - 1);
        if self.is_open(&n23)
            && self.has_malus(&n15) && self.has_malus(&node4) && self.has_malus(&node3)
            && self.has_malus(&node6) && self.has_malus(&n14) && self.has_malus(&n13)
        { emit!(n23); }
        let n24 = self.find_accepted_node(ctx, mob, nx + 1, ny - 1, nz + 1);
        if self.is_open(&n24)
            && self.has_malus(&n16) && self.has_malus(&node1) && self.has_malus(&node3)
            && self.has_malus(&node6) && self.has_malus(&n11) && self.has_malus(&n13)
        { emit!(n24); }
        let n25 = self.find_accepted_node(ctx, mob, nx - 1, ny - 1, nz - 1);
        if self.is_open(&n25)
            && self.has_malus(&n17) && self.has_malus(&node4) && self.has_malus(&node2)
            && self.has_malus(&node6) && self.has_malus(&n14) && self.has_malus(&n12)
        { emit!(n25); }
        let n26 = self.find_accepted_node(ctx, mob, nx - 1, ny - 1, nz + 1);
        if self.is_open(&n26)
            && self.has_malus(&n18) && self.has_malus(&node1) && self.has_malus(&node2)
            && self.has_malus(&node6) && self.has_malus(&n11) && self.has_malus(&n12)
        { emit!(n26); }

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
        self.path_type_of_mob_fly(ctx, x, y, z, mob)
    }

    fn get_path_type(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        self.path_type_fly(ctx, x, y, z)
    }
}
