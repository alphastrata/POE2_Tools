//$ crates/poe_tree/src/data/poe_tree/pathfinding.rs
use super::edges::Edge;
use super::stats::Stat;
use super::type_wrappings::NodeId;

use std::cmp::{Ordering, Reverse};
use std::collections::{BinaryHeap, HashMap, HashSet};

use super::PassiveTree;

#[derive(Eq, PartialEq)]
struct NodeCost {
    node_id: NodeId,
    cost: usize,
}

impl Ord for NodeCost {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost) // Reverse order for min-heap
    }
}

impl PartialOrd for NodeCost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Pathfinding algos..
impl PassiveTree {
    /// There is a limit on the maximum passive points you can aquire in game, lets take advantage of that to do less work.
    const STEP_LIMIT: i32 = 123;

    pub fn is_node_within_distance(&self, start: NodeId, target: NodeId, max_steps: usize) -> bool {
        let path = self.find_path(start, target);
        !path.is_empty() && path.len() <= max_steps + 1
    }

    /// naive BFS
    pub fn find_path(&self, start: NodeId, end: NodeId) -> Vec<NodeId> {
        use std::collections::{HashSet, VecDeque};

        let mut depths = HashMap::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut came_from = std::collections::HashMap::new();

        queue.push_back(start);
        visited.insert(start);
        depths.insert(start, 0); // Start node at depth 0

        while let Some(current) = queue.pop_front() {
            let current_depth = *depths.get(&current).unwrap_or(&0);
            if current == end {
                let mut path = vec![end];
                let mut step = end;
                while let Some(&prev) = came_from.get(&step) {
                    path.push(prev);
                    step = prev;
                }
                path.reverse();
                return path;
            }

            // Stop exploring further if max steps are reached
            if current_depth >= Self::STEP_LIMIT {
                continue;
            }

            for edge in &self.edges {
                let neighbor = if edge.start == current {
                    edge.end
                } else if edge.end == current {
                    edge.start
                } else {
                    continue;
                };

                if !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    queue.push_back(neighbor);
                    came_from.insert(neighbor, current);
                }
            }
        }

        vec![]
    }

    pub fn fuzzy_search_nodes(&self, query: &str) -> Vec<usize> {
        log::debug!("Performing search for query: {}", query);
        self.nodes
            .iter()
            .filter(|(_, node)| node.name.to_lowercase().contains(&query.to_lowercase()))
            .map(|(id, _)| *id)
            .collect()
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
            let (from, to) = (edge.start, edge.end);

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

    pub fn find_shortest_path(&self, start: NodeId, target: NodeId) -> Vec<NodeId> {
        let mut distances: HashMap<NodeId, usize> = HashMap::new();
        let mut predecessors: HashMap<NodeId, NodeId> = HashMap::new();
        let mut priority_queue = BinaryHeap::new();

        // Initialize distances
        for &node_id in self.nodes.keys() {
            distances.insert(node_id, usize::MAX);
        }
        distances.insert(start, 0);

        // Push the starting node with a cost of 0
        priority_queue.push(Reverse(NodeCost {
            node_id: start,
            cost: 0,
        }));

        while let Some(Reverse(NodeCost { node_id, cost })) = priority_queue.pop() {
            // Stop processing if we've reached the target
            if node_id == target {
                break;
            }

            // Skip outdated entries in the priority queue
            if cost > *distances.get(&node_id).unwrap_or(&usize::MAX) {
                continue;
            }

            // Process all neighbors of the current node
            for edge in self.edges.iter() {
                let neighbor = if edge.start == node_id {
                    edge.end
                } else if edge.end == node_id {
                    edge.start
                } else {
                    continue;
                };

                let new_cost = cost + 1; // Assuming unweighted edges
                if new_cost < *distances.get(&neighbor).unwrap_or(&usize::MAX) {
                    eprintln!(
                        "Updating distance for node {:?} from {} to {}",
                        neighbor,
                        distances.get(&neighbor).unwrap_or(&usize::MAX),
                        new_cost
                    );
                    distances.insert(neighbor, new_cost);
                    predecessors.insert(neighbor, node_id);
                    priority_queue.push(Reverse(NodeCost {
                        node_id: neighbor,
                        cost: new_cost,
                    }));
                }
            }
        }

        // Reconstruct the path from `predecessors`
        let mut path = Vec::new();
        let mut current = target;
        while let Some(&prev) = predecessors.get(&current) {
            path.push(current);
            current = prev;
        }

        // Add the start node if we reached it
        if current == start {
            path.push(start);
            path.reverse();
            eprintln!("Path found: {:?}", path);
            return path;
        }

        // No valid path found
        eprintln!("No path found from {:?} to {:?}", start, target);
        Vec::new()
    }

    pub fn all_nodes_with_distance(&self, start: NodeId, delta: usize) -> Vec<Vec<NodeId>> {
        let mut distances: HashMap<NodeId, usize> = HashMap::new();
        let mut priority_queue = BinaryHeap::new();
        let mut result: Vec<Vec<NodeId>> = vec![Vec::new(); delta + 1];

        // Initialize distances
        for &node_id in self.nodes.keys() {
            distances.insert(node_id, usize::MAX);
        }
        distances.insert(start, 0);

        // Start with the source node
        priority_queue.push(NodeCost {
            node_id: start,
            cost: 0,
        });

        while let Some(NodeCost { node_id, cost }) = priority_queue.pop() {
            if cost > delta {
                continue;
            }

            if cost > *distances.get(&node_id).unwrap_or(&usize::MAX) {
                continue;
            }

            // Add node to the corresponding distance group
            result[cost].push(node_id);

            for edge in self.edges.iter() {
                let neighbor = if edge.start == node_id {
                    edge.end
                } else if edge.end == node_id {
                    edge.start
                } else {
                    continue;
                };

                let new_cost = cost + 1; // Unweighted edges
                if new_cost < *distances.get(&neighbor).unwrap_or(&usize::MAX) {
                    distances.insert(neighbor, new_cost);
                    priority_queue.push(NodeCost {
                        node_id: neighbor,
                        cost: new_cost,
                    });
                }
            }
        }

        // Filter out empty groups
        result
            .into_iter()
            .filter(|group| !group.is_empty())
            .collect()
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

#[cfg(test)]
mod test {
    use super::*;
    use std::{fs::File, io::BufReader};
    // #[test]
    // fn frontiers() {
    // flow of water = 49220 -> criticals11 = 4157 -> criticals3 = 34168
    // 55088 is the +10% to crits
    // criticals 3 = 34168 +%36 to crits within 8s (moving out from 49220)
    // criticals14 = 20024 ->
    // attack_damage1 = 7576 +10%
    // attack_damage3 = 33866 +10% -> connects to Flow of Water 49220
    // attack_speed27 = 42857 -> 7576 ->  33866
    // flow of water -> criticals15 = 44223
    // flow of water -> intelligence9 = 8975
    // attack_speed46 = 14725 -> 49220
    // attack_speed37 = 34233
    // }

    #[test]
    fn nodes_within_distance() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();

        let tree = PassiveTree::from_value(&u).unwrap();

        // Starting node for the test
        let start_node = 49220;
        let max_distance = 4;

        // Get all nodes within the specified distance
        let nodes_by_distance = tree.all_nodes_with_distance(start_node, max_distance);

        // Ensure the result is not empty
        assert!(
            !nodes_by_distance.is_empty(),
            "No nodes found within distance {} from node {}.",
            max_distance,
            start_node
        );

        // Print nodes grouped by distance with their details
        nodes_by_distance
            .iter()
            .enumerate()
            .for_each(|(distance, nodes)| {
                if distance > 3 {
                    println!("Distance {}: {:?}", distance, nodes);
                    nodes.iter().enumerate().for_each(|(e, &node_id)| {
                        if let Some(node) = tree.nodes.get(&node_id) {
                            {
                                println!(
                                    "{}  Node ID: {},\n{}Pos: {}, \n{}Name: {}, \n{}Stats: {:?}",
                                    " ".repeat(e),
                                    node_id,
                                    " ".repeat(e),
                                    node.position,
                                    " ".repeat(e),
                                    node.name,
                                    " ".repeat(e),
                                    node.stats
                                );
                            }
                        }
                    });
                    println!()
                }
            });

        // Example assertions for further validation
        // Ensure nodes exist at each distance
        for distance in 0..=max_distance {
            assert!(
                nodes_by_distance.get(distance).is_some(),
                "No nodes found at distance {} from node {}.",
                distance,
                start_node
            );
        }
    }
    #[test]
    fn equivalent_path_lengths_to_target() {
        let file = File::open("data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();

        let tree = PassiveTree::from_value(&u).unwrap();

        // Define the two expected paths
        let path1 = [10364, 42736, 56045, 58329]; // Path via Attack Damage nodes
        let path2 = [10364, 42736, 13419, 42076]; // Path via Critical Damage nodes

        // Find the shortest path to the target for both paths
        let actual_path1 = tree.find_shortest_path(path1[0], path1[3]);
        let actual_path2 = tree.find_shortest_path(path2[0], path1[3]);

        println!("Path 1 (via Attack Damage): {:?}", actual_path1);
        println!("Path 2 (via Critical Damage): {:?}", actual_path2);
    }

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
