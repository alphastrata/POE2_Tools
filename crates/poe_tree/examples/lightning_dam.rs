use poe_tree::consts::CHAR_START_NODES;

fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    PassiveTree::from_value(&tree_data).unwrap()
}

fn main() {
    let tree = quick_tree();

    
    let keyword = "evasion_rating_+%";
    let min_value = 15.0f32;

    let mut num_nodes = 0;

    let sum: f32 = tree
        .nodes
        .iter()
        .filter_map(|(_node_id, poe_node)| {
            let skill = poe_node.as_passive_skill(&tree);
            let stat_sum: f32 = skill
                .stats
                .iter()
                .filter(|s| {
                    s.name == keyword
                        && s.value == min_value
                        && matches!(s.operand, Operand::Percentage)
                })
                .map(|s| s.value)
                .sum();

            if stat_sum > 0.0 {
                num_nodes += 1;
                Some(stat_sum)
            } else {
                None
            }
        })
        .sum();

    // Specify the levels up to which we'll evaluate paths
    let levels = vec![20, 40];

    // Get paths using Dijkstra
    let paths = tree.dijkstra_paths(&CHAR_START_NODES, &levels);

    for ((start, level), path) in paths {
        println!(
            "Path from start node {start} to level {level}: {:?}",
            path
        );
    }
}
