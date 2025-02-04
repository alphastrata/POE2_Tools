//! Demonstrating:
//! - usage of `walk_n_steps` in serial and parallel.
//! - examples of abstracting over the RPC interface that `poe_vis` provides
//! - using rayon with the pathfinding output from methods on PassiveTree

use poe_tree::{consts::get_level_one_nodes, edges::Edge, type_wrappings::NodeId};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::{
    env,
    thread::sleep,
    time::{Duration, Instant},
};

mod common;
use common::*;

fn main() {
    pretty_env_logger::init();
    let visualiser = env::args().any(|arg| arg == "--visualiser");
    let client = Client::new();
    let mut tree = quick_tree();
    tree.remove_hidden();

    let nodes: Vec<(&'static str, &[NodeId; 2])> = get_level_one_nodes()
        .iter()
        .map(|(name, ids)| (*name, ids))
        .collect();

    if visualiser && ping(&client).is_err() {
        eprintln!("You have requested that this example app show you the visualistions of the paths it creates. For this to work poe_vis's `vis` binary must be running and available on port {VIS_URL}");
        std::process::exit(1)
    }

    // Less garbled output to stdout in the serial case.
    // nodes.iter().for_each(|(character, node_ids)| {
    nodes.par_iter().for_each(|(character, node_ids)| {
        let local_client = client.clone();
        let char_start = Instant::now();
        println!("{}:", character);
        node_ids.iter().for_each(|&start_node| {
            println!("\tStart node: {}", start_node);
            //NOTE: these numbers are kept low to spare your hardware && to save
            // you life hours of watching the paths... it is rather hypnotic.
            // You're welcome.
            [40].iter().for_each(|&steps| {
                let paths = tree.walk_n_steps(start_node, steps);
                assert!(
                    !paths.is_empty(),
                    "No paths found for start {} and {} steps",
                    start_node,
                    steps
                );

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
                    paths.iter().for_each(|path| {
                        // Activate path nodes
                        path.iter().for_each(|&node| {
                            activate_node(&local_client, node);
                            sleep(Duration::from_millis(15));
                        });
                        sleep(Duration::from_millis(175));
                        // Deactivate path nodes
                        path.iter().for_each(|&node| {
                            deactivate_node(&local_client, node);
                            sleep(Duration::from_millis(10));
                        });
                    });
                }
                println!(
                    "\t\tLevels {}: {} possible paths for {}",
                    steps,
                    paths.len(),
                    character
                );
            });
            println!("{} finished in: {:?}\n", character, char_start.elapsed());
        });
    });
}
