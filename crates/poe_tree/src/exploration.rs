use super::{type_wrappings::NodeId, PassiveTree};

use ahash::AHashMap;
use bit_set::BitSet;
use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::{unbounded, Receiver, Sender}; // for cloneable receivers
use smallvec::SmallVec;
use std::{collections::VecDeque, sync::Arc, thread, time::Duration};

impl PassiveTree {
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

    //BASELINE
    // pub fn walk_n_steps(&self, start: NodeId, steps: usize) -> Vec<Vec<NodeId>> {
    //     let t1 = std::time::Instant::now();
    //     let mut paths = Vec::new();
    //     let mut queue = VecDeque::new();

    //     // Initialize queue with the starting node in its own path
    //     queue.push_back(vec![start]);

    //     while let Some(path) = queue.pop_front() {
    //         let last_node = *path.last().unwrap();

    //         if path.len() - 1 == steps {
    //             paths.push(path.clone()); // Store paths of exactly `n` steps
    //             continue;
    //         }

    //         for edge in &self.edges {
    //             let (next_node, other_node) = (edge.start, edge.end);

    //             if next_node == last_node && !path.contains(&other_node) {
    //                 let mut new_path = path.clone();
    //                 new_path.push(other_node);
    //                 queue.push_back(new_path);
    //             } else if other_node == last_node && !path.contains(&next_node) {
    //                 let mut new_path = path.clone();
    //                 new_path.push(next_node);
    //                 queue.push_back(new_path);
    //             }
    //         }
    //     }

    //     log::debug!(
    //         "Walking {} neighbours took {}ms",
    //         steps,
    //         t1.elapsed().as_millis()
    //     );

    //     paths
    // }

    //NOTE: just bitset
    // About 1-3% better
    // pub fn walk_n_steps(&self, start: NodeId, steps: usize) -> Vec<Vec<NodeId>> {
    //     use bit_set::BitSet;

    //     let t1 = std::time::Instant::now();
    //     let mut paths = Vec::new();
    //     let mut queue = VecDeque::new();

    //     queue.push_back((start, 0, BitSet::new())); // (current_node, path_length, visited_nodes)

    //     while let Some((last_node, length, mut visited)) = queue.pop_front() {
    //         if length == steps {
    //             paths.push(vec![last_node]); // Store only endpoints to reduce memory usage
    //             continue;
    //         }

    //         for next_node in self.neighbors(&last_node) {
    //             if visited.insert(next_node as usize) {
    //                 // Avoid cycles
    //                 queue.push_back((next_node, length + 1, visited.clone()));
    //             }
    //         }
    //     }

    //     log::debug!(
    //         "Walking {} steps took {}ms",
    //         steps,
    //         t1.elapsed().as_millis()
    //     );

    //     paths
    // }

    // NOTE: smallvec
    // pub fn walk_n_steps<const N: usize>(&self, start: NodeId, steps: usize) -> Vec<Vec<NodeId>> {
    //     let t1 = std::time::Instant::now();
    //     let mut paths = Vec::new();
    //     let mut queue = VecDeque::new();

    //     // Store full paths, initialized with `start`
    //     queue.push_back(SmallVec::<[NodeId; N]>::from_elem(start, 1));

    //     while let Some(path) = queue.pop_front() {
    //         let last_node = *path.last().unwrap();

    //         if path.len() - 1 == steps {
    //             paths.push(path.to_vec()); // Store paths of exactly `n` steps
    //             continue;
    //         }

    //         for neighbor in self.neighbors(last_node) {
    //             if !path.contains(&neighbor) {
    //                 // Ensure no cycles in the current path
    //                 let mut new_path = path.clone();
    //                 new_path.push(neighbor);
    //                 queue.push_back(new_path);
    //             }
    //         }
    //     }

    //     log::debug!(
    //         "Walking {} steps took {}ms",
    //         steps,
    //         t1.elapsed().as_millis()
    //     );

    //     paths
    // }

    // smallvec and bitset
    pub fn walk_n_steps<const N: usize>(&self, start: NodeId, steps: usize) -> Vec<Vec<NodeId>> {
        let t1 = std::time::Instant::now();
        let mut paths = Vec::new();
        let mut queue = VecDeque::new();

        let visited = BitSet::with_capacity(self.nodes.len()); // O(1) lookups

        queue.push_back((
            SmallVec::<[NodeId; N]>::from_elem(start, 1),
            visited.clone(),
        ));

        while let Some((path, mut visited)) = queue.pop_front() {
            let last_node = *path.last().unwrap();

            if path.len() - 1 == steps {
                paths.push(path.to_vec());
                continue;
            }

            self.neighbors(last_node).for_each(|neighbor| {
                if visited.insert(neighbor as usize) {
                    // O(1) cycle check
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back((new_path, visited.clone())); // Clone BitSet, but it's efficient
                }
            });
        }

        log::debug!(
            "Walking {} steps took {}ms",
            steps,
            t1.elapsed().as_millis()
        );

        paths
    }
}

struct CSRGraph {
    offsets: Vec<usize>,
    neighbors: Vec<NodeId>,
    node_map: AHashMap<NodeId, usize>,
}

impl CSRGraph {
    fn from_tree(tree: &PassiveTree) -> Self {
        let mut node_map = AHashMap::new();
        let mut offsets = Vec::with_capacity(tree.nodes.len() + 1);
        let mut neighbors = Vec::new();

        let mut index = 0;
        for &node in tree.nodes.keys() {
            node_map.insert(node, index);
            offsets.push(neighbors.len());
            let mut adj: Vec<NodeId> = tree.neighbors(node).into_iter().collect();
            adj.sort();
            neighbors.extend(adj);
            index += 1;
        }
        offsets.push(neighbors.len());

        CSRGraph {
            offsets,
            neighbors,
            node_map,
        }
    }

    #[inline]
    fn get_neighbors(&self, node: NodeId) -> &[NodeId] {
        if let Some(&idx) = self.node_map.get(&node) {
            &self.neighbors[self.offsets[idx]..self.offsets[idx + 1]]
        } else {
            &[]
        }
    }
}
impl PassiveTree {
    pub fn walk_n_steps_csr<const N: usize>(
        &self,
        start: NodeId,
        steps: usize,
    ) -> Vec<Vec<NodeId>> {
        let t1 = std::time::Instant::now();
        let csr_graph = CSRGraph::from_tree(self);

        let mut paths = Vec::new();
        let mut queue = VecDeque::new();

        queue.push_back(SmallVec::<[NodeId; N]>::from_elem(start, 1));

        while let Some(path) = queue.pop_front() {
            let last_node = *path.last().unwrap();

            if path.len() - 1 == steps {
                paths.push(path.to_vec());
                continue;
            }

            for &neighbor in csr_graph.get_neighbors(last_node) {
                // Uses CSR adjacency list
                if !path.contains(&neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back(new_path);
                }
            }
        }

        log::debug!(
            "Walking {} steps took {}ms",
            steps,
            t1.elapsed().as_millis()
        );

        paths
    }
}

#[cfg(test)]
mod test {
    use crate::{consts::get_level_one_nodes, edges::Edge, quick_tree, type_wrappings::NodeId};
    use ahash::AHashMap;
    use rayon::prelude::*;

    #[test]
    fn walk_15_steps() {
        let expected_lengths: AHashMap<NodeId, usize> = AHashMap::from([
            (4739, 4922),
            (44871, 5289), // Sorcerer/Witch
            (10364, 3693),
            (52980, 3721), // Monk
            (56651, 3250),
            (13828, 3393), // Ranger
            (38646, 2404),
            (3936, 1791), // Warrior
            (59915, 4055),
            (59779, 4028), // Mercenary
            (50084, 2579),
            (13855, 2924), // Unknown
        ]);

        const STEPS: usize = 15;

        let mut tree = quick_tree();
        tree.remove_hidden();
        let nodes: Vec<(&'static str, &[NodeId; 2])> = get_level_one_nodes()
            .iter()
            .map(|(name, ids)| (*name, ids))
            .collect();

        nodes.par_iter().for_each(|(character, node_ids)| {
            node_ids.iter().for_each(|&start_node| {
                let paths = tree.walk_n_steps::<STEPS>(start_node, STEPS);
                let expected = *expected_lengths.get(&start_node).unwrap_or(&0);
                assert_eq!(
                    paths.len(),
                    expected,
                    "Incorrect path length for {} (Start node: {})",
                    character,
                    start_node
                );

                paths.iter().for_each(|path| {
                    path.windows(2).for_each(|pair| {
                        let (from, to) = (pair[0], pair[1]);
                        let edge = Edge {
                            start: from,
                            end: to,
                        };
                        let rev_edge = Edge {
                            start: to,
                            end: from,
                        };
                        assert!(
                            tree.edges.contains(&edge) || tree.edges.contains(&rev_edge),
                            "Invalid edge in path: {:?}",
                            path
                        );
                    });
                });
            });
        });
    }
    #[test]
    fn walk_15_steps_csr() {
        let expected_lengths: AHashMap<NodeId, usize> = AHashMap::from([
            (4739, 4922),
            (44871, 5289), // Sorcerer/Witch
            (10364, 3693),
            (52980, 3721), // Monk
            (56651, 3250),
            (13828, 3393), // Ranger
            (38646, 2404),
            (3936, 1791), // Warrior
            (59915, 4055),
            (59779, 4028), // Mercenary
            (50084, 2579),
            (13855, 2924), // Unknown
        ]);

        const STEPS: usize = 15;

        let mut tree = quick_tree();
        tree.remove_hidden();
        let nodes: Vec<(&'static str, &[NodeId; 2])> = get_level_one_nodes()
            .iter()
            .map(|(name, ids)| (*name, ids))
            .collect();

        nodes.par_iter().for_each(|(character, node_ids)| {
            node_ids.iter().for_each(|&start_node| {
                let paths = tree.walk_n_steps_csr::<STEPS>(start_node, STEPS);
                let expected = *expected_lengths.get(&start_node).unwrap_or(&0);
                assert_eq!(
                    paths.len(),
                    expected,
                    "Incorrect path length for {} (Start node: {})",
                    character,
                    start_node
                );

                paths.iter().for_each(|path| {
                    path.windows(2).for_each(|pair| {
                        let (from, to) = (pair[0], pair[1]);
                        let edge = Edge {
                            start: from,
                            end: to,
                        };
                        let rev_edge = Edge {
                            start: to,
                            end: from,
                        };
                        assert!(
                            tree.edges.contains(&edge) || tree.edges.contains(&rev_edge),
                            "Invalid edge in path: {:?}",
                            path
                        );
                    });
                });
            });
        });
    }
}
