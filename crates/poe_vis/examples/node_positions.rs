use poe_tree::type_wrappings::NodeId;
use reqwest::blocking::Client;
use serde_json::Value;

mod common;
use common::{ping, VIS_URL};

fn get_node_pos(
    client: &Client,
    node: NodeId,
) -> Result<(f64, f64, f64), Box<dyn std::error::Error>> {
    let json = format!(
        r#"{{"jsonrpc": "2.0", "method": "get_node_pos", "params": [{}], "id": 1}}"#,
        node
    );
    let resp: Value = client
        .post(VIS_URL)
        .header("Content-Type", "application/json")
        .body(json)
        .send()?
        .json()?;
    if let Some(result) = resp.get("result").and_then(|r| r.as_array()) {
        if result.len() == 3 {
            let x = result[0].as_f64().ok_or("Invalid x")?;
            let y = result[1].as_f64().ok_or("Invalid y")?;
            let z = result[2].as_f64().ok_or("Invalid z")?;
            return Ok((x, y, z));
        }
    }
    Err("Invalid response".into())
}

fn main() {
    let client = Client::new();
    ping(&client).expect("Server unreachable");

    let nodes: Vec<NodeId> = vec![
        10364, 52980, 56651, 59915, 59779, 38646, 3936, 50084, 13855, 4739, 44871,
    ];

    for node in nodes {
        match get_node_pos(&client, node) {
            Ok((x, y, z)) => println!("Node {} pos: [{}, {}, {}]", node, x, y, z),
            Err(e) => eprintln!("Node {} error: {}", node, e),
        }
    }
}
