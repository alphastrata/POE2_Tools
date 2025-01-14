//$ src\data\poe_tree\pathfinding.rs
use super::consts::*;
use super::coordinates::Group;
use super::skills::PassiveSkill;
use super::stats::{Operand, Stat};
use super::type_wrappings::{EdgeId, GroupId, NodeId};

use serde_json::Value;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::{collections::VecDeque, fs};

use super::PassiveTree;

#[derive(Debug, Clone, Eq, PartialEq)]
struct NodeCost {
    node_id: NodeId,
    cost: usize,
}

// Implement ordering for BinaryHeap (min-heap behavior)
impl Ord for NodeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost) // Reverse to get min-heap
    }
}
impl PartialOrd for NodeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'data> PassiveTree<'data> {
    pub fn find_shortest_path(&self, start: usize, target: usize) -> Vec<NodeId> {
        // let mut distances: HashMap<usize, usize> = HashMap::new();
        // let mut predecessors: HashMap<usize, usize> = HashMap::new();
        // let mut priority_queue = BinaryHeap::new();

        // // Initialize distances
        // for &node_id in self.nodes.keys() {
        //     distances.insert(node_id, usize::MAX);
        // }
        // distances.insert(start, 0);

        // // Start with the source node
        // priority_queue.push(NodeCost {
        //     node_id: start,
        //     cost: 0,
        // });

        // while let Some(NodeCost { node_id, cost }) = priority_queue.pop() {
        //     // Stop if we reached the target
        //     if node_id == target {
        //         break;
        //     }

        //     // Skip if this path is not optimal
        //     if cost > *distances.get(&node_id).unwrap_or(&usize::MAX) {
        //         continue;
        //     }

        //     // Explore neighbors
        //     if let Some(node) = self.nodes.get(&node_id) {
        //         for &neighbor in &node.connections {
        //             let new_cost = cost + 1; // Assume unweighted edges (cost = 1)
        //             if new_cost < *distances.get(&neighbor).unwrap_or(&usize::MAX) {
        //                 distances.insert(neighbor, new_cost);
        //                 predecessors.insert(neighbor, node_id);
        //                 priority_queue.push(NodeCost {
        //                     node_id: neighbor,
        //                     cost: new_cost,
        //                 });
        //             }
        //         }
        //     }

        // }

        // Reconstruct path from `predecessors`
        // let mut path = Vec::new();
        // let mut current = target;
        // while let Some(&prev) = predecessors.get(&current) {
        //     path.push(current);
        //     current = prev;
        //     if current == start {
        //         path.push(start);
        //         path.reverse();
        //         return path;
        //     }
        // }

        Vec::new() // No path found
    }

    pub fn find_path_with_limit(
        &self,
        start: usize,
        target: usize,
        explored_paths: &mut Vec<Vec<usize>>,
    ) -> Vec<usize> {
        // find_path_with_limit(self, start, target)

        vec![]
    }
}

/// Count how many edges from `start` to `node` by walking `came_from`.
fn distance_to_start(came_from: &HashMap<usize, usize>, mut node: usize) -> usize {
    let mut dist = 0;
    while let Some(&parent) = came_from.get(&node) {
        if parent == node {
            break; // Reached the start
        }
        node = parent;
        dist += 1;
    }
    dist
}
impl<'data> PassiveTree<'data> {
    pub fn fuzzy_search_nodes(&self, query: &str) -> Vec<usize> {
        _fuzzy_search_nodes(self, query)
    }
}

fn _fuzzy_search_nodes(data: &PassiveTree, query: &str) -> Vec<usize> {
    data.nodes
        .iter()
        .filter(|(_, node)| node.name.to_lowercase().contains(&query.to_lowercase()))
        .map(|(id, _)| *id)
        .collect()
}
/// Rebuild the path backward from `target` to `start`, then reverse it.
fn rebuild_path(came_from: &HashMap<usize, usize>, start: usize, target: usize) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current = target;
    while current != start {
        path.push(current);
        current = came_from[&current];
    }
    path.push(start);
    path.reverse();
    path
}
//$ src\data\poe_tree\pathfinding.rs
use super::consts::*;
use super::coordinates::Group;
use super::skills::PassiveSkill;
use super::stats::{Operand, Stat};
use serde_json::Value;

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::{collections::VecDeque, fs};

use super::PassiveTree;
pub type GroupId = usize;
pub type NodeId = usize;

#[derive(Debug, Clone, Eq, PartialEq)]
struct NodeCost {
    node_id: NodeId,
    cost: usize,
}

// Implement ordering for BinaryHeap (min-heap behavior)
impl Ord for NodeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost) // Reverse to get min-heap
    }
}
impl PartialOrd for NodeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'data> PassiveTree<'data> {
    pub fn find_shortest_path(&self, start: usize, target: usize) -> Vec<NodeId> {
        // let mut distances: HashMap<usize, usize> = HashMap::new();
        // let mut predecessors: HashMap<usize, usize> = HashMap::new();
        // let mut priority_queue = BinaryHeap::new();

        // // Initialize distances
        // for &node_id in self.nodes.keys() {
        //     distances.insert(node_id, usize::MAX);
        // }
        // distances.insert(start, 0);

        // // Start with the source node
        // priority_queue.push(NodeCost {
        //     node_id: start,
        //     cost: 0,
        // });

        // while let Some(NodeCost { node_id, cost }) = priority_queue.pop() {
        //     // Stop if we reached the target
        //     if node_id == target {
        //         break;
        //     }

        //     // Skip if this path is not optimal
        //     if cost > *distances.get(&node_id).unwrap_or(&usize::MAX) {
        //         continue;
        //     }

        //     // Explore neighbors
        //     if let Some(node) = self.nodes.get(&node_id) {
        //         for &neighbor in &node.connections {
        //             let new_cost = cost + 1; // Assume unweighted edges (cost = 1)
        //             if new_cost < *distances.get(&neighbor).unwrap_or(&usize::MAX) {
        //                 distances.insert(neighbor, new_cost);
        //                 predecessors.insert(neighbor, node_id);
        //                 priority_queue.push(NodeCost {
        //                     node_id: neighbor,
        //                     cost: new_cost,
        //                 });
        //             }
        //         }
        //     }

        // }

        // Reconstruct path from `predecessors`
        // let mut path = Vec::new();
        // let mut current = target;
        // while let Some(&prev) = predecessors.get(&current) {
        //     path.push(current);
        //     current = prev;
        //     if current == start {
        //         path.push(start);
        //         path.reverse();
        //         return path;
        //     }
        // }

        Vec::new() // No path found
    }

    pub fn find_path_with_limit(
        &self,
        start: usize,
        target: usize,
        explored_paths: &mut Vec<Vec<usize>>,
    ) -> Vec<usize> {
        // find_path_with_limit(self, start, target)

        vec![]
    }
}

/// Count how many edges from `start` to `node` by walking `came_from`.
fn distance_to_start(came_from: &HashMap<usize, usize>, mut node: usize) -> usize {
    let mut dist = 0;
    while let Some(&parent) = came_from.get(&node) {
        if parent == node {
            break; // Reached the start
        }
        node = parent;
        dist += 1;
    }
    dist
}
impl<'data> PassiveTree<'data> {
    pub fn fuzzy_search_nodes(&self, query: &str) -> Vec<usize> {
        _fuzzy_search_nodes(self, query)
    }
}

fn _fuzzy_search_nodes(data: &PassiveTree, query: &str) -> Vec<usize> {
    data.nodes
        .iter()
        .filter(|(_, node)| node.name.to_lowercase().contains(&query.to_lowercase()))
        .map(|(id, _)| *id)
        .collect()
}
/// Rebuild the path backward from `target` to `start`, then reverse it.
fn rebuild_path(came_from: &HashMap<usize, usize>, start: usize, target: usize) -> Vec<usize> {
    let mut path = Vec::new();
    let mut current = target;
    while current != start {
        path.push(current);
        current = came_from[&current];
    }
    path.push(start);
    path.reverse();
    path
}
