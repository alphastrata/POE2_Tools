use std::collections::HashMap;

use poe_tree::{consts::CHAR_START_NODES, stats::Operand, PassiveTree};

/// Helper function to truncate the path to the first 3 and last 3 NodeIds
fn truncate_path(path: &[u32]) -> String {
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

fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").expect("Failed to open POE2_Tree.json");
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value =
        serde_json::from_reader(reader).expect("Failed to parse JSON");
    PassiveTree::from_value(&tree_data).expect("Failed to create PassiveTree")
}

fn main() {
    pretty_env_logger::init();
    let tree = quick_tree();

    // Mapping of start node IDs to class names
    let char_start_nodes: HashMap<u32, &str> = HashMap::from([
        (50459, "Ranger"),
        (47175, "Warrior"),
        (50986, "Mercenary"),
        // (61525, "???"),
        (54447, "Witch or Sorceress"),
        (44683, "Monk"),
    ]);

    let levels = vec![50, 99, 123];
    let keyword = "lightning";
    let min_value = 1.0f32;

    // Extract start node IDs
    let start_node_ids: Vec<u32> = char_start_nodes.keys().cloned().collect();

    // Use the library's parallel Dijkstra function to get paths
    let paths = tree.parallel_dijkstra_with_all_paths(&start_node_ids, &levels);

    for ((start, level), path) in &paths {
        // Get the class name from the start node ID
        let class_name = char_start_nodes.get(start).unwrap_or(&"Unknown");

        // Truncate the path
        let truncated_path = truncate_path(path);

        // Calculate sum and count of stats using iterators
        let (_num_nodes, sum) = path.iter().fold((0, 0.0f32), |(count, acc), &node_id| {
            if let Some(poe_node) = tree.nodes.get(&node_id) {
                let skill = poe_node.as_passive_skill(&tree);
                let node_sum: f32 = skill
                    .stats
                    .iter()
                    .filter(|s| {
                        s.name.contains(keyword)
                            && s.value > min_value
                            && matches!(s.operand, Operand::Percentage)
                    })
                    .map(|s| {
                        println!("total+={:?}", s.value);
                        s.value
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

        // Debugging: Inspect stats on nodes with non-zero contributions
        for &node_id in path {
            if let Some(poe_node) = tree.nodes.get(&node_id) {
                let skill = poe_node.as_passive_skill(&tree);
                for stat in &skill.stats {
                    if stat.name == keyword {
                        println!("Node {}: {} = {} {:?}", node_id, stat.name, stat.value, stat.operand);
                    }
                }
            }
        }
        

        if sum > 0.0 {
            println!(
                "{} @{} can obtain {:.1} to {} with path: {}",
                class_name, level, sum, keyword, truncated_path
            );
        }
    }
}
