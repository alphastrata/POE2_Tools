use poe_tree::{consts::get_level_one_nodes, type_wrappings::NodeId};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::{collections::HashMap, thread::sleep, time::Duration};

mod common;
use common::*;

const STEPS: usize = 20;
const FPS: u64 = 100; // 10 activations per second

fn main() {
    pretty_env_logger::init();
    let client = Client::new();
    let mut tree = quick_tree();
    tree.remove_hidden();

    let nodes: Vec<(&'static str, &[NodeId; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();

    let tailwind_colours = [
        "red-500",
        "blue-500",
        "green-500",
        "yellow-500",
        "purple-500",
        "pink-500",
        "indigo-500",
        "teal-500",
        "cyan-500",
        "orange-500",
        "lime-500",
        "amber-500",
    ];

    let rng = rand::rng();
    let colour_map = nodes
        .iter()
        .enumerate()
        .map(|(i, (character, _))| (*character, tailwind_colours[i % tailwind_colours.len()]))
        .collect::<HashMap<_, _>>();

    if ping(&client).is_err() {
        eprintln!("Error: poe_vis's `vis` binary must be running.");
        std::process::exit(1);
    }

    let mut node_counts: HashMap<NodeId, usize> = HashMap::new();

    let a_node_counts = std::sync::Arc::new(std::sync::Mutex::new(&mut node_counts));
    nodes.par_iter().for_each(|(character, node_ids)| {
        let local_client = client.clone();
        let colour = *colour_map.get(character).unwrap_or(&"gray-500");

        let mut seen_nodes = std::collections::HashSet::new(); // Track unique visits per character

        node_ids.iter().for_each(|&start_node| {
            let paths = tree.walk_n_steps(start_node, STEPS);
            if paths.is_empty() {
                eprintln!("No paths found for {} at {} steps", start_node, STEPS);
                return;
            }

            paths.iter().for_each(|path| {
                path.iter().for_each(|&node| {
                    // activate_node_with_colour(&local_client, node, colour);
                    seen_nodes.insert(node); // Ensure we count per character
                });

                path.iter().for_each(|&node| {
                    // deactivate_node(&local_client, node);
                });
            });
        });

        // Update node_counts once per character
        seen_nodes.into_iter().for_each(|node| {
            let mut entry = a_node_counts.lock().unwrap();
            *entry.entry(node.clone()).or_insert(0) += 1;
        });
    });

    // Print the node counts after each loop
    println!("\n--- NODE HIT COUNTS ---");
    for (node, count) in &node_counts {
        println!("Node {}: {}", node, count);
        // activate_node_with_colour(&client, *node, "WHITE");
    }
    println!("----------------------\n");
}
