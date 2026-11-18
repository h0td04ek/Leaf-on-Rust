//! Перенос `net.minecraft.world.level.pathfinder.Target`.

use super::node::{Node, NodeRef};
use std::cell::RefCell;
use std::rc::Rc;

/// `Target` — целевой узел с лучшей эвристикой.
#[derive(Debug, Clone)]
pub struct Target {
    /// Узел-цель (координаты наследуются от Node).
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// `bestHeuristic`.
    pub best_heuristic: f32,
    /// `bestNode`.
    pub best_node: Option<NodeRef>,
    /// `reached`.
    pub reached: bool,
}

impl Target {
    /// `new Target(Node)`.
    pub fn from_node(node: &Node) -> Self {
        Self {
            x: node.x,
            y: node.y,
            z: node.z,
            best_heuristic: f32::MAX,
            best_node: None,
            reached: false,
        }
    }

    /// `new Target(x, y, z)`.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self {
            x,
            y,
            z,
            best_heuristic: f32::MAX,
            best_node: None,
            reached: false,
        }
    }

    /// `updateBest`.
    pub fn update_best(&mut self, heuristic: f32, node: NodeRef) {
        if heuristic < self.best_heuristic {
            self.best_heuristic = heuristic;
            self.best_node = Some(node);
        }
    }

    /// `getBestNode`.
    pub fn best_node(&self) -> Option<NodeRef> {
        self.best_node.clone()
    }

    /// `setReached`.
    pub fn set_reached(&mut self) {
        self.reached = true;
    }

    /// `isReached`.
    pub fn is_reached(&self) -> bool {
        self.reached
    }

    /// Как `Node` для сравнения расстояний.
    pub fn as_node(&self) -> Node {
        Node::new(self.x, self.y, self.z)
    }

    /// Удобный доступ для `distanceManhattan` от Node.
    pub fn distance_manhattan_from(&self, node: &Node) -> f32 {
        node.distance_manhattan(&self.as_node())
    }

    /// `Rc<RefCell<Node>>`-копия для heap-операций (Target сам
    /// не участвует в куче).
    pub fn node_ref(&self) -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(self.as_node()))
    }

    /// Поверхностная копия для debug-вывода (bestNode не переносится —
    /// в Java DebugData хранит сами Target-объекты).
    pub fn clone_shallow(&self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z,
            best_heuristic: self.best_heuristic,
            best_node: self.best_node.clone(),
            reached: self.reached,
        }
    }
}
