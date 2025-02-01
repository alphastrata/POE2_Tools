use poe_tree::{consts::get_level_one_nodes, edges::Edge, PassiveTree};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::{
    env,
    thread::sleep,
    time::{Duration, Instant},
};

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

fn main() {
    pretty_env_logger::init();
    let visualiser = env::args().any(|arg| arg == "--visualiser");
    let client = Client::new();
    let mut tree = quick_tree();
    tree.remove_hidden();

    let nodes: Vec<(&'static str, &[u32; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();

    nodes.par_iter().for_each(|(character, node_ids)| {
        let local_client = client.clone();
        let char_start = Instant::now();
        println!("{}:", character);
        node_ids.iter().for_each(|&start_node| {
            println!("\tStart node: {}", start_node);
            for &steps in &[12] {
                let paths = tree.walk_n_steps(start_node, steps);
                assert!(
                    !paths.is_empty(),
                    "No paths found for start {} and {} steps",
                    start_node,
                    steps
                );
                println!("\t\tLevels {}: {} possible paths", steps, paths.len());
                // Validate edges
                paths.iter().for_each(|path| {
                    path.windows(2).for_each(|pair| {
                        let (from, to) = (pair[0], pair[1]);
                        let edge = Edge {
                            start: from,
                            end: to,
                        };
                        let rev_edge = Edge {
                            start: to,
                            end: from,
                        };
                        assert!(
                            tree.edges.contains(&edge) || tree.edges.contains(&rev_edge),
                            "Invalid edge in path: {:?}",
                            path
                        );
                    });
                });
                if visualiser {
                    for path in &paths {
                        // Activate path nodes
                        for &node in path {
                            activate_node(&local_client, node);
                            sleep(Duration::from_millis(15));
                        }
                        sleep(Duration::from_millis(175));
                        // Deactivate path nodes
                        for &node in path {
                            deactivate_node(&local_client, node);
                            sleep(Duration::from_millis(10));
                        }
                    }
                }
            }
        });
        println!("{} finished in: {:?}\n", character, char_start.elapsed());
    });
}
