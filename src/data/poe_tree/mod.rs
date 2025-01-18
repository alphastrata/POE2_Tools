//$ src\data\poe_tree\mod.rs
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
    pub groups: HashMap<GroupId, coordinates::Group>,
    pub nodes: HashMap<NodeId, PoeNode>,
    pub edges: HashSet<Edge>,
    pub passive_skills: HashMap<String, skills::PassiveSkill>,
}

impl PassiveTree {
    /// The main parser for the POE2_Tree.json we found...
    /// NOTE this panics! intentionally, if there's a problem parsing I want to know and be pointed to the place we need to fixs
    pub fn from_value(val: &Value) -> Result<Self, serde_json::Error> {
        //TODO: this is pretty nasty...

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
                                eprintln!("{:#?}", skill_val);
                                panic!();
                                #[allow(unreachable_code)]
                                None
                            }
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

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
                        let (wx, wy) = {
                            let mut wx_wy = (0.0, 0.0);
                            groups.get(&parent).iter().for_each(|group| {
                                match calculate_world_position(group, radius, position) {
                                    result => {
                                        // eprintln!("{:?}", nval);
                                        wx_wy = result;
                                    }
                                }
                            });
                            wx_wy
                        };

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

        let edges: HashSet<Edge> = match val.get("passive_tree") {
            Some(tree) => match tree.get("nodes") {
                Some(nodes) => match nodes.as_object() {
                    Some(obj) => {
                        obj.iter()
                            .flat_map(|(from_id, node)| {
                                let from_id = match from_id.parse::<usize>() {
                                    Ok(id) => id,
                                    Err(e) => {
                                        eprintln!("Failed to parse from_id `{}`: {}", from_id, e);
                                        panic!("Invalid from_id in data");
                                    }
                                };

                                match node.get("connections") {
                                    /*
                                     "connections": [ {"id": 29361,"radius": 3},{"id": 65437,"radius": -5}...]
                                    */
                                    Some(cons) => match cons.as_array() {
                                        Some(array) => array.iter().filter_map(move |connection| {
                                            match connection.get("id").and_then(|id| id.as_u64()) {
                                                Some(to_id) => Some(Edge {
                                                    start: from_id,
                                                    end: to_id as usize,
                                                }),
                                                None => {
                                                    eprintln!(
                                                        "Invalid connection in node `{}`: {:?}",
                                                        from_id, connection
                                                    );
                                                    dbg!(cons, array);
                                                    None
                                                }
                                            }
                                        }),
                                        None => {
                                            eprintln!(
                "Connections field is not an array in node `{}`: {:?}",
                from_id, cons
            );
                                            panic!()
                                        }
                                    },
                                    None => {
                                        eprintln!(
                                            "Missing connections field in node `{}`: {:?}",
                                            from_id, node
                                        );
                                        panic!()
                                    }
                                }
                            })
                            .collect()
                    }
                    None => {
                        eprintln!("Nodes field is not an object: {:?}", nodes);
                        panic!("Invalid nodes structure in data");
                    }
                },
                None => {
                    eprintln!("Missing nodes field in tree: {:?}", tree);
                    panic!("Invalid tree structure in data");
                }
            },
            None => {
                eprintln!("Missing passive_tree field in val: {:?}", val);
                panic!("Invalid passive_tree structure in data");
            }
        };

        Ok(PassiveTree {
            groups,
            nodes,
            edges,
            passive_skills,
        })
    }
}

/// Make the world position (wx, wy) for a node.
fn calculate_world_position(group: &coordinates::Group, radius: u8, position: usize) -> (f64, f64) {
    let r = radius as usize;
    let radius_value = ORBIT_RADII.get(r).unwrap_or_else(|| {
        panic!(
            "Failed to retrieve radius for r={} with position={} and group coordinates=({}, {})",
            r, position, group.x, group.y
        )
    });

    let slots = ORBIT_SLOTS.get(r).copied().unwrap_or_else(|| {
        eprintln!(
            "Failed to retrieve slots for r={} with position={} and group coordinates=({}, {})",
            radius, position, group.x, group.y
        );
        eprintln!("Defaulting to 60 slots.");
        60
    }) as f64;

    // Calculate the angle in radians
    //TODO: f16, or f32?

    let angle = match slots as usize {
        16 => {
            // Use predefined angles for 16-slot orbits
            const PREDEFINED_16: [f64; 16] = [
                0.0, 30.0, 45.0, 60.0, 90.0, 120.0, 135.0, 150.0, 180.0, 210.0, 225.0, 240.0,
                270.0, 300.0, 315.0, 330.0,
            ];
            PREDEFINED_16[position % 16].to_radians()
        }
        40 => {
            // Use predefined angles for 40-slot orbits
            const PREDEFINED_40: [f64; 40] = [
                0.0, 10.0, 20.0, 30.0, 40.0, 45.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0,
                120.0, 130.0, 135.0, 140.0, 150.0, 160.0, 170.0, 180.0, 190.0, 200.0, 210.0, 220.0,
                225.0, 230.0, 240.0, 250.0, 260.0, 270.0, 280.0, 290.0, 300.0, 310.0, 315.0, 320.0,
                330.0, 340.0, 350.0,
            ];
            PREDEFINED_40[position % 40].to_radians()
        }
        _ => {
            // Uniform angle division for ALL other cases
            (2.0 * std::f64::consts::PI * position as f64 / slots) - (std::f64::consts::PI / 2.0)
        }
    };

    //polar-to-Cartesian
    (
        group.x + radius_value * angle.cos(),
        group.y + radius_value * angle.sin(),
    )
}
