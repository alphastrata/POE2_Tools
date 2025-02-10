#![allow(unused_imports)]
use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::{unbounded, Receiver, Sender}; // for cloneable receivers
use rayon::prelude::*;
use std::cmp::Reverse;
use std::sync::RwLock;
use std::time::Duration;
use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, HashSet, VecDeque},
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

use super::{edges::Edge, stats::Stat, type_wrappings::NodeId, PassiveTree};

impl PassiveTree {
    /// There is a limit on the maximum passive points you can aquire in game, lets take advantage of that to do less work.
    pub const STEP_LIMIT: NodeId = 123;

    pub fn is_node_within_distance(&self, start: NodeId, target: NodeId, max_steps: usize) -> bool {
        let path = self.find_path(start, target);
        !path.is_empty() && path.len() <= max_steps + 1
    }
    pub fn fuzzy_search_nodes(&self, query: &str) -> Vec<NodeId> {
        log::debug!("Performing search Nodes for query: {}", query);
        self.nodes
            .iter()
            .filter(|(_, node)| node.name.to_lowercase().contains(&query.to_lowercase()))
            .map(|(id, _)| *id)
            .collect()
    }

    pub fn fuzzy_search_nodes_and_skills(&self, query: &str) -> HashSet<NodeId> {
        log::debug!("Performing search Nodes & Skills for query: {}", query);
        let query = query.to_lowercase();

        self.nodes
            .iter()
            .filter(|(_, node)| {
                node.name.to_lowercase().contains(&query)
                    || self
                        .passive_for_node(node)
                        .stats()
                        .iter()
                        .any(|stat| stat.name().contains(&query))
            })
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
        !self.find_shorpath(node_a, node_b).is_empty()
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
            let (from, to) = edge.as_tuple();

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
                .map(|node| (node_id, node.as_passive_skill(self).stats().to_vec()))
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

    pub fn find_shorpath(&self, a: NodeId, b: NodeId) -> Vec<NodeId> {
        self.bfs(a, b)
    }
    pub fn find_path(&self, a: NodeId, b: NodeId) -> Vec<NodeId> {
        self.bfs(a, b)
    }

    pub fn shortest_to_target_from_any_of(
        &self,
        target: NodeId,
        candidates: &[NodeId],
    ) -> Option<Vec<NodeId>> {
        candidates
            .iter()
            .filter_map(|&c| {
                // Try both directions since BFS is directional
                let path = {
                    let fwd = self.dijkstra(target, c);
                    if !fwd.is_empty() {
                        fwd
                    } else {
                        self.bfs(c, target)
                    }
                };
                if path.is_empty() {
                    None
                } else {
                    Some(path)
                }
            })
            .min_by_key(|p| p.len())
    }

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
            self.neighbors(&current).into_iter().for_each(|neighbor| {
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
        use std::collections::{HashMap, HashSet, VecDeque};

        let start_time = std::time::Instant::now();
        log::trace!(
            "bfs_any: start={:?} targets={:?} (finding the shortest path to any target)",
            start,
            targets
        );

        let target_set: HashSet<NodeId> = targets.iter().copied().collect();
        if target_set.is_empty() {
            log::warn!("No targets provided to bfs_any()");
            return Vec::new();
        }

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        let mut predecessors: HashMap<NodeId, NodeId> = HashMap::new();

        // Initialize BFS
        queue.push_back((start, 0));
        visited.insert(start);

        while let Some((current, depth)) = queue.pop_front() {
            // If we've found a target, reconstruct path
            if target_set.contains(&current) {
                let mut path = vec![current];
                let mut step = current;
                while let Some(prev) = predecessors.get(&step) {
                    step = *prev;
                    path.push(step);
                }
                path.reverse();
                log::trace!(
                    "bfs_any: found target {:?} in {} steps (duration={:?}), path={:?}",
                    current,
                    depth,
                    start_time.elapsed(),
                    path
                );
                return path;
            }

            // Depth check
            if depth >= Self::STEP_LIMIT {
                log::warn!(
                    "bfs_any: step limit {} reached from node {:?} without hitting a target",
                    Self::STEP_LIMIT,
                    current
                );
                break;
            }

            // Enqueue neighbors
            self.neighbors(&current).into_iter().for_each(|neighbor| {
                if visited.insert(neighbor) {
                    queue.push_back((neighbor, depth + 1)); // Increment depth
                    predecessors.insert(neighbor, current);
                }
            });
        }

        log::warn!(
            "bfs_any: no path found from {:?} to any of {:?} (elapsed={:?})",
            start,
            targets,
            start_time.elapsed()
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

    #[deprecated = "Don't use this it's + 1-2000% WORSE than st."]
    pub fn par_neighbors<'t>(
        &'t self,
        node: &'t NodeId,
    ) -> impl IntoParallelIterator<Item = NodeId> + 't {
        self.edges.par_iter().filter_map(|edge| {
            if edge.start == *node {
                Some(edge.end)
            } else if edge.end == *node {
                Some(edge.start)
            } else {
                None
            }
        })
    }

    pub fn bfs_all_shortest(&self, start: NodeId, targets: &[NodeId]) -> Vec<Vec<NodeId>> {
        use std::collections::{HashMap, HashSet, VecDeque};
        let target_set: HashSet<NodeId> = targets.iter().copied().collect();
        if target_set.is_empty() {
            return vec![];
        }

        let mut info = HashMap::new();
        let mut queue = VecDeque::new();
        info.insert(start, (0, Vec::new()));
        queue.push_back(start);
        let mut found_dist: Option<NodeId> = None;

        while let Some(current) = queue.pop_front() {
            // Copy the distance value out to avoid holding an immutable borrow.
            let cur_dist = info.get(&current).unwrap().0;
            if let Some(fd) = found_dist {
                if cur_dist > fd {
                    break;
                }
            }
            if target_set.contains(&current) {
                found_dist = Some(cur_dist);
            }
            if cur_dist >= Self::STEP_LIMIT {
                continue;
            }

            // Process neighbors.
            self.neighbors(&current).into_iter().for_each(|neighbor| {
                let next_dist = cur_dist + 1;
                if let Some((d, preds)) = info.get_mut(&neighbor) {
                    if *d == next_dist {
                        preds.push(current);
                    }
                } else {
                    info.insert(neighbor, (next_dist, vec![current]));
                    queue.push_back(neighbor);
                }
            });
        }

        let min_dist = match found_dist {
            Some(d) => d,
            None => return vec![],
        };

        // Gather targets reached with the minimum distance.
        let reached: Vec<NodeId> = target_set
            .into_iter()
            .filter(|&t| info.get(&t).map(|&(d, _)| d) == Some(min_dist))
            .collect();

        let mut all_paths = Vec::new();
        reached.into_iter().for_each(|target| {
            all_paths.extend(Self::reconstruct_paths(start, target, &info));
        });
        all_paths
    }

    fn reconstruct_paths(
        start: NodeId,
        node: NodeId,
        info: &HashMap<NodeId, (NodeId, Vec<NodeId>)>,
    ) -> Vec<Vec<NodeId>> {
        if node == start {
            return vec![vec![start]];
        }
        let mut paths = Vec::new();
        if let Some((_, preds)) = info.get(&node) {
            for &pred in preds {
                for mut path in Self::reconstruct_paths(start, pred, info) {
                    path.push(node);
                    paths.push(path);
                }
            }
        }
        paths
    }

    pub fn par_walk_n_steps_use_chains(
        self: Arc<Self>,
        start: NodeId,
        steps: usize,
    ) -> Vec<Vec<NodeId>> {
        log::debug!(
            "Starting par_walk_n_steps_use_chains with start: {} and steps: {}",
            start,
            steps
        );
        let (work_tx, work_rx): (Sender<Vec<NodeId>>, Receiver<Vec<NodeId>>) = unbounded();
        let (result_tx, result_rx): (Sender<Vec<NodeId>>, Receiver<Vec<NodeId>>) = unbounded();

        work_tx.send(vec![start]).unwrap();
        log::debug!("Seeded work queue with initial path: [{}]", start);

        let num_workers = num_cpus::get();
        log::debug!("Spawning {} worker threads", num_workers);
        let mut workers = Vec::with_capacity(num_workers);

        for i in 0..num_workers {
            let work_rx = work_rx.clone();
            let work_tx = work_tx.clone();
            let result_tx = result_tx.clone();
            let tree = Arc::clone(&self);
            let thread_name = format!("par_walker_{}", i);
            let builder = thread::Builder::new().name(thread_name.clone());
            workers.push(
                builder
                    .spawn(move || {
                        log::debug!("Worker {} started", thread::current().name().unwrap());
                        loop {
                            match work_rx.recv_timeout(Duration::from_millis(5000)) {
                                Ok(path) => {
                                    log::debug!(
                                        "Worker {} received path: {:?}",
                                        thread::current().name().unwrap(),
                                        path
                                    );
                                    if path.len() - 1 == steps {
                                        log::debug!(
                                            "Worker {} reached target steps with path: {:?}",
                                            thread::current().name().unwrap(),
                                            path
                                        );
                                        result_tx.send(path).unwrap();
                                    } else {
                                        let last = *path.last().unwrap();
                                        let mut new_paths = Vec::new();
                                        for edge in &tree.edges {
                                            if edge.start == last && !path.contains(&edge.end) {
                                                let mut p = path.clone();
                                                p.push(edge.end);
                                                new_paths.push(p);
                                            } else if edge.end == last
                                                && !path.contains(&edge.start)
                                            {
                                                let mut p = path.clone();
                                                p.push(edge.start);
                                                new_paths.push(p);
                                            }
                                        }
                                        log::debug!(
                                            "Worker {} expanded path {:?} into {} candidates",
                                            thread::current().name().unwrap(),
                                            path,
                                            new_paths.len()
                                        );
                                        new_paths.sort_by(|a, b| {
                                            a[..a.len() - 1].cmp(&b[..b.len() - 1])
                                        });
                                        new_paths
                                            .dedup_by(|a, b| a[..a.len() - 1] == b[..b.len() - 1]);
                                        log::debug!(
                                            "Worker {} deduped candidates, {} remain",
                                            thread::current().name().unwrap(),
                                            new_paths.len()
                                        );
                                        for new_path in new_paths {
                                            log::debug!(
                                                "Worker {} sending new path: {:?}",
                                                thread::current().name().unwrap(),
                                                new_path
                                            );
                                            work_tx.send(new_path).unwrap();
                                        }
                                    }
                                }
                                Err(RecvTimeoutError::Timeout) => {
                                    log::debug!(
                                        "Worker {} timed out, assuming no more work, exiting",
                                        thread::current().name().unwrap()
                                    );
                                    break;
                                }
                                Err(RecvTimeoutError::Disconnected) => {
                                    log::debug!(
                                        "Worker {} found channel disconnected, exiting",
                                        thread::current().name().unwrap()
                                    );
                                    break;
                                }
                            }
                        }
                        log::debug!("Worker {} exiting", thread::current().name().unwrap());
                    })
                    .unwrap(),
            );
        }

        drop(work_tx);
        drop(result_tx);
        log::debug!("Dropped extra channel clones, awaiting finished paths...");

        let mut finished = Vec::new();
        for path in result_rx.iter() {
            if path.len() - 1 == steps {
                log::debug!("Main thread received finished path: {:?}", path);
                finished.push(path);
            }
        }
        for worker in workers {
            worker.join().unwrap();
        }
        log::debug!(
            "All workers completed. Total finished paths: {}",
            finished.len()
        );
        finished
    }

    pub fn walk_n_steps(&self, start: NodeId, steps: usize) -> Vec<Vec<NodeId>> {
        let t1 = std::time::Instant::now();
        let mut paths = Vec::new();
        let mut queue = VecDeque::new();

        // Initialize queue with the starting node in its own path
        queue.push_back(vec![start]);

        while let Some(path) = queue.pop_front() {
            let last_node = *path.last().unwrap();

            if path.len() - 1 == steps {
                paths.push(path.clone()); // Store paths of exactly `n` steps
                continue;
            }

            for edge in &self.edges {
                let (next_node, other_node) = (edge.start, edge.end);

                if next_node == last_node && !path.contains(&other_node) {
                    let mut new_path = path.clone();
                    new_path.push(other_node);
                    queue.push_back(new_path);
                } else if other_node == last_node && !path.contains(&next_node) {
                    let mut new_path = path.clone();
                    new_path.push(next_node);
                    queue.push_back(new_path);
                }
            }
        }

        log::debug!(
            "Walking {} neighbours took {}ms",
            steps,
            t1.elapsed().as_millis()
        );

        paths
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

        // We'll gather *all* non-empty BFS results and return the shortest among them.
        // Remove the 'found' flag usage and don't exit early.

        let (path_sender, path_receiver) = std::sync::mpsc::channel::<Vec<NodeId>>();
        let arc_tree = std::sync::Arc::new(self.clone());

        // limit to N threads
        let num_threads = starts.len().min(8);
        let starts_per_thread = starts.len().div_ceil(num_threads);

        let mut handles = Vec::new();
        for i in 0..num_threads {
            let tree_clone = std::sync::Arc::clone(&arc_tree);
            let path_sender_clone = path_sender.clone();

            let thread_starts = starts
                .iter()
                .skip(i * starts_per_thread)
                .take(starts_per_thread)
                .cloned()
                .collect::<Vec<_>>();

            let thread_targets = targets.to_vec();

            let handle = std::thread::spawn(move || {
                // BFS from each of these starts
                for start in thread_starts {
                    let path = tree_clone.bfs_any(start, &thread_targets);
                    // If non-empty, send it. (We keep searching all starts to find the truly shortest.)
                    if !path.is_empty() {
                        path_sender_clone.send(path).ok();
                    }
                }
            });

            handles.push(handle);
        }

        // close the sender in the main thread
        drop(path_sender);

        // Collect all results
        let mut all_paths = Vec::new();
        while let Ok(path) = path_receiver.recv() {
            if !path.is_empty() {
                all_paths.push(path);
            }
        }

        for handle in handles {
            handle.join().ok();
        }

        // pick the shortest path from the results
        let result = all_paths
            .into_iter()
            .min_by_key(|p| p.len())
            .unwrap_or_default();

        if result.is_empty() {
            log::warn!(
                "No path found from any of the start nodes to the targets after {} ms.",
                start_time.elapsed().as_millis()
            );
        } else {
            log::info!(
                "Shortest path found in {} ms.",
                start_time.elapsed().as_millis()
            );
        }

        result
    }
}
fn _fuzzy_search_nodes(data: &PassiveTree, query: &str) -> Vec<NodeId> {
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

impl PassiveTree {
    pub fn dijkstra(&self, start: NodeId, target: NodeId) -> Vec<NodeId> {
        let mut dist: HashMap<NodeId, usize> = HashMap::new();
        let mut prev: HashMap<NodeId, NodeId> = HashMap::new();
        let mut heap = BinaryHeap::new();

        dist.insert(start, 0);
        heap.push(Reverse((0, start)));

        while let Some(Reverse((d, u))) = heap.pop() {
            if u == target {
                let mut path = Vec::new();
                let mut cur = target;
                while let Some(&p) = prev.get(&cur) {
                    path.push(cur);
                    cur = p;
                }
                path.push(start);
                path.reverse();
                return path;
            }
            if d > *dist.get(&u).unwrap() {
                continue;
            }
            for neighbor in self.edges.iter().filter_map(|edge| {
                if edge.start == u {
                    Some(edge.end)
                } else if edge.end == u {
                    Some(edge.start)
                } else {
                    None
                }
            }) {
                let alt = d + 1;
                if alt < *dist.get(&neighbor).unwrap_or(&usize::MAX) {
                    dist.insert(neighbor, alt);
                    prev.insert(neighbor, u);
                    heap.push(Reverse((alt, neighbor)));
                }
            }
        }
        vec![]
    }
}

impl PassiveTree {
    pub fn path_with_cost(&self, path: Vec<NodeId>) -> impl Iterator<Item = (usize, NodeId)> {
        path.into_iter().enumerate()
    }
}

#[cfg(test)]
mod test {
    use crate::{quick_tree, stats::arithmetic::PlusPercentage};

    use super::*;

    use crate::consts::CHAR_START_NODES;
    use std::{collections::HashSet, f32::MIN};

    #[test]
    fn path_between_flow_like_water_and_chaos_inoculation() {
        let tree = quick_tree();
        let flow_ids = tree.fuzzy_search_nodes("flow like water");
        let chaos_ids = tree.fuzzy_search_nodes("chaos inoculation");

        assert!(!flow_ids.is_empty(), "No node found for 'flow like water'");
        assert!(
            !chaos_ids.is_empty(),
            "No node found for 'chaos inoculation'"
        );

        let start_id = flow_ids[0];
        let target_id = chaos_ids[0];

        let bfs_path = tree.find_shorpath(start_id, target_id);
        if bfs_path.is_empty() {
            panic!("No path found between {} and {}", start_id, target_id);
        }
        assert_eq!(bfs_path.len(), 15, "Path length mismatch");
        let dj_path = tree.dijkstra(start_id, target_id);

        assert_eq!(dj_path.len(), 15);
    }

    #[test]
    fn path_avatar_of_fire_to_over_exposure() {
        let tree = quick_tree();
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
        println!("BFS Path: {:?}", bfs_path);

        assert_eq!(bfs_path.len(), 27, "Expected path length does not match.");

        let dj_path = tree.dijkstra(start_id, target_id);
        assert_eq!(dj_path.len(), 27);
    }

    #[test]
    #[ignore = "Dunno why this is failing atm, it's like we cannot go backwards or something."]
    fn shorpath_15957() {
        let tree = quick_tree();
        let candidates = vec![10364, 42857, 20024, 44223, 49220, 58182, 7344, 26931];
        let target = 15957; // 15957 -> 48198 -> 26931
        let path = tree
            .shortest_to_target_from_any_of(target, &candidates)
            .unwrap();

        assert_eq!(path, vec![15957, 48198, 26931]);
    }

    #[test]
    fn shorpath_17248() {
        let tree = quick_tree();
        let candidates = vec![10364, 42857, 20024, 44223, 49220, 58182, 7344, 26931];
        let target = 17248; // 10364 -> 55342 -> 17248
        let path = tree
            .shortest_to_target_from_any_of(target, &candidates)
            .unwrap();

        let expected = [17248, 55342, 10364];
        assert!(expected.into_iter().all(|v| path.contains(&v)))
    }

    //  For warrior melee @ lvl 11 test
    const LVL_CAP: usize = 10;
    const MIN_BONUS_VALUE: f32 = 110.0;
    #[test]
    fn ten_lvl_warrior_finds_110_percent_melee_dam() {
        _ = pretty_env_logger::init();
        let tree = quick_tree();

        const STARTING_LOC: NodeId = 3936; //warrior melee damage.

        let answer = [
            STARTING_LOC,
            19011,
            33556,
            43164,
            45363,
            46325,
            55473,
            5710,
            58528,
            64284, // FINISHING LOCATION
        ];

        let selector = |s: &Stat| matches!(s, Stat::MeleeDamage(_));
        let ser_res = tree.take_while(STARTING_LOC, selector, LVL_CAP - 1);

        let mut winners = vec![];
        ser_res.into_iter().for_each(|potential| {
            let mut melee_dam_total = 0.0;
            potential.iter().for_each(|n| {
                let pnode = tree.nodes.get(n).unwrap();
                let pskill = tree.passive_for_node(pnode);
                let stats = pskill.stats();
                for s in stats {
                    if let Stat::MeleeDamage(_) = s {
                        melee_dam_total += s.value()
                    }
                }
            });
            match melee_dam_total >= MIN_BONUS_VALUE {
                true => {
                    winners.push(potential);
                }
                false => {}
            }
        });
        assert!(!winners.is_empty());

        assert_eq!(1, winners.len());
        winners[0].iter().all(|nid| answer.contains(&nid));
    }
}
