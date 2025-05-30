use poe_tree::{consts::get_level_one_nodes, type_wrappings::NodeId};
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::{
    env, mem,
    thread::sleep,
    time::{Duration, Instant},
};

mod common;
use common::*;

// 40 should be possible for most hardware.
const STEPS: usize = 22;

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
        eprintln!("You have requested that this example app show you the visualisations of the paths it creates. For this to work poe_vis's `vis` binary must be running and available on port {VIS_URL}");
        std::process::exit(1)
    }

    nodes.par_iter().for_each(|(character, node_ids)| {
        let local_client = client.clone();
        let char_start = Instant::now();
        println!("{character}:");
        node_ids.iter().for_each(|&start_node| {
            println!("\tStart node: {start_node}");

            let paths: Vec<Vec<NodeId>> = tree.walk_n_steps(start_node, STEPS);

            println!(
                "\t\tLevels {}: {} possible paths for {}",
                STEPS,
                paths.len(),
                character
            );
            let size_bytes = mem::size_of_val(&paths);
            println!("\t\t{} = {})", character, bytes_to_human(size_bytes));

            paths.iter().for_each(|path| {
                if visualiser {
                    if let (Some(first), Some(last)) = (path.first(), path.last()) {
                        let first_pos = common::get_node_position(&client, *first);
                        let last_pos = common::get_node_position(&client, *last);

                        common::draw_circle(&local_client, 200.0, first_pos, "pink-500", 3000);
                        common::draw_circle(&local_client, 200.0, last_pos, "amber-700", 3000);
                    }

                    // Activate path nodes
                    path.iter().for_each(|&node| {
                        activate_node(&local_client, node);
                        sleep(Duration::from_millis(8));
                    });

                    sleep(Duration::from_millis(34));

                    // Deactivate path nodes
                    path.iter().for_each(|&node| {
                        deactivate_node(&local_client, node);
                        sleep(Duration::from_millis(8));
                    });
                }
            });
        });
        println!("{} finished in: {:?}\n", character, char_start.elapsed());
    });
}
