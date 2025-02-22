/// Optimisers go here.
use ahash::AHashMap;
use std::collections::VecDeque;

use crate::{stats::Stat, type_wrappings::NodeId, PassiveTree};

// TODO: Eventually we'll have different types of optimiser, probably suited to different things...
/// Maybe we make each one its own thing...
pub struct Optimiser {
    pub results: Vec<Vec<NodeId>>,
}

//NOTE: hard to call this an optimiser per se, it just brute forces a 'better' path...
impl PassiveTree {
    ///
    pub fn collect_stats<F>(
        tree: &PassiveTree,
        path: &[NodeId],
        predicate: F,
    ) -> AHashMap<String, f32>
    where
        F: Fn(&Stat) -> bool,
    {
        let mut map = AHashMap::new();
        path.into_iter()
            .flat_map(|nid| tree.nodes.get(nid))
            .for_each(|pnode| {
                let skill = pnode.as_passive_skill(tree);
                skill.stats().into_iter().for_each(|s| {
                    if predicate(s) {
                        *map.entry(format!("{:?}", s)).or_default() += s.value();
                    }
                });
            });
        map
    }

    /// Takes while the predicate's criteria is strictly better than what is calculated from the baseline's Stat(s)
    pub fn take_while_better<F>(
        &self,
        start: NodeId,
        predicate: F,
        depth: usize,
        baseline: &[NodeId],
    ) -> Vec<Vec<NodeId>>
    where
        F: Fn(&Stat) -> bool,
    {
        fn pareto_better(a: &AHashMap<String, f32>, b: &AHashMap<String, f32>) -> bool {
            let mut better = false;
            for (k, &bv) in b {
                let av = a.get(k).copied().unwrap_or(0.0);
                if av < bv {
                    return false;
                }
                if av > bv {
                    better = true;
                }
            }
            better
        }

        let base_map = Self::collect_stats(self, baseline, &predicate);
        let mut valid = vec![];
        let mut queue = VecDeque::new();
        queue.push_back(vec![start]);

        while let Some(path) = queue.pop_front() {
            if path.len() > depth {
                continue;
            }
            if path.len() == depth {
                let stats = Self::collect_stats(self, &path, &predicate);
                if pareto_better(&stats, &base_map) {
                    valid.push(path);
                }
                continue;
            }
            let last = *path.last().unwrap();
            self.edges.iter().for_each(|edge| {
                let (a, b) = (edge.start, edge.end);
                if a == last && !path.contains(&b) {
                    let mut new_path = path.clone();
                    new_path.push(b);
                    queue.push_back(new_path);
                } else if b == last && !path.contains(&a) {
                    let mut new_path = path.clone();
                    new_path.push(a);
                    queue.push_back(new_path);
                }
            });
        }

        // Sort results by total stat sum (descending)
        valid.sort_by(|a, b| {
            let sum_a: f32 = Self::collect_stats(self, a, &predicate).values().sum();
            let sum_b: f32 = Self::collect_stats(self, b, &predicate).values().sum();
            sum_b.partial_cmp(&sum_a).unwrap()
        });
        valid
    }
}

// TODO: an optimiser that only provides 'swaps'

// TODO: an optimiser that can take in 'pins' i.e MUST HAVE nodes' IDs then make paths adhering to those constraints

// TODO: an optimiser that can ensure a minimum +% worth of a Node's possible Stats in <= x moves.
