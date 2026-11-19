//! Перенос `net.minecraft.world.level.pathfinder.Path`.

use crate::core::block_pos::BlockPos;
use crate::world::level::pathfinder::node::{Node, NodeRef};
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::rc::Rc;

/// `Path` — найденный путь.
#[derive(Debug, Clone)]
pub struct Path {
    /// `nodes`.
    pub nodes: Vec<NodeRef>,
    /// `debugData` — только при captureDebug.
    pub debug_data: Option<PathDebugData>,
    /// `nextNodeIndex`.
    pub next_node_index: usize,
    /// `target`.
    pub target: BlockPos,
    /// `distToTarget`.
    pub dist_to_target: f32,
    /// `reached`.
    pub reached: bool,
    /// `isProcessed` — путь завершил асинхронную обработку
    /// (Kaiiju async path processing); синхронно созданные = true.
    pub processed: bool,
}

/// `Path.DebugData`.
#[derive(Debug, Clone)]
pub struct PathDebugData {
    /// `openSet`.
    pub open_set: Vec<NodeRef>,
    /// `closedSet`.
    pub closed_set: Vec<NodeRef>,
    /// `targetNodes`.
    pub target_nodes: Vec<crate::world::level::pathfinder::target::Target>,
}

impl Path {
    /// `new Path(nodes, target, reached)`.
    pub fn new(nodes: Vec<NodeRef>, target: BlockPos, reached: bool) -> Self {
        let dist_to_target = nodes
            .last()
            .map(|n| n.borrow().distance_manhattan_pos(&target))
            .unwrap_or(f32::MAX);
        Self {
            nodes,
            debug_data: None,
            next_node_index: 0,
            target,
            dist_to_target,
            reached,
            processed: true,
        }
    }

    /// `advance`.
    pub fn advance(&mut self) {
        self.next_node_index += 1;
    }

    /// `notStarted`.
    pub fn not_started(&self) -> bool {
        self.next_node_index == 0
    }

    /// `isDone`.
    pub fn is_done(&self) -> bool {
        self.next_node_index >= self.nodes.len()
    }

    /// `getEndNode`.
    pub fn end_node(&self) -> Option<NodeRef> {
        self.nodes.last().cloned()
    }

    /// `getNode(index)`.
    pub fn node(&self, index: usize) -> NodeRef {
        self.nodes[index].clone()
    }

    /// `truncateNodes(length)`.
    pub fn truncate_nodes(&mut self, length: usize) {
        if self.nodes.len() > length {
            self.nodes.truncate(length);
        }
    }

    /// `replaceNode(index, node)`.
    pub fn replace_node(&mut self, index: usize, node: NodeRef) {
        self.nodes[index] = node;
    }

    /// `getNodeCount`.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// `getNextNodeIndex`.
    pub fn next_node_index(&self) -> usize {
        self.next_node_index
    }

    /// `setNextNodeIndex`.
    pub fn set_next_node_index(&mut self, index: usize) {
        self.next_node_index = index;
    }

    /// `getEntityPosAtNode(entity, index)` — центр узла +
    /// половина ширины сущности.
    pub fn entity_pos_at_node(&self, bb_width: f32, index: usize) -> Vec3 {
        let n = self.nodes[index].borrow();
        let d = n.x as f64 + ((bb_width + 1.0).floor() as i32) as f64 * 0.5;
        let d1 = n.y as f64;
        let d2 = n.z as f64 + ((bb_width + 1.0).floor() as i32) as f64 * 0.5;
        Vec3::new(d, d1, d2)
    }

    /// `getNodePos(index)`.
    pub fn node_pos(&self, index: usize) -> BlockPos {
        let n = self.nodes[index].borrow();
        BlockPos::new(n.x, n.y, n.z)
    }

    /// `getNextEntityPos(entity)`.
    pub fn next_entity_pos(&self, bb_width: f32) -> Vec3 {
        self.entity_pos_at_node(bb_width, self.next_node_index)
    }

    /// `getNextNodePos`.
    pub fn next_node_pos(&self) -> BlockPos {
        self.node_pos(self.next_node_index)
    }

    /// `getNextNode`.
    pub fn next_node(&self) -> NodeRef {
        self.nodes[self.next_node_index].clone()
    }

    /// `getPreviousNode`.
    pub fn previous_node(&self) -> Option<NodeRef> {
        if self.next_node_index > 0 {
            Some(self.nodes[self.next_node_index - 1].clone())
        } else {
            None
        }
    }

    /// `sameAs`.
    pub fn same_as(&self, other: Option<&Path>) -> bool {
        match other {
            Some(o) => {
                self.nodes.len() == o.nodes.len()
                    && self
                        .nodes
                        .iter()
                        .zip(o.nodes.iter())
                        .all(|(a, b)| {
                            let (a, b) = (a.borrow(), b.borrow());
                            a.x == b.x && a.y == b.y && a.z == b.z
                        })
            }
            None => false,
        }
    }

    /// `canReach`.
    pub fn can_reach(&self) -> bool {
        self.reached
    }

    /// `getTarget`.
    pub fn target_pos(&self) -> BlockPos {
        self.target
    }

    /// `getDistToTarget`.
    pub fn dist_to_target(&self) -> f32 {
        self.dist_to_target
    }

    /// `setDebug`.
    pub fn set_debug(&mut self, open_set: Vec<NodeRef>, closed_set: Vec<NodeRef>, targets: Vec<crate::world::level::pathfinder::target::Target>) {
        self.debug_data = Some(PathDebugData {
            open_set,
            closed_set,
            target_nodes: targets,
        });
    }

    /// `copy`.
    pub fn copy(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            debug_data: None,
            next_node_index: self.next_node_index,
            target: self.target,
            dist_to_target: self.dist_to_target,
            reached: self.reached,
            processed: self.processed,
        }
    }

    /// `toString`.
    pub fn to_string(&self) -> String {
        format!("Path(length={})", self.nodes.len())
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        self.next_node_index == other.next_node_index
            && self.reached == other.reached
            && self.target == other.target
            && self.same_as(Some(other))
    }
}
