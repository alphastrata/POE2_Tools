mod common;
use common::*;
use poe_tree::type_wrappings::NodeId;

use reqwest::blocking::Client;
use std::collections::HashSet;

fn main() {
    let mut tree = quick_tree();
    tree.remove_hidden();

    let client = Client::new();

    let visualiser = std::env::args().any(|arg| arg == "--visualiser");
    if visualiser && ping(&client).is_err() {
        eprintln!("You have requested that this example app show you the visualisations of the paths it creates. For this to work poe_vis's `vis` binary must be running and available on port {VIS_URL}");
        std::process::exit(1)
    }

    let active_nodes: HashSet<NodeId> = [
        10364, 55342, 17248, 11604, 31765, 15775, 61196, 14267, 51741, 8975, 17088, 57821, 722,
        61834, 56045, 53960, 30808, 2361, 32442, 35696, 12253, 11504, 30839, 41017, 35671, 14262,
        32763, 38728, 44776, 14539, 48773, 26034, 3630, 45631, 11598, 37514, 34316, 4536, 31172,
        51707, 62624,
    ]
    .into_iter()
    .collect();

    let passive_branches = tree.branches(&active_nodes);
    if visualiser {
        passive_branches
            .iter()
            .enumerate()
            .for_each(|(_i, branch)| {
                let pos = common::get_node_position(&client, *branch);
                common::draw_circle(&client, 190.0, pos, "pink-500", 60000);
            });
    }
}
