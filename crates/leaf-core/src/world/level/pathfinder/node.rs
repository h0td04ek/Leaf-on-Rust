//! Перенос `net.minecraft.world.level.pathfinder.Node`.
//!
//! Узлы разделяются через `Rc<RefCell<Node>>` — `cameFrom` и
//! ссылки из `BinaryHeap`/`Target` работают как в Java
//! (ссылочная семантика).

use crate::core::block_pos::BlockPos;
use crate::util::mth;
use crate::world::level::pathfinder::path_type::PathType;
use crate::world::phys::vec3::Vec3;
use std::cell::RefCell;
use std::rc::Rc;

/// `Node` — узел поиска пути.
#[derive(Debug)]
pub struct Node {
    /// `x`, `y`, `z`.
    pub x: i32,
    pub y: i32,
    pub z: i32,
    /// `hash` — кэшированный хэш.
    pub hash: i32,
    /// `heapIdx` — индекс в BinaryHeap (-1 = не в куче).
    pub heap_idx: i32,
    /// `g`, `h`, `f`.
    pub g: f32,
    pub h: f32,
    pub f: f32,
    /// `cameFrom`.
    pub came_from: Option<Rc<RefCell<Node>>>,
    /// `closed`.
    pub closed: bool,
    /// `walkedDistance`.
    pub walked_distance: f32,
    /// `costMalus`.
    pub cost_malus: f32,
    /// `type`.
    pub path_type: PathType,
}

/// Общий тип узла.
pub type NodeRef = Rc<RefCell<Node>>;

impl Node {
    /// `new Node(x, y, z)`.
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        let hash = Self::create_hash(x, y, z);
        Self {
            x,
            y,
            z,
            hash,
            heap_idx: -1,
            g: 0.0,
            h: 0.0,
            f: 0.0,
            came_from: None,
            closed: false,
            walked_distance: 0.0,
            cost_malus: 0.0,
            path_type: PathType::Blocked,
        }
    }

    /// `Node` как `Rc<RefCell>`.
    pub fn node_ref(x: i32, y: i32, z: i32) -> NodeRef {
        Rc::new(RefCell::new(Self::new(x, y, z)))
    }

    /// `createHash`.
    pub fn create_hash(x: i32, y: i32, z: i32) -> i32 {
        (y & 0xFF) | ((x & 32767) << 8) | ((z & 32767) << 24)
            | if x < 0 { i32::MIN } else { 0 }
            | if z < 0 { 32768 } else { 0 }
    }

    /// `cloneAndMove`.
    pub fn clone_and_move(&self, x: i32, y: i32, z: i32) -> Self {
        let mut node = Self::new(x, y, z);
        node.heap_idx = self.heap_idx;
        node.g = self.g;
        node.h = self.h;
        node.f = self.f;
        node.came_from = self.came_from.clone();
        node.closed = self.closed;
        node.walked_distance = self.walked_distance;
        node.cost_malus = self.cost_malus;
        node.path_type = self.path_type;
        node
    }

    /// `distanceTo(Node)`.
    pub fn distance_to(&self, other: &Node) -> f32 {
        let fx = (other.x - self.x) as f32;
        let fy = (other.y - self.y) as f32;
        let fz = (other.z - self.z) as f32;
        mth::sqrt_f32(fx * fx + fy * fy + fz * fz)
    }

    /// `distanceTo(BlockPos)`.
    pub fn distance_to_pos(&self, pos: &BlockPos) -> f32 {
        let fx = (pos.x - self.x) as f32;
        let fy = (pos.y - self.y) as f32;
        let fz = (pos.z - self.z) as f32;
        mth::sqrt_f32(fx * fx + fy * fy + fz * fz)
    }

    /// `distanceToXZ`.
    pub fn distance_to_xz(&self, other: &Node) -> f32 {
        let fx = (other.x - self.x) as f32;
        let fz = (other.z - self.z) as f32;
        mth::sqrt_f32(fx * fx + fz * fz)
    }

    /// `distanceToSqr(Node)`.
    pub fn distance_to_sqr(&self, other: &Node) -> f32 {
        let fx = (other.x - self.x) as f32;
        let fy = (other.y - self.y) as f32;
        let fz = (other.z - self.z) as f32;
        fx * fx + fy * fy + fz * fz
    }

    /// `distanceToSqr(BlockPos)`.
    pub fn distance_to_sqr_pos(&self, pos: &BlockPos) -> f32 {
        let fx = (pos.x - self.x) as f32;
        let fy = (pos.y - self.y) as f32;
        let fz = (pos.z - self.z) as f32;
        fx * fx + fy * fy + fz * fz
    }

    /// `distanceManhattan(Node)`.
    pub fn distance_manhattan(&self, other: &Node) -> f32 {
        ((other.x - self.x).abs() + (other.y - self.y).abs() + (other.z - self.z).abs()) as f32
    }

    /// `distanceManhattan(BlockPos)`.
    pub fn distance_manhattan_pos(&self, pos: &BlockPos) -> f32 {
        ((pos.x - self.x).abs() + (pos.y - self.y).abs() + (pos.z - self.z).abs()) as f32
    }

    /// `asBlockPos`.
    pub fn as_block_pos(&self) -> BlockPos {
        BlockPos::new(self.x, self.y, self.z)
    }

    /// `asVec3`.
    pub fn as_vec3(&self) -> Vec3 {
        Vec3::new(self.x as f64, self.y as f64, self.z as f64)
    }

    /// `inOpenSet`.
    pub fn in_open_set(&self) -> bool {
        self.heap_idx >= 0
    }
}

/// `Node.equals` — по hash и координатам.
pub fn node_eq(a: &Node, b: &Node) -> bool {
    a.hash == b.hash && a.x == b.x && a.y == b.y && a.z == b.z
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        node_eq(self, other)
    }
}

impl Eq for Node {}

impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node{{x={}, y={}, z={}}}", self.x, self.y, self.z)
    }
}
