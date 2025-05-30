//! Example demonstrating how to extract specific nodes based on a keyword,
//! and calculate both the sum of matching stats and the count of nodes that match.
/*
- Extract nodes structured like this:
    "evasion2": {
        "name": "Evasion",
        "icon": "skillicons/passives/evade",
        "stats": {
            "evasion_rating_+%": 15
        }
    }
- Calculate the sum of the matching stats and count the nodes with matches.
*/

mod common;
use common::quick_tree;

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
                .stats()
                .iter()
                .filter(|s| s.as_str() == keyword && s.value() == min_value)
                .map(|s| s.value())
                .sum();

            if stat_sum > 0.0 {
                num_nodes += 1;
                Some(stat_sum)
            } else {
                None
            }
        })
        .sum();

    println!("Total: {sum}+% from {num_nodes} matching nodes for {keyword}.");
}
