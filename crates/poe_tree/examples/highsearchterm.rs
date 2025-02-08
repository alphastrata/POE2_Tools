// ... (imports remain unchanged)
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::{
    env,
    thread::sleep,
    time::{Duration, Instant},
};

mod common;
use common::*;
use poe_tree::consts::LEVEL_ONE_NODES;
use poe_tree::stats::Stat;

fn main() {
    pretty_env_logger::init();
    let visualiser = env::args().any(|arg| arg == "--visualiser");
    let client = Client::new();
    let mut tree = quick_tree();
    tree.remove_hidden();
    let start_node = LEVEL_ONE_NODES[2];

    if visualiser && ping(&client).is_err() {
        eprintln!("Visualiser requested but ping failed.");
        std::process::exit(1)
    }

    let mut colours = get_colours(&client).unwrap();
    if colours.is_empty() {
        eprintln!("No colours available.");
        return;
    }
    colours.sort();

    let t1 = Instant::now();
    let potential_destinations =
        tree.take_while(start_node, |s| matches!(s, Stat::ChaosDamage(_)), 23);
    log::info!("{}ms", t1.elapsed().as_millis());

    potential_destinations
        .par_iter()
        .enumerate()
        .for_each(|(i, path)| {
            let colour = &colours[i % colours.len()];
            log::info!(
                "Path of len:{} ready in {:#?}secs",
                path.len(),
                t1.elapsed().as_secs_f64()
            );
            if visualiser {
                // Activate first node.
                if let Some(&first) = path.first() {
                    activate_node_with_colour(&client, first, colour);
                    sleep(Duration::from_millis(15));
                }
                // Activate edges and nodes via windows(2).
                for window in path.windows(2) {
                    let (from, to) = (window[0], window[1]);
                    common::activate_edge_with_colour(&client, from, to, colour);
                    activate_node_with_colour(&client, to, colour);
                    sleep(Duration::from_millis(15));
                }
                sleep(Duration::from_millis(85));
                // Deactivate edges in reverse.
                for window in path.windows(2).rev() {
                    let (from, to) = (window[0], window[1]);
                    // common::deactivate_edge(&client, from, to);
                    sleep(Duration::from_millis(10));
                }
                // Deactivate the first node.
                if let Some(&first) = path.first() {
                    deactivate_node(&client, first);
                }
            }
        });
}
