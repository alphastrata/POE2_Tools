#![allow(dead_code)]
use bevy::math::{Vec2, Vec3};
use poe_tree::{type_wrappings::NodeId, PassiveTree};
use reqwest::blocking::Client;
use serde_json::Value;

pub const VIS_URL: &str = "http://localhost:6004";

pub fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    PassiveTree::from_value(&tree_data).unwrap()
}

pub fn ping(
    client: &reqwest::blocking::Client,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let json = r#"{"jsonrpc":"2.0","method":"ping","params":[],"id":1}"#;
    client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send()
}

fn send_node_command(client: &Client, node: NodeId, method: &str) {
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

pub fn activate_node(client: &Client, node: NodeId) {
    send_node_command(client, node, "activate_node");
}

pub fn deactivate_node(client: &Client, node: NodeId) {
    send_node_command(client, node, "deactivate_node");
}

pub fn get_colours(client: &Client) -> Result<Vec<String>, reqwest::Error> {
    let json = r#"{"jsonrpc": "2.0", "method": "get_available_colours", "params": [], "id": 1}"#;
    let response = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send()?
        .json::<Value>()?;

    if let Some(result) = response.get("result") {
        if let Some(colours) = result.as_array() {
            return Ok(colours
                .iter()
                .filter_map(|c| c.as_str().map(String::from))
                .collect());
        }
    }
    Ok(vec![])
}

pub fn activate_node_with_colour(client: &Client, node: NodeId, colour: &str) {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"activate_node_with_colour","params":[{}, "{}"],"id":1}}"#,
        node, colour
    );
    let res = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send();
    if let Err(e) = res {
        eprintln!(
            "Error activating node {} with colour {}: {}",
            node, colour, e
        );
    }
}

pub fn clear(client: &Client) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let json = r#"{"jsonrpc":"2.0","method":"clear","params":[],"id":1}"#;
    client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send()
}

pub fn draw_circle(client: &Client, radius: f32, origin: Vec3, mat: &str) {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"draw_circle","params":[{}, [{}, {}, {}], "{}"],"id":1}}"#,
        radius, origin.x, origin.y, origin.z, mat
    );
    let res = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send();
    if let Err(e) = res {
        eprintln!("Error drawing circle: {}", e);
    }
}

pub fn draw_rect(client: &Client, half_size: Vec2, origin: Vec3, mat: &str) {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"draw_rect","params":[[{}, {}], [{}, {}, {}], "{}"],"id":1}}"#,
        half_size.x, half_size.y, origin.x, origin.y, origin.z, mat
    );
    let res = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send();
    if let Err(e) = res {
        eprintln!("Error drawing rectangle: {}", e);
    }
}

fn main() {}
