// src/data.rs
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::{
    collections::{HashMap, VecDeque},
    fs,
};

pub const ORBIT_RADII: [f64; 8] = [0.0, 82.0, 162.0, 335.0, 493.0, 662.0, 812.0, 972.0];
pub const ORBIT_SLOTS: [usize; 8] = [1, 6, 16, 16, 40, 60, 60, 60];

#[derive(Debug, Clone)]
pub struct Group {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Default)]
pub struct PassiveSkill {
    pub name: Option<String>,
    pub is_notable: bool,
    pub stats: Vec<(String, f64)>, // no more HashMap
}

#[derive(Debug, Clone, Default)]
pub struct Node {
    pub node_id: usize,
    pub skill_id: Option<String>,
    pub parent: usize,
    pub radius: usize,
    pub position: usize,
    pub connections: Vec<usize>,
    // Derived data
    pub name: String,
    pub is_notable: bool,
    pub stats: Vec<(String, f64)>,
    pub wx: f64,
    pub wy: f64,
    pub active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct PassiveTree {
    pub groups: HashMap<usize, Group>,
    pub nodes: HashMap<usize, Node>,
    pub passive_skills: HashMap<String, PassiveSkill>,
}

impl PassiveTree {
    pub fn compute_positions_and_stats(&mut self) {
        for (_, node) in self.nodes.iter_mut() {
            // 1) group pos
            if let Some(group) = self.groups.get(&node.parent) {
                // radius
                let r = ORBIT_RADII.get(node.radius).copied().unwrap_or(0.0);
                // how many slots on this orbit
                let slots = ORBIT_SLOTS.get(node.radius).copied().unwrap_or(1) as f64;
                // convert node.position into an angle
                let angle = node.position as f64 * (2.0 * std::f64::consts::PI / slots);

                node.wx = group.x + r * angle.cos();
                node.wy = group.y + r * angle.sin();
            }

            // 2) fill name, is_notable, stats from skill table
            if let Some(skill_id) = &node.skill_id {
                if let Some(skill) = self.passive_skills.get(skill_id) {
                    node.name = skill.name.clone().unwrap_or_default();
                    node.is_notable = skill.is_notable;
                    node.stats = skill.stats.clone();
                }
            }
        }
    }
    pub fn load_tree(path: &str) -> Self {
        let data = fs::read_to_string(path).expect("Failed to read JSON");
        let json: Value = serde_json::from_str(&data).expect("Invalid JSON");

        // parse groups
        let mut groups = HashMap::new();
        if let Some(obj) = json["passive_tree"]["groups"].as_object() {
            for (gid, gval) in obj {
                let gx = gval["x"].as_f64().unwrap_or(0.0);
                let gy = gval["y"].as_f64().unwrap_or(0.0);
                groups.insert(
                    gid.parse::<usize>().unwrap_or_default(),
                    Group { x: gx, y: gy },
                );
            }
        }

        // parse nodes
        let mut nodes = HashMap::new();
        if let Some(obj) = json["passive_tree"]["nodes"].as_object() {
            for (node_id, nval) in obj {
                let skill_id = nval["skill_id"].as_str().map(|s| s.to_string());
                let parent = nval["parent"].as_u64().unwrap_or(0) as usize;
                let radius = nval["radius"].as_u64().unwrap_or(0) as usize;
                let position = nval["position"].as_u64().unwrap_or(0) as usize;

                let mut connections = Vec::new();
                if let Some(conn_arr) = nval["connections"].as_array() {
                    for c in conn_arr {
                        if let Some(cid) = c["id"].as_u64() {
                            connections.push(cid as usize);
                        }
                    }
                }
                let node_id = node_id
                    .parse::<usize>()
                    .expect("It is impossible to have a node without a NodeID");

                nodes.insert(
                    node_id,
                    Node {
                        node_id,
                        skill_id,
                        parent,
                        radius,
                        position,
                        connections,
                        name: String::new(),
                        is_notable: false,
                        stats: Vec::new(),
                        wx: 0.0,
                        wy: 0.0,
                        active: false,
                    },
                );
            }
        }

        // make connections bidirectional
        {
            // we create a separate scope so the borrow checker sees we don't need
            // the mutable borrow outside
            let ids: Vec<usize> = nodes.keys().copied().collect();
            for id in ids {
                // safe because we have a separate reference inside the loop
                let node_connections = nodes[&id].connections.clone();
                for &other_id in &node_connections {
                    if let Some(other_node) = nodes.get_mut(&other_id) {
                        if !other_node.connections.contains(&id) {
                            other_node.connections.push(id);
                        }
                    }
                }
            }
        }

        // parse passive_skills
        let mut passive_skills = HashMap::new();
        if let Some(skills_obj) = json["passive_skills"].as_object() {
            for (skill_id, sval) in skills_obj {
                let name = sval["name"].as_str().map(|s| s.to_string());
                let is_notable = sval["is_notable"].as_bool().unwrap_or(false);

                let mut stats_vec = Vec::new();
                if let Some(st) = sval["stats"].as_object() {
                    for (k, v) in st {
                        if let Some(num) = v.as_f64() {
                            stats_vec.push((k.clone(), num));
                        }
                    }
                }

                passive_skills.insert(
                    skill_id.clone(),
                    PassiveSkill {
                        name,
                        is_notable,
                        stats: stats_vec,
                    },
                );
            }
        }

        let mut output = Self {
            groups,
            nodes,
            passive_skills,
        };
        output.compute_positions_and_stats();

        output
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct NodeCost {
    node_id: usize,
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

impl PassiveTree {
    pub fn find_shortest_path(&self, start: usize, target: usize) -> Vec<usize> {
        let mut distances: HashMap<usize, usize> = HashMap::new();
        let mut predecessors: HashMap<usize, usize> = HashMap::new();
        let mut priority_queue = BinaryHeap::new();

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
            // Stop if we reached the target
            if node_id == target {
                break;
            }

            // Skip if this path is not optimal
            if cost > *distances.get(&node_id).unwrap_or(&usize::MAX) {
                continue;
            }

            // Explore neighbors
            if let Some(node) = self.nodes.get(&node_id) {
                for &neighbor in &node.connections {
                    let new_cost = cost + 1; // Assume unweighted edges (cost = 1)
                    if new_cost < *distances.get(&neighbor).unwrap_or(&usize::MAX) {
                        distances.insert(neighbor, new_cost);
                        predecessors.insert(neighbor, node_id);
                        priority_queue.push(NodeCost {
                            node_id: neighbor,
                            cost: new_cost,
                        });
                    }
                }
            }
        }

        // Reconstruct path from `predecessors`
        let mut path = Vec::new();
        let mut current = target;
        while let Some(&prev) = predecessors.get(&current) {
            path.push(current);
            current = prev;
            if current == start {
                path.push(start);
                path.reverse();
                return path;
            }
        }

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

/// Returns the path from `start` to `target`, or an empty Vec if none found.
/// Uses BFS with a `step_limit`.
fn find_path_with_limit(
    tree: &PassiveTree,
    start: usize,
    target: usize,
    step_limit: usize,
) -> Vec<usize> {
    // Edge case: trivial
    if start == target {
        return vec![start];
    }

    // Record how we reached each visited node (for reconstructing the path):
    let mut came_from: HashMap<usize, usize> = HashMap::new();
    came_from.insert(start, start);

    // Standard BFS queue
    let mut queue = VecDeque::new();
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        // If we’ve gone too deep, stop exploring this branch
        let dist_so_far = distance_to_start(&came_from, current);
        if dist_so_far >= step_limit {
            continue;
        }

        // Expand neighbors
        if let Some(node) = tree.nodes.get(&current) {
            for &neighbor in &node.connections {
                // If we haven’t visited `neighbor` yet
                if !came_from.contains_key(&neighbor) {
                    // Record that we reached `neighbor` from `current`
                    came_from.insert(neighbor, current);

                    // If we found the target, reconstruct path and return
                    if neighbor == target {
                        return rebuild_path(&came_from, start, target);
                    }
                    queue.push_back(neighbor);
                }
            }
        }
    }
    // If we exhaust the queue with no result, no path was found (within step_limit).
    Vec::new()
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

impl PassiveTree {
    pub fn fuzzy_search_nodes(&self, query: &str) -> Vec<usize> {
        fuzzy_search_nodes(self, query)
    }
}

fn fuzzy_search_nodes(data: &PassiveTree, query: &str) -> Vec<usize> {
    data.nodes
        .iter()
        .filter(|(_, node)| node.name.to_lowercase().contains(&query.to_lowercase()))
        .map(|(id, _)| *id)
        .collect()
}

#[cfg(debug_assertions)]
impl PassiveTree {
    /// Debugging BFS to print all discovered paths from `start` up to `max_depth`.
    /// This won't find *all* paths in a large cyclical graph, but it shows
    /// the first time we visit each new node, up to `max_depth`.
    pub fn debug_print_paths(&self, start: usize, max_depth: usize) {
        use std::collections::{HashSet, VecDeque};

        // We'll track which nodes we've visited so we don't cycle forever.
        let mut visited = HashSet::new();
        visited.insert(start);

        // Each item is a path (list of node IDs)
        let mut queue = VecDeque::new();
        queue.push_back(vec![start]);

        while let Some(path) = queue.pop_front() {
            // Print the path with (id: name)
            let path_str = path
                .iter()
                .map(|&nid| {
                    let node = &self.nodes[&nid];
                    format!("({}: {})", nid, node.name)
                })
                .collect::<Vec<_>>()
                .join(" -> ");
            println!("Exploring path: {}", path_str);

            // If we haven’t hit our depth limit, expand
            if path.len() < max_depth {
                let current_node = *path.last().unwrap();
                if let Some(node) = self.nodes.get(&current_node) {
                    for &conn in &node.connections {
                        if !visited.contains(&conn) {
                            visited.insert(conn);
                            let mut next_path = path.clone();
                            next_path.push(conn);
                            queue.push_back(next_path);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_print_paths() {
        let mut tree = PassiveTree::load_tree("data/POE2_TREE.json");

        tree.debug_print_paths(49220, 20);
    }

    #[test]
    fn test_path_between_flow_like_water_and_chaos_inoculation() {
        let mut tree = PassiveTree::load_tree("data/POE2_TREE.json");
        tree.compute_positions_and_stats();

        // Use fuzzy search to find nodes
        let flow_ids = fuzzy_search_nodes(&tree, "flow like water");
        let chaos_ids = fuzzy_search_nodes(&tree, "chaos inoculation");

        assert!(!flow_ids.is_empty(), "No node found for 'flow like water'");
        assert!(
            !chaos_ids.is_empty(),
            "No node found for 'chaos inoculation'"
        );

        let start_id = flow_ids[0];
        let target_id = chaos_ids[0];

        // Find shortest path using Dijkstra's Algorithm
        let path = tree.find_shortest_path(start_id, target_id);
        if path.is_empty() {
            println!("No path found between {} and {}", start_id, target_id);
        } else {
            println!("Path found: {:?}", path);
            for node_id in path.iter() {
                let n = tree.nodes.get(&node_id).unwrap();
                if !n.name.contains("Attribute") {
                    print!("(ID:{} NAME: {}) ->", node_id, n.name);
                } else {
                    print!("[ID:{}] ->", node_id);
                }
            }
        }
        assert_eq!(path.len(), 15);
    }

    #[test]
    fn test_bidirectional_connections() {
        let tree: PassiveTree = PassiveTree::load_tree("data/POE2_TREE.json");

        for (&node_id, node) in &tree.nodes {
            for &connected_id in &node.connections {
                let other_node = &tree.nodes[&connected_id];
                assert!(
                    other_node.connections.contains(&node_id),
                    "Connection missing from {} to {}",
                    connected_id,
                    node_id
                );
            }
        }

        println!("All connections are bidirectional.");
    }

    #[test]
    fn test_path_between_avatar_of_fire_and_over_exposure() {
        let tree = PassiveTree::load_tree("data/POE2_TREE.json");

        // Use fuzzy search to find nodes
        let avatar_ids = fuzzy_search_nodes(&tree, "Avatar of Fire");
        let over_exposure_ids = fuzzy_search_nodes(&tree, "Over Exposure");

        assert!(!avatar_ids.is_empty(), "No node found for 'Avatar of Fire'");
        assert!(
            !over_exposure_ids.is_empty(),
            "No node found for 'Over Exposure'"
        );

        let start_id = avatar_ids[0];
        let target_id = over_exposure_ids[0];

        // Find shortest path using Dijkstra's Algorithm
        let path = tree.find_shortest_path(start_id, target_id);

        if path.is_empty() {
            panic!(
                "No path found between {} and {}",
                tree.nodes[&start_id].name, tree.nodes[&target_id].name
            );
        } else {
            println!("Path found: {:?}", path);
            for node_id in path.iter() {
                let n = tree.nodes.get(&node_id).unwrap();
                println!("(ID:{} NAME: {})", node_id, n.name);
            }
        }
        assert_eq!(path.len(), 95, "The path length should be 95 steps.");
    }
}
