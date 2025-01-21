//$ crates/poe_vis/src/main.rs

use std::{fs::File, io::BufReader};

use poe_tree::{character::Character, config::UserConfig, PassiveTree};
use poe_vis::TreeVis;

fn main() {
    pretty_env_logger::init();

    let config: UserConfig = UserConfig::load_from_file("data/user_config.toml");
    let file = File::open("data/POE2_Tree.json").unwrap();
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader).unwrap();

    let mut tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

    // There's a lot of noise in the data for atlas passives etc that we don't plot.
    log::debug!("Removing hidden nodes...");
    tree.remove_hidden();
    log::debug!("Hidden nodes removed.");
    // Load the character data, defaulting to `None` if the file is missing or invalid
    let character = Character::load_from_toml("data/character.toml");

    log::debug!(
        "Found {} nodes and {} groups",
        tree.nodes.len(),
        tree.groups.len(),
    );

    // Initialize the visualization
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_decorations(true) // Show OS-specific window decorations
            .with_inner_size([1920.0, 1080.0]) // Set initial window size
            .with_min_inner_size([800.0, 600.0]) // Set a reasonable minimum size
            .with_transparent(false), // Disable transparency for standard window appearance

        ..Default::default()
    };

    _ = eframe::run_native(
        "poe-tools",
        options,
        Box::new(|_cc| Ok(Box::new(TreeVis::new(&mut tree, config, character)))),
    );
}
