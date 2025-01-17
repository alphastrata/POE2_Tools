//$ src/main.rs
use poo_tools::{
    config::{UserCharacter, UserConfig},
    data::prelude::PassiveTree,
    visualiser::TreeVis,
};

fn main() {
    // Load the user config
    let config: UserConfig = UserConfig::load_from_file("data/user_config.toml");

    // Load the passive tree data
    let (passive_tree, val) = PassiveTree::from_file("data/POE2_TREE.json");

    // Load the character data, defaulting to `None` if the file is missing or invalid
    let character = UserCharacter::load_from_toml("character.toml");

    println!(
        "Found {} nodes and {} groups",
        passive_tree.nodes.len(),
        passive_tree.groups.len(),
    );

    // Initialize the visualization
    let native_opts = eframe::NativeOptions::default();
    _ = eframe::run_native(
        "POE2_TREE debug vis tool",
        native_opts,
        Box::new(|_cc| Ok(Box::new(TreeVis::new(passive_tree, config, character)))),
    );
}
