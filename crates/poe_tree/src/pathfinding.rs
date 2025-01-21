//$ crates/poe_tree/src/pathfinding.rs
use super::edges::Edge;
use super::stats::Stat;
use super::type_wrappings::NodeId;

use std::collections::HashSet;

use super::PassiveTree;

// Pathfinding algos..
impl PassiveTree {
    /// There is a limit on the maximum passive points you can aquire in game, lets take advantage of that to do less work.
    const STEP_LIMIT: i32 = 123;

    pub fn is_node_within_distance(&self, start: NodeId, target: NodeId, max_steps: usize) -> bool {
        let path = self.find_path(start, target);
        !path.is_empty() && path.len() <= max_steps + 1
    }

    pub fn fuzzy_search_nodes(&self, query: &str) -> Vec<u32> {
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
                .map(|node| (node_id, node.as_passive_skill(&self).stats.to_vec()))
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

    pub fn find_shortest_path(&self, a: NodeId, b: NodeId) -> Vec<u32> {
        self.bfs(a, b)
    }
    pub fn find_path(&self, a: NodeId, b: NodeId) -> Vec<u32> {
        self.bfs(a, b)
    }
}
fn _fuzzy_search_nodes(data: &PassiveTree, query: &str) -> Vec<u32> {
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
impl PassiveTree {
    pub fn bfs(&self, start: NodeId, target: NodeId) -> Vec<NodeId> {
        use std::collections::{HashMap, HashSet, VecDeque};

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessors = HashMap::new();

        queue.push_back(start);
        visited.insert(start);

        while let Some(current) = queue.pop_front() {
            if current == target {
                // Reconstruct the path
                let mut path = vec![target];
                let mut step = target;
                while let Some(&prev) = predecessors.get(&step) {
                    path.push(prev);
                    step = prev;
                }
                path.reverse();
                return path;
            }

            // Explore neighbors via edges
            self.edges
                .iter()
                .filter_map(|edge| {
                    if edge.start == current {
                        Some(edge.end)
                    } else if edge.end == current {
                        Some(edge.start)
                    } else {
                        None
                    }
                })
                .for_each(|neighbor| {
                    if visited.insert(neighbor) {
                        queue.push_back(neighbor);
                        predecessors.insert(neighbor, current);
                    }
                });
        }

        log::warn!("No path found from {} to {}", start, target);
        vec![] // No path found
    }
}

pub fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("../../data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let mut tree = PassiveTree::from_value(&tree_data).unwrap();

    tree.remove_hidden();
    tree
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn path_between_flow_like_water_and_chaos_inoculation() {
        let tree: PassiveTree = quick_tree();

        // Use fuzzy search to find nodes
        let flow_ids = tree.fuzzy_search_nodes("flow like water");
        let chaos_ids = tree.fuzzy_search_nodes("chaos inoculation");

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
        }
        // Update this value based on expected path length after refactoring
        assert_eq!(path.len(), 15, "Path length mismatch");
        println!("{:#?}", path);
    }

    #[test]
    fn test_path_avatar_of_fire_to_over_exposure() {
        let tree = quick_tree();

        // Use fuzzy search to find nodes
        let avatar_ids = tree.fuzzy_search_nodes("Avatar of Fire");
        let over_exposure_ids = tree.fuzzy_search_nodes("Overexposure");

        assert!(!avatar_ids.is_empty(), "No node found for 'Avatar of Fire'");
        assert!(
            !over_exposure_ids.is_empty(),
            "No node found for 'Overexposure'"
        );

        let start_id = avatar_ids[0];
        let target_id = over_exposure_ids[0];

        // Find paths using BFS
        let bfs_path = tree.bfs(start_id, target_id);

        // Assertions
        assert!(!bfs_path.is_empty(), "No path found using BFS!");

        println!("Path from Avatar of Fire to Overexposure:");
        println!("BFS Path: {:?}", bfs_path);
        assert_eq!(bfs_path.len(), 27, "Expected path length does not match.");
    }

    #[test]
    fn bfs_pathfinding() {
        let tree = quick_tree();

        let start = 10364;
        let target = 58329;
        let expected_path = vec![10364, 42736, 56045, 58329];

        let actual_path = tree.bfs(start, target);
        assert_eq!(actual_path, expected_path, "Paths do not match!");
    }

    // #[test]
    // fn equivalent_path_lengths_to_target() {
    //     let tree = quick_tree();

    //     // Define the two expected paths
    //     let path1 = [10364, 42736, 56045, 58329]; // Path via Attack Damage nodes
    //     let path2 = [10364, 42736, 13419, 42076]; // Path via Critical Damage nodes

    //     // Find the shortest path to the target for both paths
    //     let actual_path1 = tree.find_shortest_path(path1[0], path1[3]);
    //     let actual_path2 = tree.find_shortest_path(path2[0], path1[3]);

    //     println!("Path 1 (via Attack Damage): {:?}", actual_path1);
    //     println!("Path 2 (via Critical Damage): {:?}", actual_path2);
    // }
}
