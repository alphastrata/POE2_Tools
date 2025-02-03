use rand::prelude::IndexedRandom;
use reqwest::blocking::Client;

mod common;
use common::*;

fn main() {
    let tree = common::quick_tree();
    let client = Client::new();

    if let Err(e) = common::ping(&client) {
        panic!("{}", e);
    }

    let colour_options = get_colours(&client).unwrap();

    if colour_options.is_empty() {
        eprintln!("No colours available.");
        return;
    }

    let mut rng = rand::rng();

    tree.nodes
        .iter()
        .for_each(|(nid, _node)| match colour_options.choose(&mut rng) {
            Some(colour) => {
                log::debug!(
                    "Custom highlight requested for {} with colour: {}",
                    nid,
                    colour,
                );

                common::activate_node_with_colour(&client, *nid, colour);
                std::thread::sleep(std::time::Duration::from_millis(125));
            }
            None => {
                eprintln!("Bit problems... {}", colour_options.len());
            }
        });
}
