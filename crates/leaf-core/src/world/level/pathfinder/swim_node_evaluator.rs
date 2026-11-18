//! Перенос `net.minecraft.world.level.pathfinder.SwimNodeEvaluator`.

use crate::world::entity::mob::MobPathing;
use crate::world::level::block::state::fluid_state::FluidState;
use crate::world::level::pathfinder::node::{Node, NodeRef};
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path_computation_type::PathComputationType;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::level::pathfinder::pathfinding_context::PathfindingContext;
use crate::world::level::pathfinder::target::Target;
use crate::tags::fluid_tags::FluidTags;
use crate::util::mth;
use std::collections::HashMap;

/// `SwimNodeEvaluator` — оценщик для водных мобов (Dolphin и т.п.).
pub struct SwimNodeEvaluator {
    /// `allowBreaching`.
    pub allow_breaching: bool,
    /// `pathTypesByPosCache`.
    path_types_by_pos_cache: HashMap<i64, PathType>,
    nodes: HashMap<i32, NodeRef>,
    entity_width: i32,
    entity_height: i32,
    entity_depth: i32,
    can_pass_doors: bool,
    can_open_doors: bool,
    can_float: bool,
    can_walk_over_fences: bool,
}

impl SwimNodeEvaluator {
    /// `new SwimNodeEvaluator(allowBreaching)`.
    pub fn new(allow_breaching: bool) -> Self {
        Self {
            allow_breaching,
            path_types_by_pos_cache: HashMap::new(),
            nodes: HashMap::new(),
            entity_width: 0,
            entity_height: 0,
            entity_depth: 0,
            can_pass_doors: true,
            can_open_doors: false,
            can_float: false,
            can_walk_over_fences: false,
        }
    }

    /// `findAcceptedNode(x, y, z)`.
    fn find_accepted_node(&mut self, ctx: &mut PathfindingContext, mob: &dyn MobPathing, x: i32, y: i32, z: i32) -> Option<NodeRef> {
        let mut node = None;
        let cached = self.get_cached_block_type(ctx, x, y, z);
        if (self.allow_breaching && cached == PathType::Breach) || cached == PathType::Water {
            let malus = mob.get_pathfinding_malus(cached);
            if malus >= 0.0 {
                let n = self.get_node(x, y, z);
                {
                    let mut nb = n.borrow_mut();
                    nb.path_type = cached;
                    nb.cost_malus = nb.cost_malus.max(malus);
                    if ctx
                        .get_block_state(crate::core::block_pos::BlockPos::new(x, y, z))
                        .map(|s| s.get_fluid_state().is_empty())
                        .unwrap_or(true)
                    {
                        nb.cost_malus += 8.0;
                    }
                }
                node = Some(n);
            }
        }
        node
    }

    /// `getCachedBlockType`.
    fn get_cached_block_type(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        let key = crate::core::block_pos::BlockPos::as_long_static(x, y, z);
        if let Some(t) = self.path_types_by_pos_cache.get(&key) {
            return *t;
        }
        let t = self.path_type_of_mob(ctx, x, y, z);
        self.path_types_by_pos_cache.insert(key, t);
        t
    }

    /// `getPathTypeOfMob` — mob-параметр в Java не используется.
    fn path_type_of_mob(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        use crate::core::block_pos::BlockPos;
        for i in 0..self.entity_width {
            for i1 in 0..self.entity_height {
                for i2 in 0..self.entity_depth {
                    let pos = BlockPos::new(x + i, y + i1, z + i2);
                    let state = ctx.get_block_state(pos);
                    let fluid_state = state
                        .as_ref()
                        .map(|s| s.get_fluid_state())
                        .unwrap_or_else(FluidState::empty);
                    let is_air_pathfindable = state
                        .as_ref()
                        .map(|s| s.is_pathfindable(ctx.level, pos, PathComputationType::Water) && s.is_air())
                        .unwrap_or(false);
                    if fluid_state.is_empty() && is_air_pathfindable {
                        return PathType::Breach;
                    }
                    if !fluid_state.is_in_named_tag(&FluidTags::water()) {
                        return PathType::Blocked;
                    }
                }
            }
        }
        let pos = BlockPos::new(x, y, z);
        let block_state = ctx.get_block_state(pos);
        match block_state {
            Some(s) if s.is_pathfindable(ctx.level, pos, PathComputationType::Water) => PathType::Water,
            _ => PathType::Blocked,
        }
    }

    /// `isNodeValid`.
    fn is_node_valid(&self, node: &Option<NodeRef>) -> bool {
        node.as_ref().map(|n| !n.borrow().closed).unwrap_or(false)
    }

    /// `hasMalus`.
    fn has_malus(&self, node: &Option<NodeRef>) -> bool {
        node.as_ref().map(|n| n.borrow().cost_malus >= 0.0).unwrap_or(false)
    }
}

impl Default for SwimNodeEvaluator {
    fn default() -> Self {
        Self::new(false)
    }
}

impl NodeEvaluator for SwimNodeEvaluator {
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
    fn set_can_float(&mut self, _v: bool) {}
    fn set_can_walk_over_fences(&mut self, v: bool) { self.can_walk_over_fences = v; }

    fn prepare(&mut self, mob: &dyn MobPathing) {
        self.nodes.clear();
        self.entity_width = mth::floor(mob.bb_width() as f64 + 1.0) as i32;
        self.entity_height = mth::floor(mob.bb_height() as f64 + 1.0) as i32;
        self.entity_depth = self.entity_width;
        self.path_types_by_pos_cache.clear();
    }

    fn done(&mut self, _mob: &dyn MobPathing) {
        self.path_types_by_pos_cache.clear();
    }

    fn get_start(&mut self, _ctx: &mut PathfindingContext, mob: &dyn MobPathing) -> Option<NodeRef> {
        let bb = mob.bounding_box();
        Some(self.get_node(
            mth::floor(bb.min_x),
            mth::floor(bb.min_y + 0.5),
            mth::floor(bb.min_z),
        ))
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
        let (nx, ny, nz) = {
            let n = node.borrow();
            (n.x, n.y, n.z)
        };
        // 6 направлений (все 3D — Direction.values())
        let dirs: [(i32, i32, i32); 6] = [
            (0, -1, 0), (-1, 0, 0), (1, 0, 0), (0, 0, -1), (0, 0, 1), (0, 1, 0),
        ];
        let mut map: HashMap<usize, Option<NodeRef>> = HashMap::new();
        for (idx, (dx, dy, dz)) in dirs.iter().enumerate() {
            let n = self.find_accepted_node(ctx, mob, nx + dx, ny + dy, nz + dz);
            map.insert(idx, n.clone());
            if self.is_node_valid(&n) {
                if let Some(nn) = n {
                    output.push(nn);
                }
            }
        }
        // Горизонтальные диагонали: Direction.Plane.HORIZONTAL порядок —
        // NORTH, EAST, SOUTH, WEST (как в Direction). getClockWise.
        // Индексы dirs: 3=North(z-1),4=South(z+1),1=West(x-1),2=East(x+1)?
        // Используем семантику Plane.HORIZONTAL: [SOUTH? ...]. Ваниль:
        // for (Direction d : HORIZONTAL) { cw = d.getClockWise();
        //   node(x + d.dx + cw.dx, y, z + d.dz + cw.dz) }
        // Порядок HORIZONTAL в 1.21: NORTH, EAST, SOUTH, WEST? На самом деле
        // Direction.Plane.HORIZONTAL = [NORTH, EAST, SOUTH, WEST].
        // В нашем массиве dirs порядок произвольный — задаём явные пары.
        let horiz: [(i32, i32, i32, i32); 4] = [
            // (dir_dx,dir_dz, cw_dx,cw_dz) — NORTH→EAST, EAST→SOUTH, SOUTH→WEST, WEST→NORTH
            (0, -1, 1, 0),
            (1, 0, 0, 1),
            (0, 1, -1, 0),
            (-1, 0, 0, -1),
        ];
        // Индекс в map по направлению: нужен поиск по (dx,dy,dz)
        let idx_of = |dx: i32, dy: i32, dz: i32| -> Option<usize> {
            dirs.iter().position(|d| *d == (dx, dy, dz))
        };
        for (ddx, ddz, cdx, cdz) in horiz {
            let i1 = idx_of(ddx, 0, ddz);
            let i2 = idx_of(cdx, 0, cdz);
            if let (Some(a), Some(b)) = (i1, i2) {
                if self.has_malus(&map.get(&a).cloned().unwrap_or(None))
                    && self.has_malus(&map.get(&b).cloned().unwrap_or(None))
                {
                    let n = self.find_accepted_node(ctx, mob, nx + ddx + cdx, ny, nz + ddz + cdz);
                    if self.is_node_valid(&n) {
                        if let Some(nn) = n {
                            output.push(nn);
                        }
                    }
                }
            }
        }
        output.len() - start_len
    }

    fn get_path_type_of_mob(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32, _mob: &dyn MobPathing) -> PathType {
        self.path_type_of_mob(ctx, x, y, z)
    }

    fn get_path_type(&mut self, ctx: &mut PathfindingContext, x: i32, y: i32, z: i32) -> PathType {
        self.path_type_of_mob(ctx, x, y, z)
    }
}

// Node используется для NodeRef-типов (импорт для читаемости).
#[allow(dead_code)]
fn _unused(_: Node) {}
