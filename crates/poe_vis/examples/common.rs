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

use std::mem;

pub fn bytes_to_human(bytes: usize) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", size, units[unit])
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

pub fn get_node_position(client: &Client, node: NodeId) -> Vec3 {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"get_node_pos","params":[{}],"id":1}}"#,
        node
    );

    let res = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send();

    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<Value>() {
                if let Some(coords) = json.get("result").and_then(|v| v.as_array()) {
                    if coords.len() == 3 {
                        let x = coords[0].as_f64().unwrap_or(0.0) as f32;
                        let y = coords[1].as_f64().unwrap_or(0.0) as f32;
                        let z = coords[2].as_f64().unwrap_or(0.0) as f32;
                        return Vec3::new(x, y, z);
                    }
                }
            }
        }
        Err(e) => eprintln!("Error fetching position for node {}: {}", node, e),
    }

    Vec3::ZERO // Default to (0,0,0) on failure
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
pub fn activate_edge_with_colour(client: &Client, from: NodeId, to: NodeId, colour: &str) {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"activate_node_with_colour","params":[{}, {}, "{}"],"id":1}}"#,
        from, to, colour
    );
    let res = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send();
    if let Err(e) = res {
        eprintln!(
            "Error activating edge [{}..{}] with colour {}: {}",
            from, to, colour, e
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

pub fn draw_circle(client: &Client, radius: f32, origin: Vec3, mat: &str, glyph_duration: u64) {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"draw_circle","params":[{}, [{}, {}, {}], "{}", {}],"id":1}}"#,
        radius, origin.x, origin.y, origin.z, mat, glyph_duration
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

pub fn draw_rect(client: &Client, half_size: Vec2, origin: Vec3, mat: &str, glyph_duration: u64) {
    let json = format!(
        r#"{{"jsonrpc":"2.0","method":"draw_rect","params":[[{}, {}], [{}, {}, {}], "{}", {}],"id":1}}"#,
        half_size.x, half_size.y, origin.x, origin.y, origin.z, mat, glyph_duration
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
