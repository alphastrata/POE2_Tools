//$ src/data/poe_tree/mod.rs
pub mod consts;
pub mod coordinates;
pub mod edges;
pub mod nodes;
pub mod pathfinding;
pub mod skills;
pub mod stats;
pub mod type_wrappings;
use consts::{ORBIT_RADII, ORBIT_SLOTS};
use edges::Edge;
use nodes::PoeNode;
use serde_json::Value;
use stats::Stat;
use type_wrappings::{GroupId, NodeId};

use std::{
    collections::{HashMap, HashSet},
    fs,
};
#[derive(Debug, Clone, Default)]
pub struct PassiveTree {
    // Removed lifetime parameter since we'll own our data
    pub groups: HashMap<GroupId, coordinates::Group>,
    pub nodes: HashMap<NodeId, PoeNode>,
    pub edges: HashSet<Edge>,
    pub passive_skills: HashMap<String, skills::PassiveSkill>,
}

impl PassiveTree {
    pub fn from_value(val: &Value) -> Self {
        // First, build groups
        let groups: HashMap<GroupId, coordinates::Group> = val["passive_tree"]["groups"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .flat_map(|(gid, gval)| {
                        Some((
                            gid.parse::<usize>().ok()?,
                            coordinates::Group {
                                x: gval["x"].as_f64()?,
                                y: gval["y"].as_f64()?,
                            },
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build passive skills map
        let passive_skills: HashMap<String, skills::PassiveSkill> = val["passive_skills"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(skill_id, skill_val)| {
                        // Convert skill_val to PassiveSkill - implementation depends on your PassiveSkill structure
                        let skill = serde_json::from_value(skill_val.clone()).unwrap();
                        Some((skill_id.clone(), skill))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build nodes
        let nodes: HashMap<NodeId, PoeNode> = val["passive_tree"]["nodes"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(node_id, nval)| {
                        let node_id = node_id.parse::<usize>().ok()?;
                        let skill_id = nval["skill_id"].as_str()?.to_string();
                        let parent = nval["parent"].as_u64().unwrap_or(0) as usize;
                        let radius = nval["radius"].as_u64().unwrap_or(0) as u8;
                        let position = nval["position"].as_u64().unwrap_or(0) as usize;

                        // Calculate world position
                        let (wx, wy) = if let Some(group) = groups.get(&parent) {
                            calculate_world_position(group, radius, position)
                        } else {
                            (0.0, 0.0)
                        };

                        // Get skill details - now we clone the data we need
                        let skill = passive_skills.get(&skill_id);
                        let name = skill
                            .and_then(|s| s.name.as_ref())
                            .cloned()
                            .unwrap_or_default();
                        let is_notable = skill.map(|s| s.is_notable).unwrap_or(false);
                        let stats = skill
                            .map(|s| s.stats.clone()) // Clone the stats
                            .unwrap_or_default();

                        Some((
                            node_id,
                            PoeNode {
                                node_id,
                                skill_id,
                                parent,
                                radius,
                                position,
                                name,
                                is_notable,
                                stats,
                                wx,
                                wy,
                                active: false,
                            },
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build edges
        let edges = val["passive_tree"]["nodes"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .flat_map(|(from_id, node)| {
                        let from_id = from_id.parse::<usize>().unwrap_or_default();
                        node["connections"].as_array().unwrap().iter().filter_map(
                            move |connection| {
                                let to_id = connection.as_u64()? as usize;
                                Some(Edge {
                                    from: from_id,
                                    to: to_id,
                                })
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();

        PassiveTree {
            groups,
            nodes,
            edges,
            passive_skills,
        }
    }
}

fn calculate_world_position(group: &coordinates::Group, radius: u8, position: usize) -> (f64, f64) {
    let radius = ORBIT_RADII.get(radius as usize).copied().unwrap_or(0.0);
    let slots = ORBIT_SLOTS.get(radius as usize).copied().unwrap_or(1) as f64;
    let angle = position as f64 * (2.0 * std::f64::consts::PI / slots);

    (
        group.x + radius * angle.cos(),
        group.y + radius * angle.sin(),
    )
}
