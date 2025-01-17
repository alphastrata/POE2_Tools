//$ src/data/poe_tree/mod.rs
pub mod consts;
pub mod coordinates;
pub mod edges;
pub mod nodes;
pub mod pathfinding;
pub mod skills;
pub mod stats;
pub mod type_wrappings;
use edges::Edge;
use nodes::PoeNode;
use type_wrappings::{GroupId, NodeId};

use std::{
    collections::{HashMap, HashSet},
    fs,
};

#[derive(Debug, Clone, Default)]
pub struct PassiveTree<'data> {
    pub groups: HashMap<GroupId, coordinates::Group>,
    pub nodes: HashMap<NodeId, PoeNode<'data>>,
    pub edges: HashSet<Edge>, // Using a HashSet for bidirectional uniqueness
    pub passive_skills: HashMap<String, skills::PassiveSkill>,
}
impl<'data> PassiveTree<'data> {
    pub fn from_file(path: &str) -> (Self, serde_json::Value) {
        let data = fs::read_to_string(path).expect("Failed to read JSON");
        let json: serde_json::Value = serde_json::from_str(&data).expect("Invalid JSON");

        let mut groups = HashMap::new();
        if let Some(obj) = json["passive_tree"]["groups"].as_object() {
            for (gid, gval) in obj {
                let gx = gval["x"].as_f64().unwrap_or(0.0);
                let gy = gval["y"].as_f64().unwrap_or(0.0);
                groups.insert(
                    gid.parse::<usize>().unwrap_or_default(),
                    coordinates::Group { x: gx, y: gy },
                );
            }
        }

        let mut nodes = HashMap::new();
        if let Some(obj) = json["passive_tree"]["nodes"].as_object() {
            for (node_id, nval) in obj {
                let skill_id = nval["skill_id"]
                    .as_str()
                    .map(String::from)
                    .expect("All skills have a skill_id");
                let node = PoeNode {
                    node_id: node_id.parse::<usize>().unwrap(),
                    name: nval["passive_skills"][&skill_id].to_string(),
                    skill_id,
                    parent: nval["parent"].as_u64().unwrap_or(0) as usize,
                    radius: nval["radius"].as_u64().unwrap_or(0) as u8,
                    position: nval["position"].as_u64().unwrap_or(0) as usize,
                    is_notable: false,
                    stats: &[],
                    wx: 0.0,
                    wy: 0.0,
                    active: false,
                };
                nodes.insert(node.node_id, node);
            }
        }

        let mut edges = HashSet::new();
        for (from_id, node) in &nodes {
            if let Some(connections) =
                json["passive_tree"]["nodes"][&from_id.to_string()]["connections"].as_array()
            {
                for connection in connections {
                    if let Some(to_id) = connection.as_u64() {
                        let edge = Edge::new(
                            *from_id,
                            to_id as usize,
                            &PassiveTree {
                                groups: groups.clone(),
                                nodes: nodes.clone(),
                                edges: HashSet::new(),
                                passive_skills: HashMap::new(),
                            },
                        );
                        edges.insert(edge);
                    }
                }
            }
        }

        //TODO: Node's do not know their location

        (
            PassiveTree {
                groups,
                nodes,
                edges,
                passive_skills: HashMap::new(),
            },
            json,
        )
    }
}
