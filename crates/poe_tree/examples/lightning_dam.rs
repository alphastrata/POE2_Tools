use std::collections::HashMap;

mod common;
use common::quick_tree;
use poe_tree::type_wrappings::NodeId;

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

    let char_start_nodes: HashMap<NodeId, &str> = HashMap::from([
        (50459, "Ranger"),
        (47175, "Warrior"),
        (50986, "Mercenary"),
        (54447, "Witch or Sorceress"),
        (44683, "Monk"),
    ]);

    let levels: [u16; 3] = [10, 20, 30]; // NOTE: 30+ MAY not be possible on your system, proceed with caution.
    let keyword = "lightning";
    let min_value = 1.0f32;

    let start_node_ids: Vec<NodeId> = char_start_nodes.keys().cloned().collect();

    let paths: Vec<((NodeId, u16), Vec<NodeId>)> = levels
        .iter()
        .flat_map(|level| {
            start_node_ids.iter().flat_map(|start| {
                // we can clone all day everyday when it's 1/4 of a pointer.
                let start = start.clone();
                let level = level.clone();
                tree.walk_n_steps(start, level as usize)
                    .into_iter()
                    .map(move |path| ((start.clone(), level.clone()), path))
            })
        })
        .collect();

    for ((start, level), path) in &paths {
        let class_name = char_start_nodes.get(start).unwrap_or(&"Unknown");
        let truncated_path = truncate_path(path);

        let (_num_nodes, sum) = path.iter().fold((0, 0.0f32), |(count, acc), &node_id| {
            if let Some(poe_node) = tree.nodes.get(&node_id) {
                let skill = poe_node.as_passive_skill(&tree);
                let node_sum: f32 = skill
                    .stats()
                    .iter()
                    .filter(|s| s.name().contains(keyword) && s.value() > min_value)
                    .map(|s| {
                        println!("total+={:?}", s.value());
                        s.value()
                    })
                    .sum();
                if node_sum > 0.0 {
                    (count + 1, acc + node_sum)
                } else {
                    (count, acc)
                }
            } else {
                (count, acc)
            }
        });

        if sum > 0.0 {
            println!(
                "{} @{} can obtain {:.1} to {} with path: {}",
                class_name, level, sum, keyword, truncated_path
            );
        }
    }
}
