//! Перенос `net.minecraft.util.Graph`.

use std::collections::{HashMap, HashSet};

/// `Graph` — DFS-поиск циклов.
pub struct Graph;

impl Graph {
    /// `depthFirstSearch` — поиск цикла в графе.
    ///
    /// Возвращает true, если найден цикл (currentNode уже в pathSet).
    /// `graph`: узел → множество соседей. `nonCyclicalNodes` — уже проверенные
    /// ациклические узлы (мемоизация). `onNonCyclicalNodeFound` вызывается для
    /// каждого ациклического узла в порядке завершения DFS.
    pub fn depth_first_search<T, F>(
        graph: &HashMap<T, HashSet<T>>,
        non_cyclical_nodes: &mut HashSet<T>,
        path_set: &mut HashSet<T>,
        on_non_cyclical_node_found: &mut F,
        current_node: &T,
    ) -> bool
    where
        T: Eq + std::hash::Hash + Clone,
        F: FnMut(&T),
    {
        if non_cyclical_nodes.contains(current_node) {
            return false;
        }
        if path_set.contains(current_node) {
            return true;
        }
        path_set.insert(current_node.clone());
        if let Some(neighbors) = graph.get(current_node) {
            for object in neighbors {
                if Self::depth_first_search(graph, non_cyclical_nodes, path_set, on_non_cyclical_node_found, object) {
                    return true;
                }
            }
        }
        path_set.remove(current_node);
        non_cyclical_nodes.insert(current_node.clone());
        on_non_cyclical_node_found(current_node);
        false
    }
}
