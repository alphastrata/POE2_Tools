use reqwest::blocking::Client;

mod common;
use common::*;

const STEPS: usize = 20;

fn main() {
    pretty_env_logger::init();
    let client = Client::new();
    let mut tree = quick_tree();
    tree.remove_hidden();

    let local_client = client.clone();

    let start_node = 44871;

    let paths = tree.walk_n_steps_csr::<STEPS>(start_node, STEPS);

    paths
        .iter()
        .map(|path| {
            dbg!(path.len());
            path
        })
        .for_each(|path| {
            path.iter().for_each(|&node| {
                activate_node_with_colour(&local_client, node, "teal-500");
                // seen_nodes.insert(node);
            });
        });
}
