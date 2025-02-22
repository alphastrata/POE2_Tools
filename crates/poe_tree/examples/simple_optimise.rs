pub struct Optimiser {
    pub results: Vec<Vec<NodeId>>,
    pub starting_path: Vec<NodeId>,
    pub allowable_swaps: u8,
}

use ahash::{AHashMap, HashSet};
use poe_tree::{stats::Stat, type_wrappings::NodeId, PassiveTree};
mod common;

fn main() {
    let mut tree = common::quick_tree();

    let start_warrior: NodeId = 47175;

    let starting_path: HashSet<NodeId> = [47175, 3936, 46325, 39581, 6839, 5710]
        .into_iter()
        .collect();
    // Convert baseline to Vec<NodeId>
    let baseline: Vec<NodeId> = starting_path.iter().copied().collect();
    let allowable_delta = 7;
    let mut best_path: Vec<NodeId> = [47175, 3936, 43164, 5710, 33556, 55473, 46325]
        .into_iter()
        .collect();
    best_path.sort();

    let s = |s: &Stat| {
        matches!(
            s,
            Stat::MeleeDamage(_)
                | Stat::PhysicalDamage(_)
                | Stat::AttackDamage(_)
                | Stat::MeleeDamageAtCloseRange(_)
        )
    };

    let branchpoints = tree.branches(&starting_path);
    let searchspace = branchpoints
        .iter()
        .flat_map(|b| tree.take_while_better(*b, s, allowable_delta, &baseline));

    let mut results: Vec<Vec<NodeId>> = searchspace.into_iter().collect();

    let s = |s: &Stat| {
        matches!(
            s,
            Stat::MeleeDamage(_)
                | Stat::PhysicalDamage(_)
                | Stat::AttackDamage(_)
                | Stat::MeleeDamageAtCloseRange(_)
        )
    };

    for path in &results {
        let mut stat_map: AHashMap<String, f32> = AHashMap::new();
        path.iter()
            .flat_map(|nid| tree.nodes.get(nid))
            .flat_map(|pnode| {
                let skill = pnode.as_passive_skill(&tree);
                skill.stats().iter().filter(|stat| s(stat))
            })
            .for_each(|stat| {
                *stat_map.entry(stat.as_str().to_string()).or_insert(0.0) += stat.value();
            });

        println!("Path: {:?}", path);
        for (key, total) in &stat_map {
            println!("\t{}: {}", key, total);
        }
    }

    assert!(
        results.iter_mut().any(|v| {
            v.sort();
            v == &best_path.to_vec()
        }),
        "{:#?}",
        results
    );
}
