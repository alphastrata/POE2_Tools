#![allow(unused_imports, unused_must_use, unused_mut, dead_code)]
use bevy::prelude::*;
use poe_vis::resources::UserConfig;

fn quick_tree() -> poe_tree::PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let mut tree = poe_tree::PassiveTree::from_value(&tree_data).unwrap();

    tree.remove_hidden();
    tree
}
// Main function
fn main() {
    let crate_name = env!("CARGO_PKG_NAME").replace('-', "_");
    let log_filter = format!("{}=trace", crate_name);

    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                filter: log_filter,
                ..Default::default()
            }),
            MeshPickingPlugin,
        ))
        .add_plugins(poe_vis::PoeVis)
        .run();
}
