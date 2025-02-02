mod common;

fn main() {
    let mut tree = common::quick_tree();
    println!("Total number of nodes [ALL] : {}", tree.nodes.len());
    println!("Total number of edges [ALL]: {}", tree.edges.len());

    tree.remove_hidden(); // This prunes everything NOT in the default passives, so no ascendencies etc.
    println!(
        "Total number of nodes [JUST THE VANILLA PASSIVES]: {}",
        tree.nodes.len()
    );
    println!(
        "Total number of edges [JUST THE VANILLA PASSIVES]: {}",
        tree.edges.len()
    );
    /*
    Total number of nodes [ALL] : 3557
    Total number of nodes [JUST THE VANILLA PASSIVES]: 2902
     */
}
