use rayon::prelude::*;
use std::collections::HashMap;

mod common;
use common::quick_tree;
use poe_tree::{stats::Stat, type_wrappings::NodeId};

fn truncate_path(path: &[NodeId]) -> String {
    if path.len() <= 6 {
        format!("{:?}", path)
    } else {
        let first = &path[..3];
        let last = &path[path.len() - 3..];
        let first_str = first
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let last_str = last
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{}, ..., {}]", first_str, last_str)
    }
}

fn main() {
    pretty_env_logger::init();
    let tree = quick_tree();

    let char_start_nodes: HashMap<NodeId, &str> = HashMap::from([(54447, "Witch or Sorceress")]);

    let levels: [u16; 1] = [123];
    let keyword = "lightning_damage_+%";
    let min_value = 1.0f32;

    // Build targets from nodes with the keyword.
    let potential_destinations: Vec<NodeId> = tree
        .nodes
        .iter()
        .filter(|(_, pnode)| pnode.contains_stat_with_keyword(&tree, keyword))
        .map(|(nid, _)| nid.clone())
        .collect();
    log::info!("Num nodes with keyword {}", potential_destinations.len());

    let start_node_ids: Vec<NodeId> = char_start_nodes.keys().cloned().collect();

    // For each level & start, do BFS to any target.
    let a_tree = std::sync::Arc::new(&tree);
    let results: Vec<((NodeId, u16), Vec<NodeId>, f32)> = levels
        .iter()
        .flat_map(|&level| {
            let a_tree = a_tree.clone();
            let level = level.clone();
            let potential_destinations = std::sync::Arc::new(&potential_destinations);
            start_node_ids.iter().map(move |start| {
                let path = a_tree.bfs_any(start.clone(), &potential_destinations);
                let sum: f32 = path.iter().fold(0.0, |acc, &node_id| {
                    if let Some(poe_node) = a_tree.nodes.get(&node_id) {
                        let skill = poe_node.as_passive_skill(&a_tree);
                        acc + skill
                            .stats()
                            .iter()
                            .filter(|s| s.name().contains(keyword) && s.value() > min_value)
                            .map(|s| s.value())
                            .sum::<f32>()
                    } else {
                        acc
                    }
                });
                ((start.clone(), level.clone()), path, sum)
            })
        })
        .collect();

    // Process results in parallel.
    results.par_iter().for_each(|((start, level), path, sum)| {
        let class_name = char_start_nodes.get(start).unwrap_or(&"Unknown");
        if !path.is_empty() && *sum > 0.0 && path.len() <= *level as usize {
            let truncated = truncate_path(path);
            log::info!(
                "{} @{} can obtain {:.1} to {} with path: {}",
                class_name,
                level,
                sum,
                keyword,
                truncated
            );
        } else {
            log::debug!(
                "No bonus for '{}' for {} at level {}",
                keyword,
                class_name,
                level
            );
        }
    });

    // NOTE: you can hovewever, be more articulate about what you 'want', by using the enum variants on Stat, which is much more performant than string matching.
    let potential_destinations: Vec<NodeId> = tree
        .nodes
        .iter()
        .map(|(nid, pnode)| (nid, pnode.as_passive_skill(&tree)))
        .filter(|(_nid, passive)| {
            passive.stats().iter().any(|s| {
                matches!(
                    s,
                    Stat::LightningDamage(_)
                        | Stat::LightningDamageWhileAffectedByHeraldOfThunder(_)
                        | Stat::LightningExposureEffect(_) // and you can just keep going...
                )
            })
        })
        .map(|(nid, _)| *nid)
        .collect();

    log::info!("Num matches {}", potential_destinations.len());

    // For each level & start, do BFS to any target.
    let a_tree = std::sync::Arc::new(&tree);
    let results: Vec<((NodeId, u16), Vec<NodeId>, f32)> = levels
        .iter()
        .flat_map(|&level| {
            let a_tree = a_tree.clone();
            let level = level.clone();
            let potential_destinations = std::sync::Arc::new(&potential_destinations);
            start_node_ids.iter().map(move |start| {
                let path = a_tree.bfs_any(start.clone(), &potential_destinations);
                let sum: f32 = path.iter().fold(0.0, |acc, &node_id| {
                    if let Some(poe_node) = a_tree.nodes.get(&node_id) {
                        let skill = poe_node.as_passive_skill(&a_tree);
                        acc + skill
                            .stats()
                            .iter()
                            .filter(|s| s.name().contains(keyword) && s.value() > min_value)
                            .map(|s| s.value())
                            .sum::<f32>()
                    } else {
                        acc
                    }
                });
                ((start.clone(), level.clone()), path, sum)
            })
        })
        .collect();

    // Process results in parallel.
    results.par_iter().for_each(|((start, level), path, sum)| {
        let class_name = char_start_nodes.get(start).unwrap_or(&"Unknown");
        if !path.is_empty() && *sum > 0.0 && path.len() <= *level as usize {
            let truncated = truncate_path(path);
            log::info!(
                "{} @{} can obtain {:.1} to {} with path: {}",
                class_name,
                level,
                sum,
                keyword,
                truncated
            );
        } else {
            log::debug!(
                "No bonus for '{}' for {} at level {}",
                keyword,
                class_name,
                level
            );
        }
    });
}
