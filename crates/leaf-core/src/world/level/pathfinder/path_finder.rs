//! Перенос `net.minecraft.world.level.pathfinder.PathFinder`.

use crate::core::block_pos::BlockPos;
use crate::world::entity::mob::MobPathing;
use crate::world::level::level_reader::LevelReader;
use crate::world::level::pathfinder::binary_heap::BinaryHeap;
use crate::world::level::pathfinder::node::NodeRef;
use crate::world::level::pathfinder::node_evaluator::NodeEvaluator;
use crate::world::level::pathfinder::path::Path;
use crate::world::level::pathfinder::pathfinding_context::PathfindingContext;
use crate::world::level::pathfinder::path_type_cache::PathTypeCache;
use crate::world::level::pathfinder::target::Target;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// `Target` в разделяемом виде (`Rc<RefCell>` — Java-ссылка).
pub type TargetRef = Rc<RefCell<Target>>;

/// `PathFinder`.
pub struct PathFinder {
    /// `neighbors` — переиспользуемый буфер.
    neighbors: Vec<NodeRef>,
    /// `maxVisitedNodes`.
    max_visited_nodes: i32,
    /// `nodeEvaluator` — `Rc<RefCell>` т.к. navigation
    /// переиспользует тот же evaluator для getNodeEvaluator/
    /// setCanFloat (Java — общая ссылка).
    pub node_evaluator: Rc<RefCell<dyn NodeEvaluator>>,
    /// `openSet`.
    open_set: BinaryHeap,
    /// `captureDebug`.
    capture_debug: bool,
}

/// Цель поиска: Target + исходная BlockPos (как Java
/// `Map<Target, BlockPos>`).
struct SearchTarget {
    target: TargetRef,
    pos: BlockPos,
}

impl PathFinder {
    /// `new PathFinder(nodeEvaluator, maxVisitedNodes)`.
    pub fn new(node_evaluator: Rc<RefCell<dyn NodeEvaluator>>, max_visited_nodes: i32) -> Self {
        Self {
            neighbors: Vec::with_capacity(32),
            max_visited_nodes,
            node_evaluator,
            open_set: BinaryHeap::new(),
            capture_debug: false,
        }
    }

    /// `setCaptureDebug`.
    pub fn set_capture_debug(&mut self, capture_debug: bool) {
        self.capture_debug = capture_debug;
    }

    /// `setMaxVisitedNodes`.
    pub fn set_max_visited_nodes(&mut self, max_visited_nodes: i32) {
        self.max_visited_nodes = max_visited_nodes;
    }

    /// `findPath(region, mob, targets, maxRange, reachRange,
    /// maxVisitedNodesMultiplier)`.
    #[allow(clippy::too_many_arguments)]
    pub fn find_path(
        &mut self,
        level: &dyn LevelReader,
        cache: Option<&PathTypeCache>,
        mob: &dyn MobPathing,
        targets: &HashSet<BlockPos>,
        max_range: f32,
        reach_range: i32,
        max_visited_nodes_multiplier: f32,
    ) -> Option<Path> {
        self.open_set.clear();
        self.node_evaluator.borrow_mut().prepare(mob);
        let mut ctx = PathfindingContext::new(level, cache, mob.block_position());
        let start = match self.node_evaluator.borrow_mut().get_start(&mut ctx, mob) {
            Some(s) => s,
            None => {
                self.node_evaluator.borrow_mut().done(mob);
                return None;
            }
        };
        let mut map: HashMap<TargetKey, SearchTarget> = HashMap::new();
        for pos in targets {
            let t = self.node_evaluator.borrow_mut().get_target(pos.x as f64, pos.y as f64, pos.z as f64);
            let key = TargetKey::new(&t);
            map.insert(key, SearchTarget { target: Rc::new(RefCell::new(t)), pos: *pos });
        }
        let path = self.find_path_inner(&mut ctx, mob, start, &map, max_range, reach_range, max_visited_nodes_multiplier);
        self.node_evaluator.borrow_mut().done(mob);
        path
    }

    /// Внутренний `findPath(Node, Map<Target,BlockPos>, ...)`.
    #[allow(clippy::too_many_arguments)]
    fn find_path_inner(
        &mut self,
        ctx: &mut PathfindingContext,
        mob: &dyn MobPathing,
        start: NodeRef,
        targets: &HashMap<TargetKey, SearchTarget>,
        max_range: f32,
        reach_range: i32,
        max_visited_nodes_multiplier: f32,
    ) -> Option<Path> {
        let target_keys: Vec<TargetKey> = targets.keys().cloned().collect();
        {
            let mut s = start.borrow_mut();
            s.g = 0.0;
            s.h = self.get_best_h_node_ref(&start, targets);
            s.f = s.h;
        }
        self.open_set.clear();
        self.open_set.insert(start.clone());
        let as_boolean = self.capture_debug;
        let mut set1: Vec<NodeRef> = Vec::new();
        let mut set2: Vec<TargetKey> = Vec::new();
        let i1 = (self.max_visited_nodes as f32 * max_visited_nodes_multiplier) as i32;
        let mut i = 0;

        while !self.open_set.is_empty() {
            i += 1;
            if i >= i1 {
                break;
            }
            let node1 = match self.open_set.pop() {
                Some(n) => n,
                None => break,
            };
            node1.borrow_mut().closed = true;

            for key in &target_keys {
                let st = &targets[key];
                let tn = st.target.borrow().as_node();
                if node1.borrow().distance_manhattan(&tn) <= reach_range as f32 {
                    st.target.borrow_mut().set_reached();
                    if !set2.contains(key) {
                        set2.push(*key);
                    }
                }
            }
            if !set2.is_empty() {
                break;
            }
            if as_boolean {
                set1.push(node1.clone());
            }
            // `!(node1.distanceTo(node) >= maxRange)` — node здесь
            // стартовый узел (first param findPath).
            let dist_to_start = {
                let n1 = node1.borrow();
                let s = start.borrow();
                n1.distance_to(&s)
            };
            if dist_to_start < max_range {
                self.neighbors.clear();
                let count = self.node_evaluator.borrow_mut().get_neighbors(&mut self.neighbors, &node1, ctx, mob);
                let neighbors: Vec<NodeRef> = self.neighbors[..count.min(self.neighbors.len())].to_vec();
                for node2 in neighbors {
                    let f = node1.borrow().distance_to(&node2.borrow());
                    let walked = node1.borrow().walked_distance + f;
                    node2.borrow_mut().walked_distance = walked;
                    let f1 = node1.borrow().g + f + node2.borrow().cost_malus;
                    let in_open = node2.borrow().in_open_set();
                    let g2 = node2.borrow().g;
                    if node2.borrow().walked_distance < max_range && (!in_open || f1 < g2) {
                        node2.borrow_mut().came_from = Some(node1.clone());
                        node2.borrow_mut().g = f1;
                        let h = self.get_best_h_node_ref(&node2, targets);
                        node2.borrow_mut().h = h * 1.5;
                        if node2.borrow().in_open_set() {
                            let cost = node2.borrow().g + node2.borrow().h;
                            self.open_set.change_cost(&node2, cost);
                        } else {
                            let fcost = node2.borrow().g + node2.borrow().h;
                            node2.borrow_mut().f = fcost;
                            self.open_set.insert(node2.clone());
                        }
                    }
                }
            }
        }

        // Реконструкция путей.
        let reached: Vec<&SearchTarget> = set2.iter().map(|k| &targets[k]).collect();
        let option = if !reached.is_empty() {
            reached
                .iter()
                .filter_map(|st| {
                    st.target.borrow().best_node().map(|bn| {
                        self.reconstruct_path(bn, st.pos, true)
                    })
                })
                .min_by_key(|p| p.node_count())
        } else {
            target_keys
                .iter()
                .filter_map(|k| {
                    let st = &targets[k];
                    st.target.borrow().best_node().map(|bn| {
                        self.reconstruct_path(bn, st.pos, false)
                    })
                })
                .min_by(|a, b| {
                    a.dist_to_target()
                        .partial_cmp(&b.dist_to_target())
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then(a.node_count().cmp(&b.node_count()))
                })
        };
        let mut path = option?;
        if as_boolean {
            let open_heap = self.open_set.get_heap();
            let target_nodes: Vec<Target> = target_keys
                .iter()
                .map(|k| targets[k].target.borrow().clone_shallow())
                .collect();
            path.set_debug(open_heap, set1, target_nodes);
        }
        Some(path)
    }

    /// `getBestH` для NodeRef — обновляет bestNode целей.
    fn get_best_h_node_ref(&self, node: &NodeRef, targets: &HashMap<TargetKey, SearchTarget>) -> f32 {
        let n = node.borrow();
        let mut f = f32::MAX;
        for st in targets.values() {
            let f1 = n.distance_to(&st.target.borrow().as_node());
            st.target.borrow_mut().update_best(f1, node.clone());
            f = f.min(f1);
        }
        f
    }

    /// `distance(first, second)`.
    fn distance(&self, first: &NodeRef, second: &NodeRef) -> f32 {
        first.borrow().distance_to(&second.borrow())
    }

    /// `reconstructPath`.
    fn reconstruct_path(&self, node: NodeRef, target_pos: BlockPos, reaches_target: bool) -> Path {
        let mut list = vec![node.clone()];
        let mut node1 = node;
        loop {
            let next = node1.borrow().came_from.clone();
            match next {
                Some(cf) => {
                    list.insert(0, cf.clone());
                    node1 = cf;
                }
                None => break,
            }
        }
        Path::new(list, target_pos, reaches_target)
    }
}

/// Вспомогательная — позиция цели (не используется).
/// Ключ цели по координатам.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct TargetKey {
    x: i32,
    y: i32,
    z: i32,
}

impl TargetKey {
    fn new(t: &Target) -> Self {
        Self { x: t.x, y: t.y, z: t.z }
    }
}
