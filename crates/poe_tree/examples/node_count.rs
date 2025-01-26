use poe_tree::PassiveTree;

fn main() {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let mut tree = PassiveTree::from_value(&tree_data).unwrap();
    println!("Total number of nodes [ALL] : {}", tree.nodes.len());

    tree.remove_hidden(); // This prunes everything NOT in the default passives, so no ascendencies etc.
    println!(
        "Total number of nodes [JUST THE VANILLA PASSIVES]: {}",
        tree.nodes.len()
    );
    /*
    Total number of nodes [ALL] : 3557
    Total number of nodes [JUST THE VANILLA PASSIVES]: 2902
     */
}
