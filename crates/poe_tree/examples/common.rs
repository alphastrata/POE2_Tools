use poe_tree::{type_wrappings::NodeId, PassiveTree};
use reqwest::blocking::Client;

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

pub fn send_node_command(client: &Client, node: NodeId, method: &str) {
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
