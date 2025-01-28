use rayon::prelude::*;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Instant,
};

use super::{edges::Edge, stats::Stat, type_wrappings::NodeId, PassiveTree};

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
        queue.push_back((start, 0));
        visited.insert(start);

        while let Some((current, depth)) = queue.pop_front() {
            // Check if current node is one of the targets
            if target_set.contains(&current) {
                // Reconstruct the path from start to current
                let mut path = Vec::new();
                let mut step = current;
                path.push(step);

                while let Some(prev) = predecessors.get(&step) {
                    step = *prev;
                    path.push(step);
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
                if visited.insert(neighbor) {
                    queue.push_back((neighbor, depth + 1));
                    predecessors.insert(neighbor, current);
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
        let starts_per_thread = starts.len().div_ceil(num_threads);
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
            log::debug!(
                "Inspecting {nid}\t{:?} named:{} FROM {prev_node} ",
                node.skill_id,
                node.name
            );
            prev_node = *nid;
            (nid, node)
        })
        .filter(|(_, node)| node.name.to_lowercase().contains(&query.to_lowercase()))
        .map(|(id, _)| *id)
        .collect()
}

#[derive(Debug, Eq, PartialEq)]
struct NodeDistance {
    node: u32,
    distance: u16,
}

impl Ord for NodeDistance {
    fn cmp(&self, other: &Self) -> Ordering {
        other.distance.cmp(&self.distance).reverse() // Reverse for min-heap
    }
}

impl PartialOrd for NodeDistance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PassiveTree {
    /// Dijkstra's algorithm returning all paths traversed with verbose logs
    pub fn dijkstra_with_all_paths(
        &self,
        starts: &[u32],
        levels: &[u32],
    ) -> HashMap<(u32, u32), Vec<u32>> {
        let mut paths = HashMap::new();

        for &start in starts {
            let start_time = Instant::now();
            log::debug!("Starting Dijkstra for start node: {}", start);

            let mut distances = HashMap::new();
            let mut predecessors = HashMap::new();
            let mut visited = HashSet::new();
            let mut heap = BinaryHeap::new();

            // Start node has a distance of 0
            distances.insert(start, 0);
            heap.push(NodeDistance {
                node: start,
                distance: 0,
            });

            while let Some(NodeDistance { node, distance }) = heap.pop() {
                if visited.contains(&node) {
                    continue;
                }
                visited.insert(node);

                for neighbour in self.neighbors(&node) {
                    if visited.contains(&neighbour) {
                        continue;
                    }

                    let edge_weight = 1; // Use an integer edge weight
                    let new_distance = distance + edge_weight;

                    if new_distance < *distances.get(&neighbour).unwrap_or(&u16::MAX) {
                        distances.insert(neighbour, new_distance);
                        predecessors.insert(neighbour, node);
                        heap.push(NodeDistance {
                            node: neighbour,
                            distance: new_distance,
                        });
                    }
                }
            }

            // Reconstruct paths for each level
            for &level in levels {
                let mut path = Vec::new();
                let mut current = level;

                while let Some(&prev) = predecessors.get(&current) {
                    path.push(current);
                    current = prev;
                }

                path.push(start); // Add the start node
                path.reverse();

                // Save path to the map
                paths.insert((start, level), path);
            }

            log::debug!(
                "Dijkstra for start node {} completed in {:?} ms",
                start,
                start_time.elapsed().as_millis()
            );
        }

        paths
    }

    /// Parallelised Dijkstra's algorithm with all paths and logging
    pub fn parallel_dijkstra_with_all_paths(
        &self,
        starts: &[u32],
        levels: &[u32],
    ) -> HashMap<(u32, u32), Vec<u32>> {
        let paths = Arc::new(Mutex::new(HashMap::new()));

        starts.par_iter().for_each(|&start| {
            let start_time = Instant::now();
            log::debug!("Starting Dijkstra for start node: {}", start);

            let mut distances = HashMap::new();
            let mut predecessors = HashMap::new();
            let mut visited = HashSet::new();
            let mut heap = BinaryHeap::new();

            // Start node has a distance of 0
            distances.insert(start, 0);
            heap.push(NodeDistance {
                node: start,
                distance: 0,
            });

            while let Some(NodeDistance { node, distance }) = heap.pop() {
                if visited.contains(&node) {
                    continue;
                }
                visited.insert(node);

                for neighbour in self.neighbors(&node) {
                    if visited.contains(&neighbour) {
                        continue;
                    }

                    let edge_weight = 1;
                    let new_distance = distance + edge_weight;

                    if new_distance < *distances.get(&neighbour).unwrap_or(&u16::MAX) {
                        distances.insert(neighbour, new_distance);
                        predecessors.insert(neighbour, node);
                        heap.push(NodeDistance {
                            node: neighbour,
                            distance: new_distance,
                        });
                    }
                }
            }

            // Reconstruct paths for each level
            let mut local_paths = HashMap::new();
            for &level in levels {
                let mut path = Vec::new();
                let mut current = level;

                while let Some(&prev) = predecessors.get(&current) {
                    path.push(current);
                    current = prev;
                }

                path.push(start); // Add the start node
                path.reverse();

                // Save path to the local map
                local_paths.insert((start, level), path);
            }

            // Merge local paths into the global map
            let mut global_paths = paths.lock().unwrap();
            global_paths.extend(local_paths);

            log::debug!(
                "Dijkstra for start node {} completed in {:?} ms",
                start,
                start_time.elapsed().as_millis()
            );
        });

        Arc::try_unwrap(paths).unwrap().into_inner().unwrap()
    }
}

#[cfg(test)]
mod test {

    use crate::quick_tree;

    use super::*;

    const LONG_TEST_PATHS: ([u32; 10], [u32; 10], [u32; 7]) = (
        [
            10364, 42857, 20024, 44223, 49220, 36778, 36479, 12925, 61196, 58329,
        ],
        [
            10364, 42857, 20024, 44223, 49220, 14725, 34233, 32545, 61196, 58329,
        ],
        [10364, 55342, 17248, 53960, 8975, 61196, 58329],
    );

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
            log::debug!("No path found between {} and {}", start_id, target_id);
        }
        // Update this value based on expected path length after refactoring
        assert_eq!(path.len(), 15, "Path length mismatch");
        log::debug!("{:#?}", path);
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

        let bfs_path = tree.bfs(start_id, target_id);

        assert!(!bfs_path.is_empty(), "No path found using BFS!");

        println!("Path from Avatar of Fire to Overexposure:");
        println!(
            "BFS Path: for {:?} to {:?} = {:?}",
            avatar_ids, over_exposure_ids, bfs_path
        );
        assert_eq!(bfs_path.len(), 27, "Expected path length does not match.");
    }

    fn validate_shortest_path(
        actual_path: &[u32],
        expected_paths: &([u32; 10], [u32; 10], [u32; 7]),
        description: &str,
    ) {
        assert!(
            !actual_path.is_empty(),
            "{}: Expected a non-empty path, but got none.",
            description
        );

        let is_valid = actual_path == &expected_paths.0[..]
            || actual_path == &expected_paths.1[..]
            || actual_path == &expected_paths.2[..];

        assert!(
                is_valid,
                "{}: Path does not match any of the expected paths.\nActual: {:?}\nExpected: {:?} or {:?}",
                description,
                actual_path,
                expected_paths.0,
                expected_paths.1,
            );
    }

    #[test]
    fn bfs_pathfinding() {
        let tree = quick_tree();

        let start = 10364;
        let target = 58329;

        let actual_path = tree.bfs(start, target);
        validate_shortest_path(&actual_path, &LONG_TEST_PATHS, "BFS Pathfinding");
    }

    #[test]
    fn dijkstra_pathfinding() {
        let tree = quick_tree();

        let start = 10364;
        let target = 58329;

        // Fix: Stop Dijkstra early when the target is reached
        let paths = tree.dijkstra_with_all_paths(&[start], &[target]);
        let actual_path = paths
            .get(&(start, target))
            .expect("Dijkstra path not found for the given start and target.");

        validate_shortest_path(actual_path, &LONG_TEST_PATHS, "Dijkstra Pathfinding");
    }

    #[test]
    fn parallel_dijkstra_pathfinding() {
        let tree = quick_tree();

        let start = 10364;
        let target = 58329;

        let paths = tree.parallel_dijkstra_with_all_paths(&[start], &[target]);
        let actual_path = paths
            .get(&(start, target))
            .expect("Parallel Dijkstra path not found for the given start and target.");

        validate_shortest_path(
            actual_path,
            &LONG_TEST_PATHS,
            "Parallel Dijkstra Pathfinding",
        );
    }

    #[test]
    fn dijkstra_vs_parallel_dijkstra_consistency() {
        let tree = quick_tree();

        let start = 10364;
        let target = 58329;

        let dijkstra_paths = tree.dijkstra_with_all_paths(&[start], &[target]);
        let dijkstra_path = dijkstra_paths
            .get(&(start, target))
            .expect("Dijkstra path not found for the given start and target.");

        let parallel_dijkstra_paths = tree.parallel_dijkstra_with_all_paths(&[start], &[target]);
        let parallel_dijkstra_path = parallel_dijkstra_paths
            .get(&(start, target))
            .expect("Parallel Dijkstra path not found for the given start and target.");

        validate_shortest_path(dijkstra_path, &LONG_TEST_PATHS, "Dijkstra vs Expected Path");
        validate_shortest_path(
            parallel_dijkstra_path,
            &LONG_TEST_PATHS,
            "Parallel Dijkstra vs Expected Path",
        );

        // Ensure consistency between Dijkstra and Parallel Dijkstra
        assert_eq!(
            dijkstra_path, parallel_dijkstra_path,
            "Dijkstra and Parallel Dijkstra paths do not match!"
        );
    }
}
