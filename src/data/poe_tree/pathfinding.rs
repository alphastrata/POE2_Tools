//$ src/data/poe_tree/pathfinding.rs
use super::consts::*;
use super::coordinates::Group;
use super::edges::Edge;
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
impl PassiveTree {
    pub fn fuzzy_search_nodes(&self, query: &str) -> Vec<usize> {
        _fuzzy_search_nodes(self, query)
    }
    pub fn create_paths(&self, nodes: Vec<&str>) -> Result<Vec<NodeId>, String> {
        let mut path = Vec::new();
        let mut last_node_id: Option<NodeId> = None;

        for name_or_id in nodes {
            let node_id = self.find_node_by_name_or_id(name_or_id)?;
            if let Some(last_id) = last_node_id {
                if !self.are_nodes_connected(last_id, node_id) {
                    return Err(format!("No connection between {} and {}", last_id, node_id));
                }
            }
            path.push(node_id);
            last_node_id = Some(node_id);
        }

        Ok(path)
    }
    pub fn are_nodes_connected(&self, node_a: NodeId, node_b: NodeId) -> bool {
        !self.find_shortest_path(node_a, node_b).is_empty()
    }
    pub fn find_node_by_name_or_id(&self, identifier: &str) -> Result<NodeId, String> {
        // Try finding by NodeId first
        if let Ok(node_id) = identifier.parse::<NodeId>() {
            if self.nodes.contains_key(&node_id) {
                return Ok(node_id);
            }
        }

        // Fuzzy match by name
        let matches: Vec<_> = self
            .nodes
            .iter()
            .filter(|(_, node)| node.name.contains(identifier))
            .map(|(id, _)| *id)
            .collect();

        match matches.len() {
            1 => Ok(matches[0]),
            0 => Err(format!("No node found matching '{}'", identifier)),
            _ => Err(format!(
                "Ambiguous identifier '{}', multiple nodes match",
                identifier
            )),
        }
    }
    pub fn frontier_nodes_lazy<'a>(
        &'a self,
        path: &'a [NodeId],
    ) -> impl Iterator<Item = NodeId> + 'a {
        let active_set: HashSet<NodeId> = path.iter().cloned().collect();

        self.edges.iter().filter_map(move |edge| {
            // Determine the neighboring node
            let (from, to) = (edge.from, edge.to);

            if active_set.contains(&from) && !active_set.contains(&to) && !self.nodes[&to].active {
                Some(to)
            } else if active_set.contains(&to)
                && !active_set.contains(&from)
                && !self.nodes[&from].active
            {
                Some(from)
            } else {
                None
            }
        })
    }
    pub fn frontier_node_details_lazy<'a>(
        &'a self,
        frontier_nodes: impl Iterator<Item = NodeId> + 'a,
    ) -> impl Iterator<Item = (NodeId, Vec<Stat>)> + 'a {
        frontier_nodes.filter_map(move |node_id| {
            self.nodes
                .get(&node_id)
                .map(|node| (node_id, node.stats.to_vec()))
        })
    }
    pub fn create_paths_lazy<'a>(
        &'a self,
        nodes: Vec<&'a str>,
    ) -> impl Iterator<Item = Result<NodeId, String>> + 'a {
        let mut last_node_id: Option<NodeId> = None;

        nodes.into_iter().map(move |name_or_id| {
            let node_id = self.find_node_by_name_or_id(name_or_id)?;
            if let Some(last_id) = last_node_id {
                // Check connection via PassiveTree.edges
                let edge = Edge::new(last_id, node_id, self);
                if !self.edges.contains(&edge) {
                    return Err(format!("No connection between {} and {}", last_id, node_id));
                }
            }
            last_node_id = Some(node_id);
            Ok(node_id)
        })
    }
    pub fn find_path_with_limit(
        &self,
        start: NodeId,
        target: NodeId,
        explored_paths: &mut Vec<Vec<NodeId>>,
    ) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut stack = vec![(start, vec![start])];

        while let Some((current, path)) = stack.pop() {
            if current == target {
                explored_paths.push(path.clone());
                return path;
            }

            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            for edge in self.edges.iter() {
                let neighbor = if edge.from == current {
                    edge.to
                } else if edge.to == current {
                    edge.from
                } else {
                    continue;
                };

                if !visited.contains(&neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    stack.push((neighbor, new_path));
                }
            }
        }

        Vec::new() // No path found
    }
}

impl PassiveTree {
    pub fn find_shortest_path(&self, start: NodeId, target: NodeId) -> Vec<NodeId> {
        let mut distances: HashMap<NodeId, usize> = HashMap::new();
        let mut predecessors: HashMap<NodeId, NodeId> = HashMap::new();
        let mut priority_queue = BinaryHeap::new();

        // Initialize distances
        eprintln!("Initializing distances...");
        for &node_id in self.nodes.keys() {
            distances.insert(node_id, usize::MAX);
        }
        distances.insert(start, 0);

        // Start with the source node
        eprintln!("Starting from node: {:?}", start);
        priority_queue.push(NodeCost {
            node_id: start,
            cost: 0,
        });

        while let Some(NodeCost { node_id, cost }) = priority_queue.pop() {
            eprintln!("Processing node: {:?} with cost: {}", node_id, cost);
            if node_id == target {
                eprintln!("Reached target node: {:?}", target);
                break;
            }

            if cost > *distances.get(&node_id).unwrap_or(&usize::MAX) {
                eprintln!(
                    "Skipping node: {:?} as current cost ({}) is greater than known cost ({})",
                    node_id,
                    cost,
                    distances.get(&node_id).unwrap_or(&usize::MAX)
                );
                continue;
            }

            for edge in self.edges.iter() {
                let neighbor = if edge.from == node_id {
                    edge.to
                } else if edge.to == node_id {
                    edge.from
                } else {
                    continue;
                };

                let new_cost = cost + 1; // Unweighted edges
                eprintln!(
                    "Inspecting edge from {:?} to {:?} with new_cost: {}",
                    node_id, neighbor, new_cost
                );

                if new_cost < *distances.get(&neighbor).unwrap_or(&usize::MAX) {
                    eprintln!(
                        "Updating distance for node {:?} from {} to {}",
                        neighbor,
                        distances.get(&neighbor).unwrap_or(&usize::MAX),
                        new_cost
                    );
                    distances.insert(neighbor, new_cost);
                    predecessors.insert(neighbor, node_id);
                    priority_queue.push(NodeCost {
                        node_id: neighbor,
                        cost: new_cost,
                    });
                }
            }
        }

        // Reconstruct path from `predecessors`
        let mut path = Vec::new();
        let mut current = target;
        eprintln!("Reconstructing path...");
        while let Some(&prev) = predecessors.get(&current) {
            eprintln!("Node {:?} has predecessor {:?}", current, prev);
            path.push(current);
            current = prev;
            if current == start {
                path.push(start);
                path.reverse();
                eprintln!("Path found: {:?}", path);
                return path;
            }
        }

        eprintln!("No path found from {:?} to {:?}", start, target);
        Vec::new() // No path found
    }
}

fn _fuzzy_search_nodes(data: &PassiveTree, query: &str) -> Vec<usize> {
    let mut prev_node = 0;
    data.nodes
        .iter()
        .map(|(nid, node)| {
            println!(
                "Inspecting {nid}\t{:?} named:{} FROM {prev_node} ",
                node.skill_id, node.name
            );
            prev_node = *nid;
            (nid, node)
        })
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

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::File, io::BufReader};

    #[test]
    fn connected_path() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();

        let tree = PassiveTree::from_value(&u).unwrap();

        // Node IDs for the test
        let node_a = 63064; // 54413, 32683, 40068, 48198
        let node_b = 48198;

        // Check if the nodes are connected
        let are_connected = tree.are_nodes_connected(node_a, node_b);
        assert!(
            are_connected,
            "Nodes {} and {} should be connected, but they are not, .are_node_connected returned {}",
            node_a, node_b, are_connected
        );

        // Find the shortest path between the nodes
        let path = tree.find_shortest_path(node_a, node_b);
        assert!(
            !path.is_empty(),
            "No path found between {} and {}.",
            node_a,
            node_b
        );

        // Verify the path contains the correct nodes
        assert!(
            path.contains(&node_a) && path.contains(&node_b),
            "Path does not include both nodes {} and {}: {:?}",
            node_a,
            node_b,
            path
        );
    }
}
