//! Demonstrating:
//! - usage of BFS from a starting node to a target node
//! - examples of abstracting over the RPC interface that `poe_vis` provides
//! - using rayon with the pathfinding output from methods on PassiveTree

use poe_tree::{consts::get_level_one_nodes, type_wrappings::NodeId, PassiveTree};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::{env, thread::sleep, time::Duration};

const VIS_URL: &str = "http://0.0.0.0:6004";

fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    PassiveTree::from_value(&tree_data).unwrap()
}

fn send_node_command(client: &Client, node: u32, method: &str) {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"{}","params":[{}],"id":1}}"#,
        method, node
    );
    let res = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send();
    if let Err(e) = res {
        eprintln!("Error sending {} for node {}: {}", method, node, e);
    }
}

fn activate_node(client: &Client, node: u32) {
    send_node_command(client, node, "activate_node");
}

fn deactivate_node(client: &Client, node: u32) {
    send_node_command(client, node, "deactivate_node");
}

fn ping(client: &reqwest::blocking::Client) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let json = r#"{"jsonrpc":"2.0","method":"ping","params":[],"id":1}"#;
    client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send()
}
fn main() {
    pretty_env_logger::init();
    let visualiser = env::args().any(|arg| arg == "--visualiser");
    let client = Client::new();
    let mut tree = quick_tree();
    tree.remove_hidden();
    let chaos_inoculation: NodeId = 56349;

    if visualiser && ping(&client).is_err() {
        eprintln!("You have requested that this example app show you the visualistions of the paths it creates. For this to work poe_vis's `vis` binary must be running and available on port {VIS_URL}");
        std::process::exit(1)
    }

    // Forcibly collect these so we can, if we want to trivially use rayon.
    let nodes: Vec<(&'static str, &[u32; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();

    // Give yourself time to make sure the Visualiser is up and focused etc..
    sleep(Duration::from_millis(500));

    nodes.par_iter().for_each(|(character, node_ids)| {
        let local_client = client.clone();
        node_ids.iter().for_each(|&start_node| {
            let paths = tree.bfs(start_node, chaos_inoculation);
            if visualiser {
                paths.iter().for_each(|node| {
                    // Activate path nodes
                    sleep(Duration::from_millis(10));
                    activate_node(&local_client, *node);
                });
                sleep(Duration::from_millis(115));
                // Deactivate path nodes
                paths.iter().for_each(|node| {
                    deactivate_node(&local_client, *node);
                    sleep(Duration::from_millis(25));
                });
            }
            println!("Shortest path is {} steps for {character}", paths.len())
        });
    });
}
