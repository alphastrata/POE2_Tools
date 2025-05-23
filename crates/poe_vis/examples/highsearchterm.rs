// ... (imports remain unchanged)
use core::panic;
use rayon::prelude::*;
use reqwest::blocking::Client;
use std::{
    env,
    thread::sleep,
    time::{Duration, Instant},
};

mod common;
use common::*;
use poe_tree::{consts::LEVEL_ONE_NODES, type_wrappings::NodeId};

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
    const LVL_CAP: usize = 40;
    // 100 should be possible of 31 lvls..
    const MIN_BONUS_VALUE: f32 = 80.0;

    //TODO: benchmark take_while
    //TODO: benchmark maximise_paths
    //TODO: implement keystone idea (nodes that are always hit on paths)
    //TODO: flip the directions on the bfs, go broad first?
    //TODO: implement deadend nodes (nodes to immediately break from on a proximity keyword search nodes' wx, wy), for example all the chaos nodes are at the top of the board, so heading south is almost always a waste of time.

    let t1 = Instant::now();
    let filtered: Vec<Vec<NodeId>> = tree
        .maximize_paths(
            tree.take_while(
                start_node,
                |s| matches!(s, poe_tree::stats::Stat::ChaosDamage(_)),
                LVL_CAP,
            ),
            |s| Some(s.value()),
            MIN_BONUS_VALUE,
            LVL_CAP,
        )
        .into_iter()
        .collect();

    println!(
        "{} after filter. in {}s",
        filtered.len(),
        t1.elapsed().as_secs_f64()
    );

    let f_20 = filtered.par_iter();
    let total_secs = f_20.len() as f64 * 0.035;
    let hours = (total_secs / 3600.0) as u64;
    let minutes = ((total_secs % 3600.0) / 60.0) as u64;
    let seconds = (total_secs % 60.0).round() as u64;

    println!(
        "Estimated animtaion time is: {hours:02}:{minutes:02}:{seconds:02}"
    );
    if filtered.is_empty() {
        panic!(
            "Filtering was too strict for {}, try a smaller value or more levels!",
            MIN_BONUS_VALUE
        );
    }
    println!("Play the animation (Y/N)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();

    filtered.par_iter().enumerate().for_each(|(i, path)| {
        let colour = &colours[i % colours.len()];
        if visualiser {
            if let Some(&first) = path.first() {
                activate_node_with_colour(&client, first, colour);
            }

            path.windows(2).for_each(|window| {
                let (from, to) = (window[0], window[1]);
                common::activate_edge_with_colour(&client, from, to, colour);
                common::activate_node_with_colour(&client, to, colour);
                sleep(Duration::from_millis(8));
            });
            sleep(Duration::from_millis(22));
        }
    });
}
