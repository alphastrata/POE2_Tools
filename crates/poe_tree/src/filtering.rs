use rayon::prelude::*;
use std::collections::{HashSet, VecDeque};

use crate::{stats::Stat, type_wrappings::NodeId, PassiveTree};

impl PassiveTree {
    /*
    NOTE:
        It's interesting, because the &nid is 4 times bigger than a NodeId, and
        hashing isn't free, I'd of thought we'd do better on using a Vec<NodeId>
        here rather than a HashSet, however in the benchmarks it is CONSISTENTLY
        within the noise threshold, so... maybe it's all for nothing.
    */

    ///```rust, ignore
    /// fn main() {
    ///     let tree = PassiveTree::from_data(...); // create tree from data
    ///     let start_node: NodeId = 1;
    ///     let paths = tree.take_while(start_node, |s| matches!(s, Stat::LightningDamage(_)), 50);
    ///     println!("{:?}", paths);
    /// }
    /// ```
    pub fn take_while<F>(&self, start: NodeId, predicate: F, depth: usize) -> Vec<Vec<NodeId>>
    where
        F: Fn(&Stat) -> bool,
    {
        let targets: HashSet<_> = self
            .nodes
            .iter()
            .filter_map(|(&nid, node)| {
                if node
                    .as_passive_skill(self)
                    .stats()
                    .iter()
                    .any(|s| predicate(s))
                {
                    Some(nid)
                } else {
                    None
                }
            })
            .collect();
        log::trace!("Num targets: {}", targets.len());

        let mut valid_paths = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back(vec![start]);

        while let Some(path) = queue.pop_front() {
            let last = *path.last().unwrap();

            // Ensure paths are exactly `depth` long
            if path.len() == depth + 1 {
                if path.iter().any(|nid| targets.contains(nid)) {
                    log::trace!("Adding valid path: {:?}", path);
                    valid_paths.push(path.clone());
                }
                continue; // Stop expanding this path further
            }

            // Extend path only if we haven't reached the depth limit
            self.edges.iter().for_each(|edge| {
                let (a, b) = (edge.start, edge.end);
                if a == last && !path.contains(&b) {
                    log::trace!("Expanding path {:?} → {}", path, b);
                    let mut new_path = path.clone();
                    new_path.push(b);
                    queue.push_back(new_path);
                } else if b == last && !path.contains(&a) {
                    log::trace!("Expanding path {:?} → {}", path, a);
                    let mut new_path = path.clone();
                    new_path.push(a);
                    queue.push_back(new_path);
                }
            });
        }

        log::debug!("Total valid paths found: {}", valid_paths.len());
        valid_paths
    }

    pub fn par_take_while<F>(&self, start: NodeId, predicate: F, depth: usize) -> Vec<Vec<NodeId>>
    where
        F: Fn(&Stat) -> bool + Sync,
    {
        let targets: HashSet<NodeId> =
            self.nodes
                .iter()
                .fold(HashSet::new(), |mut acc, (&nid, node)| {
                    if node
                        .as_passive_skill(self)
                        .stats()
                        .iter()
                        .any(|s| predicate(s))
                    {
                        // acc.push(nid);
                        acc.insert(nid);
                    }
                    acc
                });

        let valid_paths = self.par_explore(vec![start], depth, &targets);

        // Prune paths: remove any path that's a prefix of a longer one.
        (0..valid_paths.len())
            .filter(|&i| {
                !valid_paths.iter().enumerate().any(|(j, q)| {
                    j != i && q.len() > valid_paths[i].len() && q.starts_with(&valid_paths[i])
                })
            })
            .map(|i| valid_paths[i].clone())
            .collect()
    }

    fn par_explore(
        &self,
        path: Vec<NodeId>,
        depth: usize,
        // targets: &[NodeId]
        targets: &HashSet<NodeId>,
    ) -> Vec<Vec<NodeId>> {
        let mut valid = Vec::new();
        if path.iter().any(|nid| targets.contains(nid)) {
            valid.push(path.clone());
        }
        if path.len() - 1 == depth {
            return valid;
        }
        let last = *path.last().unwrap();
        let neighbours: Vec<_> = self
            .edges
            .iter()
            .filter_map(|edge| {
                if edge.start == last && !path.contains(&edge.end) {
                    Some(edge.end)
                } else if edge.end == last && !path.contains(&edge.start) {
                    Some(edge.start)
                } else {
                    None
                }
            })
            .collect();

        let mut branches: Vec<Vec<Vec<NodeId>>> = neighbours
            .into_par_iter()
            .map(|n| {
                let mut new_path = path.clone();
                new_path.push(n);
                self.par_explore(new_path, depth, targets)
            })
            .collect();
        for mut branch in branches.drain(..) {
            valid.append(&mut branch);
        }
        valid
    }

    pub fn maximize_paths<F>(
        &self,
        paths: Vec<Vec<NodeId>>,

        stat_selector: F,
        min_bonus: f32,
        max_length: usize,
    ) -> Vec<Vec<NodeId>>
    where
        F: Fn(&Stat) -> Option<f32>,
    {
        let mut scored: Vec<(Vec<NodeId>, f32)> = paths
            .into_iter()
            .filter_map(|path| {
                let bonus: f32 = path
                    .iter()
                    .map(|node_id| {
                        let poe_node = self.nodes.get(node_id).unwrap();
                        let skill = poe_node.as_passive_skill(self);
                        skill
                            .stats()
                            .iter()
                            .filter_map(|s| stat_selector(s))
                            .sum::<f32>()
                    })
                    .sum();

                if bonus >= min_bonus {
                    Some((path, bonus))
                } else {
                    if bonus >= (min_bonus / 2.0) {
                        log::trace!("Rejecting path with summed bonus of: {}", bonus);
                    }
                    None
                }
            })
            .collect();
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scored
            .into_iter()
            .take_while(|p| p.0.len() <= max_length)
            .map(|(path, _)| path)
            .collect()
    }

    pub fn filter_winners<F>(&self, ser_res: Vec<Vec<NodeId>>, predicate: F) -> Vec<Vec<NodeId>>
    where
        F: Fn(&Stat) -> bool,
    {
        let mut winners = vec![];

        for potential in ser_res {
            let mut total_bonus = 0.0;

            for n in &potential {
                let pnode = self.nodes.get(n).unwrap();
                let pskill = self.passive_for_node(pnode);
                let stats = pskill.stats();

                for s in stats {
                    if predicate(s) {
                        total_bonus += s.value();
                    }
                }
            }

            // if total_bonus >= MIN_BONUS_VALUE {
            //     winners.push(potential);
            // }
        }

        winners
    }
}
