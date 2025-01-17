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
    pub fn from_value(val: &Value) -> Result<Self, serde_json::Error> {
        // First, build groups with error handling
        let groups: HashMap<GroupId, coordinates::Group> = val
            .get("passive_tree")
            .and_then(|tree| tree.get("groups"))
            .and_then(|groups| groups.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(gid, gval)| {
                        let gid = gid.parse::<usize>().ok()?;
                        let x = gval.get("x")?.as_f64()?;
                        let y = gval.get("y")?.as_f64()?;
                        Some((gid, coordinates::Group { x, y }))
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build passive skills map with proper error handling
        let passive_skills: HashMap<String, skills::PassiveSkill> = val
            .get("passive_skills")
            .and_then(|skills| skills.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(skill_id, skill_val)| {
                        match serde_json::from_value(skill_val.clone()) {
                            Ok(skill) => Some((skill_id.clone(), skill)),
                            Err(e) => {
                                eprintln!("Failed to parse skill {}: {}", skill_id, e);
                                None
                            }
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        // Build nodes with better error handling and null checks
        let nodes: HashMap<NodeId, PoeNode> = val
            .get("passive_tree")
            .and_then(|tree| tree.get("nodes"))
            .and_then(|nodes| nodes.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(node_id, nval)| {
                        let node_id = node_id.parse::<usize>().ok()?;
                        let skill_id = nval.get("skill_id")?.as_str()?.to_string();
                        let parent =
                            nval.get("parent").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        let radius = nval.get("radius").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                        let position =
                            nval.get("position").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                        // Calculate world position with null safety
                        let (wx, wy) = groups
                            .get(&parent)
                            .map(|group| calculate_world_position(group, radius, position))
                            .unwrap_or((0.0, 0.0));

                        // Get skill details with proper null handling
                        let skill = passive_skills.get(&skill_id);
                        let name = skill
                            .and_then(|s| s.name.as_ref())
                            .cloned()
                            .unwrap_or_default();
                        let is_notable = skill.map(|s| s.is_notable).unwrap_or(false);
                        let stats = skill.map(|s| s.stats.clone()).unwrap_or_default();

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

        // Build edges with proper error handling
        let edges: HashSet<Edge> = val
            .get("passive_tree")
            .and_then(|tree| tree.get("nodes"))
            .and_then(|nodes| nodes.as_object())
            .map(|obj| {
                obj.iter()
                    .flat_map(|(from_id, node)| {
                        let from_id = from_id.parse::<usize>().ok().unwrap_or_default();
                        node.get("connections")
                            .and_then(|cons| cons.as_array())
                            .unwrap()
                            .iter()
                            .filter_map(move |connection| {
                                connection.as_u64().map(|to_id| Edge {
                                    from: from_id,
                                    to: to_id as usize,
                                })
                            })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(PassiveTree {
            groups,
            nodes,
            edges,
            passive_skills,
        })
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
