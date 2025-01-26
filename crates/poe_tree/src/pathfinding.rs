use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

use super::edges::Edge;
use super::stats::Stat;
use super::type_wrappings::NodeId;
use super::PassiveTree;

impl PassiveTree {
    /// There is a limit on the maximum passive points you can aquire in game, lets take advantage of that to do less work.
    const STEP_LIMIT: i32 = 123;
    const MAX_NODE_ID: u32 = u16::MAX as u32;

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
                .map(|node| (node_id, node.as_passive_skill(self).stats.to_vec()))
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

impl PassiveTree {
    pub fn bfs(&self, start: NodeId, target: NodeId) -> Vec<NodeId> {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessors = HashMap::new();

        // Push the start node with depth 0
        queue.push_back((start, 0));
        visited.insert(start);

        while let Some((current, depth)) = queue.pop_front() {
            // Stop traversal if the step limit is reached
            if depth > Self::STEP_LIMIT {
                log::warn!("Step limit reached without finding the target");
                break;
            }

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
                        queue.push_back((neighbor, depth + 1)); // Increment depth
                        predecessors.insert(neighbor, current);
                    }
                });
        }

        log::warn!("No path found from {} to {}", start, target);
        vec![] // No path found
    }
    pub fn bfs_any(&self, start: NodeId, targets: &[NodeId]) -> Vec<NodeId> {
        // Convert targets slice to a HashSet for efficient lookup
        let target_set: HashSet<NodeId> = targets.iter().cloned().collect();

        // Initialize visited set, queue for BFS, and predecessors map for path reconstruction
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessors: HashMap<NodeId, NodeId> = HashMap::new();

        // Start BFS from the `start` node
        queue.push_back((start.clone(), 0));
        visited.insert(start.clone());

        while let Some((current, depth)) = queue.pop_front() {
            // Check if current node is one of the targets
            if target_set.contains(&current) {
                // Reconstruct the path from start to current
                let mut path = Vec::new();
                let mut step = current.clone();
                path.push(step.clone());

                while let Some(prev) = predecessors.get(&step) {
                    step = prev.clone();
                    path.push(step.clone());
                }

                path.reverse(); // Reverse to get path from start to target
                return path;
            }

            if depth >= Self::STEP_LIMIT {
                log::warn!(
                    "Step limit ({}) reached without finding any target from node {:?}",
                    Self::STEP_LIMIT,
                    current
                );
                break;
            }

            // Explore all neighboring nodes
            for neighbor in self.neighbors(&current) {
                if visited.insert(neighbor.clone()) {
                    queue.push_back((neighbor.clone(), depth + 1));
                    predecessors.insert(neighbor, current.clone());
                }
            }
        }

        // If no path is found to any target
        log::warn!(
            "No path found from {:?} to any of the targets: {:?}",
            start,
            targets
        );
        Vec::new()
    }
    pub fn neighbors<'t>(&'t self, node: &'t NodeId) -> impl Iterator<Item = NodeId> + 't {
        self.edges.iter().filter_map(|edge| {
            if edge.start == *node {
                Some(edge.end)
            } else if edge.end == *node {
                Some(edge.start)
            } else {
                None
            }
        })
    }
    pub fn multi_bfs(&self, starts: &[NodeId], targets: &[NodeId]) -> Vec<NodeId> {
        let start_time = std::time::Instant::now();
        log::trace!(
            "Starting Multi-Source BFS from {:?} to targets {:?}",
            starts,
            targets
        );

        if starts.is_empty() {
            log::warn!("No start nodes provided.");
            return Vec::new();
        }

        // Initialize a shared channel to receive paths from threads
        let (path_sender, path_receiver): (Sender<Vec<NodeId>>, Receiver<Vec<NodeId>>) = channel();

        // Shared flag to indicate if a path has been found
        let found = Arc::new(Mutex::new(false));

        // Number of threads to spawn
        let num_threads = starts.len().min(8); // Limit to 8 threads or number of starts

        // Divide the start nodes among the threads
        let starts_per_thread = (starts.len() + num_threads - 1) / num_threads;
        let arc_tree = Arc::new(self.clone());

        // Spawn threads
        let mut handles = Vec::new();
        for i in 0..num_threads {
            let tree_clone = Arc::clone(&arc_tree);
            let path_sender_clone = path_sender.clone();
            let found_clone = Arc::clone(&found);

            let thread_starts = starts
                .iter()
                .skip(i * starts_per_thread)
                .take(starts_per_thread)
                .cloned()
                .collect::<Vec<_>>();

            let thread_targets = targets.to_vec();

            let handle = thread::spawn(move || {
                for start in thread_starts {
                    // Check if another thread has found a path
                    {
                        let found_lock = found_clone.lock().unwrap();
                        if *found_lock {
                            log::trace!(
                                "Thread for start {:?} exiting early as a path has been found.",
                                start
                            );
                            return;
                        }
                    }

                    let path = tree_clone.bfs_any(start, &thread_targets);

                    if !path.is_empty() {
                        // Attempt to set the found flag
                        {
                            let mut found_lock = found_clone.lock().unwrap();
                            if !*found_lock {
                                *found_lock = true;
                                log::trace!(
                                    "Thread for start {:?} found a path: {:?}",
                                    start,
                                    path
                                );
                                // Send the found path
                                path_sender_clone.send(path).unwrap();
                            }
                        }
                        return;
                    }
                }
            });
            handles.push(handle);
        }

        drop(path_sender); // Close the sender to avoid blocking

        // Wait for the first path to be received
        let result = path_receiver.recv().unwrap_or_else(|_| Vec::new());

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        if result.is_empty() {
            log::warn!(
                "No path found from any of the start nodes to the targets after {} ms.",
                start_time.elapsed().as_millis()
            );
        } else {
            log::info!("Path found in {} ms.", start_time.elapsed().as_millis());
        }

        result
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

#[cfg(test)]
mod test {

    use crate::quick_tree;

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
}
