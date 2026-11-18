//! Перенос `net.minecraft.world.level.pathfinder.BinaryHeap`.

use super::node::{Node, NodeRef};
use std::cell::RefCell;
use std::rc::Rc;

/// `BinaryHeap` — бинарная куча узлов по `f`.
pub struct BinaryHeap {
    heap: Vec<NodeRef>,
    size: usize,
}

impl BinaryHeap {
    /// `new`.
    pub fn new() -> Self {
        Self {
            heap: Vec::with_capacity(128),
            size: 0,
        }
    }

    /// `insert`.
    pub fn insert(&mut self, node: NodeRef) {
        {
            let n = node.borrow();
            assert!(n.heap_idx < 0, "OW KNOWS!");
        }
        if self.size == self.heap.len() {
            self.heap.push(node.clone());
        }
        self.heap[self.size] = node.clone();
        node.borrow_mut().heap_idx = self.size as i32;
        self.size += 1;
        self.up_heap(self.size - 1);
    }

    /// `clear`.
    pub fn clear(&mut self) {
        self.size = 0;
    }

    /// `peek`.
    pub fn peek(&self) -> Option<NodeRef> {
        self.heap.first().cloned()
    }

    /// `pop`.
    pub fn pop(&mut self) -> Option<NodeRef> {
        if self.size == 0 {
            return None;
        }
        let node = self.heap[0].clone();
        self.size -= 1;
        self.heap[0] = self.heap[self.size].clone();
        if self.size > 0 {
            self.down_heap(0);
        }
        node.borrow_mut().heap_idx = -1;
        Some(node)
    }

    /// `remove`.
    pub fn remove(&mut self, node: &NodeRef) {
        let idx = node.borrow().heap_idx;
        assert!(idx >= 0, "node not in heap");
        let idx = idx as usize;
        self.size -= 1;
        self.heap[idx] = self.heap[self.size].clone();
        if self.size > idx {
            let f_node = self.heap[idx].borrow().f;
            let f_removed = node.borrow().f;
            if f_node < f_removed {
                self.up_heap(idx);
            } else {
                self.down_heap(idx);
            }
        }
        node.borrow_mut().heap_idx = -1;
    }

    /// `changeCost`.
    pub fn change_cost(&mut self, node: &NodeRef, cost: f32) {
        let old_f = node.borrow().f;
        node.borrow_mut().f = cost;
        let idx = node.borrow().heap_idx as usize;
        if cost < old_f {
            self.up_heap(idx);
        } else {
            self.down_heap(idx);
        }
    }

    /// `size`.
    pub fn size(&self) -> usize {
        self.size
    }

    /// `isEmpty`.
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// `getHeap`.
    pub fn get_heap(&self) -> Vec<NodeRef> {
        self.heap[..self.size].to_vec()
    }

    fn up_heap(&mut self, mut index: usize) {
        let node = self.heap[index].clone();
        let f = node.borrow().f;
        while index > 0 {
            let i = (index - 1) >> 1;
            let node1 = self.heap[i].clone();
            if !(f < node1.borrow().f) {
                break;
            }
            self.heap[index] = node1.clone();
            node1.borrow_mut().heap_idx = index as i32;
            index = i;
        }
        node.borrow_mut().heap_idx = index as i32;
        self.heap[index] = node;
    }

    fn down_heap(&mut self, mut index: usize) {
        let node = self.heap[index].clone();
        let f = node.borrow().f;
        loop {
            let i = 1 + (index << 1);
            let i1 = i + 1;
            if i >= self.size {
                break;
            }
            let node1 = self.heap[i].clone();
            let f1 = node1.borrow().f;
            let (node2, f2) = if i1 >= self.size {
                (None, f32::INFINITY)
            } else {
                let n = self.heap[i1].clone();
                let ff = n.borrow().f;
                (Some(n), ff)
            };
            if f1 < f2 {
                if !(f1 < f) {
                    break;
                }
                self.heap[index] = node1.clone();
                node1.borrow_mut().heap_idx = index as i32;
                index = i;
            } else {
                let Some(node2) = node2 else { break };
                if !(f2 < f) {
                    break;
                }
                self.heap[index] = node2.clone();
                node2.borrow_mut().heap_idx = index as i32;
                index = i1;
            }
        }
        node.borrow_mut().heap_idx = index as i32;
        self.heap[index] = node;
    }
}

impl Default for BinaryHeap {
    fn default() -> Self {
        Self::new()
    }
}
