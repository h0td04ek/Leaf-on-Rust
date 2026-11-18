//! Перенос `net.minecraft.world.level.pathfinder.WalkNodeEvaluator`.

use crate::core::block_pos::BlockPos;
use crate::core::direction::{Direction, Plane};
use crate::tags::block_tags::BlockTags;
use crate::tags::fluid_tags::FluidTags;
use crate::util::mth;
use crate::world::entity::mob::MobPathing;
use crate::world::level::block::state::block_state::BlockState;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::pathfinder::node::NodeRef;
use crate::world::level::pathfinder::node_evaluator::{is_burning_block, NodeEvaluator};
use crate::world::level::pathfinder::path_computation_type::PathComputationType;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::pathfinding_context::PathfindingContext;
use crate::world::level::pathfinder::target::Target;
use crate::world::phys::aabb::Aabb;
use crate::world::phys::vec3::Vec3;
use std::collections::{HashMap, HashSet};

/// `WalkNodeEvaluator`.
pub struct WalkNodeEvaluator {
    /// `nodes`.
    nodes: HashMap<i32, NodeRef>,
    /// `entityWidth`, `entityHeight`, `entityDepth`.
    entity_width: i32,
    entity_height: i32,
    entity_depth: i32,
    /// `canPassDoors`, `canOpenDoors`, `canFloat`, `canWalkOverFences`.
    can_pass_doors: bool,
    can_open_doors: bool,
    can_float: bool,
    can_walk_over_fences: bool,
    /// `pathTypesByPosCacheByMob`.
    path_types_by_pos_cache: HashMap<i64, PathType>,
    /// `collisionCache` — AABB не Hash, храним список.
    collision_cache: Vec<(Aabb, bool)>,
    /// `amphibious` — режим AmphibiousNodeEvaluator (isAmphibious).
    pub(crate) amphibious: bool,
}

impl WalkNodeEvaluator {
    /// `new`.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            entity_width: 0,
            entity_height: 0,
            entity_depth: 0,
            can_pass_doors: true,
            can_open_doors: false,
            can_float: false,
            can_walk_over_fences: false,
            path_types_by_pos_cache: HashMap::new(),
            collision_cache: Vec::new(),
            amphibious: false,
        }
    }

    /// `isAmphibious` — переопределяется AmphibiousNodeEvaluator.
    pub(crate) fn is_amphibious(&self) -> bool {
        self.amphibious
    }

    /// `SPACE_BETWEEN_WALL_POSTS`.
    pub const SPACE_BETWEEN_WALL_POSTS: f64 = 0.5;
    /// `DEFAULT_MOB_JUMP_HEIGHT`.
    const DEFAULT_MOB_JUMP_HEIGHT: f64 = 1.125;

    /// `getStartNode`.
    pub(crate) fn get_start_node(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, pos: BlockPos) -> NodeRef {
        let node = self.get_node(pos.x, pos.y, pos.z);
        let cached = self.get_cached_path_type_mob(ctx, mob, node.borrow().x, node.borrow().y, node.borrow().z);
        node.borrow_mut().path_type = cached;
        let malus = mob.get_pathfinding_malus(cached);
        node.borrow_mut().cost_malus = malus;
        node
    }

    /// `canStartAt`.
    pub(crate) fn can_start_at(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, pos: BlockPos) -> bool {
        let cached = self.get_cached_path_type_mob(ctx, mob, pos.x, pos.y, pos.z);
        cached != PathType::Open && mob.get_pathfinding_malus(cached) >= 0.0
    }

    /// `findAcceptedNode`.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn find_accepted_node(
        &mut self,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
        x: i32,
        y: i32,
        z: i32,
        vertical_delta_limit: i32,
        node_floor_level: f64,
        direction: Direction,
        path_type: PathType,
    ) -> Option<NodeRef> {
        let mut node: Option<NodeRef> = None;
        let floor_level = self.get_floor_level(ctx, BlockPos::new(x, y, z));
        if floor_level - node_floor_level > self.get_mob_jump_height(mob) {
            return None;
        }
        let cached_path_type = self.get_cached_path_type_mob(ctx, mob, x, y, z);
        let pathfinding_malus = mob.get_pathfinding_malus(cached_path_type);
        if pathfinding_malus >= 0.0 {
            let n = self.get_node_and_update_cost_to_max(x, y, z, cached_path_type, pathfinding_malus);
            node = Some(n);
        }
                if Self::does_block_have_partial_collision(path_type) {
            if let Some(n) = &node {
                let (cost, nx, ny, nz) = {
                    let nb = n.borrow();
                    (nb.cost_malus, nb.x, nb.y, nb.z)
                };
                if cost >= 0.0 && !self.can_reach_without_collision(ctx, mob, nx, ny, nz) {
                    node = None;
                }
            }
        }
        if cached_path_type != PathType::Walkable
            && (!self.is_amphibious() || cached_path_type != PathType::Water)
        {
            let node_cost_neg = match &node {
                Some(n) => n.borrow().cost_malus < 0.0,
                None => true,
            };
            if vertical_delta_limit > 0
                && node_cost_neg
                && (cached_path_type != PathType::Fence || self.can_walk_over_fences())
                && cached_path_type != PathType::UnpassableRail
                && cached_path_type != PathType::Trapdoor
                && cached_path_type != PathType::PowderSnow
            {
                node = self.try_jump_on(ctx, mob, x, y, z, vertical_delta_limit, node_floor_level, direction, path_type);
            } else if !self.is_amphibious() && cached_path_type == PathType::Water && !self.can_float() {
                node = self.try_find_first_non_water_below(ctx, mob, x, y, z, node);
            } else if cached_path_type == PathType::Open {
                node = Some(self.try_find_first_ground_node_below(ctx, mob, x, y, z));
            } else if Self::does_block_have_partial_collision(cached_path_type) && node.is_none() {
                node = Some(self.get_closed_node(x, y, z, cached_path_type));
            }
            return node;
        }
        node
    }

    /// `getMobJumpHeight`.
    fn get_mob_jump_height(&self, mob: &dyn MobPathing) -> f64 {
        (1.125f64).max(mob.max_up_step() as f64)
    }

    /// `getNodeAndUpdateCostToMax`.
    fn get_node_and_update_cost_to_max(&mut self, x: i32, y: i32, z: i32, path_type: PathType, malus: f32) -> NodeRef {
        let node = self.get_node(x, y, z);
        {
            let mut n = node.borrow_mut();
            n.path_type = path_type;
            n.cost_malus = n.cost_malus.max(malus);
        }
        node
    }

    /// `getBlockedNode`.
    fn get_blocked_node(&mut self, x: i32, y: i32, z: i32) -> NodeRef {
        let node = self.get_node(x, y, z);
        {
            let mut n = node.borrow_mut();
            n.path_type = PathType::Blocked;
            n.cost_malus = -1.0;
        }
        node
    }

    /// `getClosedNode`.
    fn get_closed_node(&mut self, x: i32, y: i32, z: i32, path_type: PathType) -> NodeRef {
        let node = self.get_node(x, y, z);
        {
            let mut n = node.borrow_mut();
            n.closed = true;
            n.path_type = path_type;
            n.cost_malus = path_type.get_malus();
        }
        node
    }

    /// `tryJumpOn`.
    fn try_jump_on(
        &mut self,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
        x: i32,
        y: i32,
        z: i32,
        vertical_delta_limit: i32,
        node_floor_level: f64,
        direction: Direction,
        path_type: PathType,
    ) -> Option<NodeRef> {
        let node = self.find_accepted_node(ctx, mob, x, y + 1, z, vertical_delta_limit - 1, node_floor_level, direction, path_type);
        let node = node?;
        if mob.bb_width() >= 1.0 {
            return Some(node);
        }
        {
            let nb = node.borrow();
            if nb.path_type != PathType::Open && nb.path_type != PathType::Walkable {
                return Some(node.clone());
            }
        }
        let d = x as f64 - direction.get_step_x() as f64 + 0.5;
        let d1 = z as f64 - direction.get_step_z() as f64 + 0.5;
        let d2 = mob.bb_width() as f64 / 2.0;
        let pos = BlockPos::new(x, y + 1, z);
        let floor1 = self.get_floor_level(ctx, pos);
        let nb = node.borrow();
        let floor2 = self.get_floor_level(ctx, BlockPos::new(nb.x, nb.y, nb.z));
        drop(nb);
        let aabb = Aabb::new(
            d - d2,
            floor1 + 0.001,
            d1 - d2,
            d + d2,
            mob.bb_height() as f64 + floor2 - 0.002,
            d1 + d2,
        );
        if self.has_collisions(ctx, mob, aabb) {
            None
        } else {
            Some(node)
        }
    }

    /// `tryFindFirstNonWaterBelow`.
    fn try_find_first_non_water_below(
        &mut self,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
        x: i32,
        y: i32,
        z: i32,
        node: Option<NodeRef>,
    ) -> Option<NodeRef> {
        let mut node = node;
        let mut y = y - 1;
        let min_y = ctx.level.min_build_height();
        while y > min_y {
            let cached = self.get_cached_path_type_mob(ctx, mob, x, y, z);
            if cached != PathType::Water {
                return node;
            }
            let malus = mob.get_pathfinding_malus(cached);
            node = Some(self.get_node_and_update_cost_to_max(x, y, z, cached, malus));
            y -= 1;
        }
        node
    }

    /// `tryFindFirstGroundNodeBelow`.
    fn try_find_first_ground_node_below(
        &mut self,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
        x: i32,
        y: i32,
        z: i32,
    ) -> NodeRef {
        let mut i = y - 1;
        let min_y = ctx.level.min_build_height();
        while i >= min_y {
            if (y - i) as f32 > mob.get_max_fall_distance() {
                return self.get_blocked_node(x, i, z);
            }
            let cached = self.get_cached_path_type_mob(ctx, mob, x, i, z);
            let malus = mob.get_pathfinding_malus(cached);
            if cached != PathType::Open {
                if malus >= 0.0 {
                    return self.get_node_and_update_cost_to_max(x, i, z, cached, malus);
                }
                return self.get_blocked_node(x, i, z);
            }
            i -= 1;
        }
        self.get_blocked_node(x, y, z)
    }

    /// `hasCollisions`.
    fn has_collisions(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, bounding_box: Aabb) -> bool {
        if let Some((_, v)) = self.collision_cache.iter().find(|(b, _)| *b == bounding_box) {
            return *v;
        }
        // `!level.noCollision(mob, bb)` — здесь bb передаём целиком
        // (Mob-вариант фильтрации — у Level).
        let v = !ctx.level.no_collision(&bounding_box);
        let _ = mob;
        self.collision_cache.push((bounding_box, v));
        v
    }

    /// `canReachWithoutCollision`.
    fn can_reach_without_collision(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, x: i32, y: i32, z: i32) -> bool {
        let mut bounding_box = mob.bounding_box();
        let mut vec3 = Vec3::new(
            x as f64 - mob.get_x() + bounding_box.get_xsize() / 2.0,
            y as f64 - mob.get_y() + bounding_box.get_ysize() / 2.0,
            z as f64 - mob.get_z() + bounding_box.get_zsize() / 2.0,
        );
        let ceil = mth::ceil(vec3.length() / bounding_box.get_size());
        vec3 = vec3.scale(1.0 / ceil as f64);
        for i in 1..=ceil {
            bounding_box = bounding_box.move_vec(&vec3);
            if self.has_collisions(ctx, mob, bounding_box) {
                return false;
            }
        }
        true
    }

    /// `getFloorLevel(BlockPos)` — уровень половины блока под pos.
    pub fn get_floor_level(&self, ctx: &mut PathfindingContext, pos: BlockPos) -> f64 {
        if (self.can_float() || self.is_amphibious())
            && ctx.level.get_fluid_state(pos).map(|f| f.is_in_named_tag(&FluidTags::water())).unwrap_or(false)
        {
            return pos.y as f64 + 0.5;
        }
        Self::get_floor_level_static(ctx.level, pos)
    }

    /// `getFloorLevel(BlockGetter, BlockPos)` static.
    pub fn get_floor_level_static(level: &dyn LevelReader, pos: BlockPos) -> f64 {
        let block_pos = pos.below();
        let shape = level
            .get_block_state(block_pos)
            .map(|s| s.get_collision_shape(level, block_pos))
            .unwrap_or_else(crate::world::phys::shapes::VoxelShape::empty);
        block_pos.y as f64 + if shape.is_empty() { 0.0 } else { shape.max(crate::core::direction::Axis::Y) }
    }

    /// `doesBlockHavePartialCollision`.
    fn does_block_have_partial_collision(path_type: PathType) -> bool {
        matches!(path_type, PathType::Fence | PathType::DoorWoodClosed | PathType::DoorIronClosed)
    }

    /// `getPathTypeWithinMobBB`.
    pub fn get_path_type_within_mob_bb(
        &mut self,
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
    ) -> HashSet<PathType> {
        let mut set = HashSet::new();
        let block_pos = ctx.mob_position;
        for i in 0..self.entity_width {
            for i1 in 0..self.entity_height {
                for i2 in 0..self.entity_depth {
                    let i3 = i + x;
                    let i4 = i1 + y;
                    let i5 = i2 + z;
                    let mut path_type = self.get_path_type(ctx, i3, i4, i5);
                    let can_pass_doors = self.can_pass_doors();
                    if path_type == PathType::DoorWoodClosed && self.can_open_doors() && can_pass_doors {
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

    /// `getPathTypeStatic`.
    pub fn get_path_type_static(ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        let path_type_from_state = ctx.get_path_type_from_state(x, y, z);
        if path_type_from_state == PathType::Open && y >= ctx.level.min_build_height() + 1 {
            match ctx.get_path_type_from_state(x, y - 1, z) {
                PathType::Open | PathType::Water | PathType::Lava | PathType::Walkable => PathType::Open,
                PathType::DamageFire => PathType::DamageFire,
                PathType::DamageOther => PathType::DamageOther,
                PathType::StickyHoney => PathType::StickyHoney,
                PathType::PowderSnow => PathType::DangerPowderSnow,
                PathType::DamageCautious => PathType::DamageCautious,
                PathType::Trapdoor => PathType::DangerTrapdoor,
                other => Self::check_neighbour_blocks(ctx, x, y, z, other),
            }
        } else {
            path_type_from_state
        }
    }

    /// `checkNeighbourBlocks`.
    pub fn check_neighbour_blocks(
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
        path_type: PathType,
    ) -> PathType {
        for i in -1..=1 {
            for i1 in -1..=1 {
                for i2 in -1..=1 {
                    if i != 0 || i2 != 0 {
                        let t = ctx.get_path_type_from_state(x + i, y + i1, z + i2);
                        if t == PathType::DamageOther {
                            return PathType::DangerOther;
                        }
                        if t == PathType::DamageFire || t == PathType::Lava {
                            return PathType::DangerFire;
                        }
                        if t == PathType::Water {
                            return PathType::WaterBorder;
                        }
                        if t == PathType::DamageCautious {
                            return PathType::DamageCautious;
                        }
                    }
                }
            }
        }
        path_type
    }

    /// `getPathTypeFromState` — классификация блока.
    pub fn get_path_type_from_state(level: &dyn LevelReader, pos: &BlockPos) -> PathType {
        let Some(state) = level.get_block_state(*pos) else {
            return PathType::Blocked;
        };
        let id = &state.block.id;
        if state.is_air() {
            return PathType::Open;
        }
        if state.is_named_tag(&BlockTags::trapdoors())
            || *id == crate::resources::identifier::Identifier::minecraft("lily_pad")
            || *id == crate::resources::identifier::Identifier::minecraft("big_dripleaf")
        {
            return PathType::Trapdoor;
        }
        if *id == crate::resources::identifier::Identifier::minecraft("powder_snow") {
            return PathType::PowderSnow;
        }
        if *id == crate::resources::identifier::Identifier::minecraft("cactus")
            || *id == crate::resources::identifier::Identifier::minecraft("sweet_berry_bush")
        {
            return PathType::DamageOther;
        }
        if *id == crate::resources::identifier::Identifier::minecraft("honey_block") {
            return PathType::StickyHoney;
        }
        if *id == crate::resources::identifier::Identifier::minecraft("cocoa") {
            return PathType::Cocoa;
        }
        if *id == crate::resources::identifier::Identifier::minecraft("wither_rose")
            || *id == crate::resources::identifier::Identifier::minecraft("pointed_dripstone")
        {
            return PathType::DamageCautious;
        }
        let fluid_state = state.get_fluid_state();
        if fluid_state.is_in_named_tag(&FluidTags::lava()) || fluid_state.is_lava() {
            return PathType::Lava;
        }
        if is_burning_block(&state) {
            return PathType::DamageFire;
        }
        use crate::world::level::block::BlockKind;
        match state.block.kind {
            BlockKind::Door { wooden } => {
                if state.is_open_property() {
                    PathType::DoorOpen
                } else if wooden {
                    PathType::DoorWoodClosed
                } else {
                    PathType::DoorIronClosed
                }
            }
            BlockKind::BaseRail => PathType::Rail,
            BlockKind::Leaves => PathType::Leaves,
            _ => {
                if state.is_named_tag(&BlockTags::fences())
                    || state.is_named_tag(&BlockTags::walls())
                    || (state.block.kind == BlockKind::FenceGate && !state.is_open_property())
                {
                    return PathType::Fence;
                }
                if !state.is_pathfindable(level, *pos, PathComputationType::Land) {
                    return PathType::Blocked;
                }
                if fluid_state.is_in_named_tag(&FluidTags::water()) || fluid_state.is_water() {
                    PathType::Water
                } else {
                    PathType::Open
                }
            }
        }
    }
}

impl WalkNodeEvaluator {
    /// `isNeighborValid`.
    pub(crate) fn is_neighbor_valid(&self, neighbor: Option<&NodeRef>, node: &NodeRef) -> bool {
        match neighbor {
            Some(n) => {
                let nb = n.borrow();
                let cb = node.borrow();
                !nb.closed && (nb.cost_malus >= 0.0 || cb.cost_malus < 0.0)
            }
            None => false,
        }
    }

    /// `isDiagonalValid(root, xNode, zNode)`.
    fn is_diagonal_valid(&self, mob: &dyn MobPathing, root: &NodeRef, x_node: Option<&NodeRef>, z_node: Option<&NodeRef>) -> bool {
        let (Some(xn), Some(zn)) = (x_node, z_node) else {
            return false;
        };
        let r = root.borrow();
        let xn_b = xn.borrow();
        let zn_b = zn.borrow();
        if zn_b.y > r.y || xn_b.y > r.y {
            return false;
        }
        if xn_b.path_type != PathType::WalkableDoor && zn_b.path_type != PathType::WalkableDoor {
            let flag = zn_b.path_type == PathType::Fence
                && xn_b.path_type == PathType::Fence
                && mob.bb_width() < 0.5;
            (zn_b.y < r.y || zn_b.cost_malus >= 0.0 || flag)
                && (xn_b.y < r.y || xn_b.cost_malus >= 0.0 || flag)
        } else {
            false
        }
    }

    /// `isDiagonalValid(node)`.
    fn is_diagonal_valid_node(&self, node: Option<&NodeRef>) -> bool {
        match node {
            Some(n) => {
                let nb = n.borrow();
                !nb.closed && nb.path_type != PathType::WalkableDoor && nb.cost_malus >= 0.0
            }
            None => false,
        }
    }

    /// `getPathTypeOfMob` — через mob.
    pub(crate) fn get_path_type_of_mob_with(
        &mut self,
        ctx: &mut PathfindingContext,
        x: i32,
        y: i32,
        z: i32,
        mob: &dyn MobPathing,
    ) -> PathType {
        let path_type_within = self.get_path_type_within_mob_bb(ctx, x, y, z);
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
        if self.entity_width <= 1
            && path_type != PathType::Open
            && mob.get_pathfinding_malus(path_type) == 0.0
            && self.get_path_type(ctx, x, y, z) == PathType::Open
        {
            PathType::Open
        } else {
            path_type
        }
    }

    /// `getCachedPathType` с mob (для getPathTypeOfMob).
    pub(crate) fn get_cached_path_type_mob(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, x: i32, y: i32, z: i32) -> PathType {
        let key = BlockPos::as_long_static(x, y, z);
        if let Some(t) = self.path_types_by_pos_cache.get(&key) {
            return *t;
        }
        let t = self.get_path_type_of_mob_with(ctx, x, y, z, mob);
        self.path_types_by_pos_cache.insert(key, t);
        t
    }
}

impl Default for WalkNodeEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeEvaluator for WalkNodeEvaluator {
    fn nodes(&mut self) -> &mut HashMap<i32, NodeRef> {
        &mut self.nodes
    }

    fn entity_width(&self) -> i32 { self.entity_width }
    fn entity_height(&self) -> i32 { self.entity_height }
    fn entity_depth(&self) -> i32 { self.entity_depth }
    fn can_pass_doors(&self) -> bool { self.can_pass_doors }
    fn can_open_doors(&self) -> bool { self.can_open_doors }
    fn can_float(&self) -> bool { self.can_float }
    fn can_walk_over_fences(&self) -> bool { self.can_walk_over_fences }

    fn set_can_pass_doors(&mut self, v: bool) { self.can_pass_doors = v; }
    fn set_can_open_doors(&mut self, v: bool) { self.can_open_doors = v; }
    fn set_can_float(&mut self, v: bool) { self.can_float = v; }
    fn set_can_walk_over_fences(&mut self, v: bool) { self.can_walk_over_fences = v; }

    fn prepare(&mut self, mob: &dyn MobPathing) {
        self.nodes.clear();
        self.entity_width = mth::floor(mob.bb_width() as f64 + 1.0) as i32;
        self.entity_height = mth::floor(mob.bb_height() as f64 + 1.0) as i32;
        self.entity_depth = self.entity_width;
        self.path_types_by_pos_cache.clear();
        self.collision_cache.clear();
    }

    fn done(&mut self, _mob: &dyn MobPathing) {
        self.path_types_by_pos_cache.clear();
        self.collision_cache.clear();
    }

    fn get_start(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing) -> Option<NodeRef> {
        let mut block_y = mob.get_block_y();
        let mut mutable = BlockPos::new(mob.get_x() as i32, block_y, mob.get_z() as i32);
        let mut block_state = ctx.get_block_state(mutable);
        if !mob.can_stand_on_fluid(&block_state.as_ref().map(|s| s.get_fluid_state()).unwrap_or_else(crate::world::level::block::state::fluid_state::FluidState::empty)) {
            if self.can_float() && mob.is_in_water() {
                loop {
                    let is_water_block = block_state
                        .as_ref()
                        .map(|s| s.block.id == crate::resources::identifier::Identifier::minecraft("water"))
                        .unwrap_or(false);
                    let is_water_source = block_state
                        .as_ref()
                        .map(|s| {
                            let fs = s.get_fluid_state();
                            fs.is_water() && fs.fluid.is_source
                        })
                        .unwrap_or(false);
                    if !is_water_block && !is_water_source {
                        block_y -= 1;
                        break;
                    }
                    block_y += 1;
                    mutable = BlockPos::new(mob.get_x() as i32, block_y, mob.get_z() as i32);
                    block_state = ctx.get_block_state(mutable);
                }
            } else if mob.on_ground() {
                block_y = mth::floor(mob.get_y() + 0.5);
            } else {
                mutable = BlockPos::containing(mob.get_x(), mob.get_y() + 1.0, mob.get_z());
                let min_y = ctx.level.min_build_height();
                while mutable.y > min_y {
                    block_y = mutable.y;
                    mutable = mutable.below();
                    if let Some(bs) = ctx.get_block_state(mutable) {
                        if !bs.is_air() && !bs.is_pathfindable(ctx.level, mutable, PathComputationType::Land) {
                            break;
                        }
                    }
                }
            }
        } else {
            loop {
                let fs = block_state.as_ref().map(|s| s.get_fluid_state()).unwrap_or_else(crate::world::level::block::state::fluid_state::FluidState::empty);
                if !mob.can_stand_on_fluid(&fs) {
                    break;
                }
                block_y += 1;
                mutable = BlockPos::new(mob.get_x() as i32, block_y, mob.get_z() as i32);
                block_state = ctx.get_block_state(mutable);
            }
            block_y -= 1;
        }

        let block_pos = mob.block_position();
        let center = BlockPos::new(block_pos.x, block_y, block_pos.z);
        if self.can_start_at(ctx, mob, center) {
            return Some(self.get_start_node(ctx, mob, center));
        }
        let bb = mob.bounding_box();
        for (cx, cz) in [
            (bb.min_x, bb.min_z),
            (bb.min_x, bb.max_z),
            (bb.max_x, bb.min_z),
            (bb.max_x, bb.max_z),
        ] {
            let p = BlockPos::new(mth::floor(cx), block_y, mth::floor(cz));
            if self.can_start_at(ctx, mob, p) {
                return Some(self.get_start_node(ctx, mob, p));
            }
        }
        Some(self.get_start_node(ctx, mob, center))
    }

    fn get_target(&mut self, x: f64, y: f64, z: f64) -> Target {
        self.get_target_node_at(x, y, z)
    }

    fn get_neighbors(
        &mut self,
        output: &mut Vec<NodeRef>,
        node: &NodeRef,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
    ) -> usize {
        let start_len = output.len();
        let mut i1 = 0;
        let (nx, ny, nz) = {
            let n = node.borrow();
            (n.x, n.y, n.z)
        };
        let cached_path_type = self.get_cached_path_type_mob(ctx, mob, nx, ny + 1, nz);
        let cached_path_type1 = self.get_cached_path_type_mob(ctx, mob, nx, ny, nz);
        if mob.get_pathfinding_malus(cached_path_type) >= 0.0 && cached_path_type1 != PathType::StickyHoney {
            i1 = mth::floor((1.0f32).max(mob.max_up_step()) as f64) as i32;
        }
        let floor_level = self.get_floor_level(ctx, BlockPos::new(nx, ny, nz));

        let mut reusable: [Option<NodeRef>; 4] = [None, None, None, None];
        for direction in Plane::HORIZONTAL.faces() {
            let node1 = self.find_accepted_node(
                ctx, mob,
                nx + direction.get_step_x(),
                ny,
                nz + direction.get_step_z(),
                i1,
                floor_level,
                *direction,
                cached_path_type1,
            );
            reusable[direction.get_2d_data_value() as usize] = node1.clone();
            if self.is_neighbor_valid(node1.as_ref(), node) {
                if let Some(n) = node1 {
                    output.push(n);
                }
            }
        }

        for directionx in Plane::HORIZONTAL.faces() {
            let clock_wise = directionx.get_clock_wise();
            let x_node = reusable[directionx.get_2d_data_value() as usize].clone();
            let z_node = reusable[clock_wise.get_2d_data_value() as usize].clone();
            if self.is_diagonal_valid(mob, node, x_node.as_ref(), z_node.as_ref()) {
                let node2 = self.find_accepted_node(
                    ctx, mob,
                    nx + directionx.get_step_x() + clock_wise.get_step_x(),
                    ny,
                    nz + directionx.get_step_z() + clock_wise.get_step_z(),
                    i1,
                    floor_level,
                    *directionx,
                    cached_path_type1,
                );
                if self.is_diagonal_valid_node(node2.as_ref()) {
                    if let Some(n) = node2 {
                        output.push(n);
                    }
                }
            }
        }
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
        self.get_path_type_of_mob_with(ctx, x, y, z, mob)
    }

    fn get_path_type(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        Self::get_path_type_static(ctx, x, y, z)
    }
}
