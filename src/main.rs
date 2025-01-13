use poo_tools::{data::PassiveTree, visualiser::TreeVis};
use std::collections::HashMap;

fn main() {
    let data = PassiveTree::load_tree("data/POE2_TREE.json");

    println!(
        "Found {} nodes and {} groups",
        data.nodes.len(),
        data.groups.len(),
    );

    // Load color_map from tree_config.toml
    let config_str = std::fs::read_to_string("tree_config.toml").unwrap();
    let config: toml::Value = toml::from_str(&config_str).unwrap();

    let mut color_map = HashMap::new();
    if let Some(colors) = config.get("colors").and_then(|v| v.as_table()) {
        for (k, v) in colors {
            if let Some(col_str) = v.as_str() {
                color_map.insert(k.clone(), col_str.to_string());
            }
        }
    }

    let native_opts = eframe::NativeOptions::default();
    _ = eframe::run_native(
        "POE2_TREE debug vis tool",
        native_opts,
        Box::new(|_cc| Ok(Box::new(TreeVis::new(data, color_map)))),
    );
}
