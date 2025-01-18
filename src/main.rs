//$ src\main.rs
use std::{fs::File, io::BufReader};

use poo_tools::{
    config::{UserCharacter, UserConfig},
    data::prelude::PassiveTree,
    visualiser::TreeVis,
};

fn main() {
    _ = pretty_env_logger::init();

    // Load the user config
    let config: UserConfig = UserConfig::load_from_file("data/user_config.toml");

    let file = File::open("data/POE2_Tree.json").unwrap();
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader).unwrap();

    let mut tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

    // Load the character data, defaulting to `None` if the file is missing or invalid
    let character = UserCharacter::load_from_toml("character.toml");

    println!(
        "Found {} nodes and {} groups",
        tree.nodes.len(),
        tree.groups.len(),
    );

    // Initialize the visualization
    let native_opts = eframe::NativeOptions::default();
    _ = eframe::run_native(
        "POE2_TREE debug vis tool",
        native_opts,
        Box::new(|_cc| Ok(Box::new(TreeVis::new(&mut tree, config, character)))),
    );
}
