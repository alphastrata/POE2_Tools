use bevy::prelude::*;
use poe_tree::calculate_world_position;
use poe_tree::PassiveTree; // Add this import

pub mod camera;

pub mod nodes;
fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let mut tree = PassiveTree::from_value(&tree_data).unwrap();

    tree.remove_hidden();
    tree
}
// Main function
fn main() {
    let passive_tree = quick_tree();

    App::new()
        .insert_resource(nodes::PassiveTreeWrapper { tree: passive_tree })
        .add_plugins(DefaultPlugins)
        .add_plugins(nodes::PoeVis)
        .run();
}
