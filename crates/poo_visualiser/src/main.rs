//$ crates/poo_visualiser/src/main.rs

use std::{fs::File, io::BufReader};

fn main() {
    pretty_env_logger::init();

    // Load the user config
    let config: UserConfig = UserConfig::load_from_file("data/user_config.toml");

    let file = File::open("data/POE2_Tree.json").unwrap();
    let reader = BufReader::new(file);
    let u = serde_json::from_reader(reader).unwrap();

    let mut tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

    // There's alot of noise in the data for atlas passives etc that we don't plot.
    tree.remove_hidden();

    // Load the character data, defaulting to `None` if the file is missing or invalid
    let character = UserCharacter::load_from_toml("character.toml");

    println!(
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
        "poo-tools",
        options,
        Box::new(|_cc| Ok(Box::new(TreeVis::new(&mut tree, config, character)))),
    );
}
