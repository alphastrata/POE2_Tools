pub mod character;
pub mod consts;
pub mod coordinates;
pub mod debug_utils;
pub mod edges;
pub mod exploration;
pub mod filtering;
pub mod nodes;
pub mod pathfinding;
pub mod pob_utils;
pub mod skills;
pub mod stats;
pub mod type_wrappings;

use ahash::{AHashMap, AHashSet, HashSet, HashSetExt};
use consts::{CHAR_START_NODES, ORBIT_RADII, ORBIT_SLOTS};
use debug_utils::format_bytes;
use edges::Edge;
use nodes::PoeNode;
use skills::PassiveSkill;
use stats::Stat;
use type_wrappings::{GroupId, NodeId};

use serde_json::Value;
use std::{mem::size_of, time::Instant};

pub mod prelude {}

#[derive(Debug, Clone, Default)]
pub struct PassiveTree {
    pub groups: AHashMap<GroupId, coordinates::Group>,
    pub nodes: AHashMap<NodeId, PoeNode>,
    pub edges: AHashSet<Edge>,
    pub passive_skills: AHashMap<String, skills::PassiveSkill>,
}

pub fn quick_tree() -> PassiveTree {
    let file = std::fs::File::open("../../data/POE2_Tree.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let tree_data: serde_json::Value = serde_json::from_reader(reader).unwrap();
    let mut tree = PassiveTree::from_value(&tree_data).unwrap();

    tree.remove_hidden();
    tree
}

impl PassiveTree {
    const CULL_NODES_AFTER_THIS: f32 = 12_400.0;

    /// 1 connection is a leaf
    /// 2 connections is almost all nodes, and 'in' and an 'out'
    /// >3 is a branch point.
    pub fn branches(&self, active_nodes: &HashSet<NodeId>) -> HashSet<NodeId> {
        let mut branch_nodes = HashSet::new();

        active_nodes
            .iter()
            .filter(|node| self.neighbors(**node).count() >= 3)
            .for_each(|node| {
                branch_nodes.insert(*node);
            });

        branch_nodes
    }

    /// Prunes nodes and edges that lie beyond CULL_NODES_AFTER_THIS and are not character start nodes.
    /// Also prunes the adjacency list to reflect the removal of these nodes/edges.
    pub fn remove_hidden(&mut self) {
        let start_time = Instant::now();

        // Determine retained node IDs
        let retained_node_ids: AHashSet<_> = self
            .nodes
            .iter()
            .filter_map(|(&nid, node)| {
                let dist = (node.wx.powi(2) + node.wy.powi(2)).sqrt();
                if dist < Self::CULL_NODES_AFTER_THIS && !CHAR_START_NODES.contains(&nid) {
                    Some(nid)
                } else {
                    None
                }
            })
            .collect();

        // Size before pruning
        let edges_size_before = self.edges.len() * size_of::<Edge>();
        let nodes_size_before = self.nodes.len() * size_of::<PoeNode>();

        // Prune edges
        let initial_edge_count = self.edges.len();
        self.edges.retain(|edge| {
            retained_node_ids.contains(&edge.start) && retained_node_ids.contains(&edge.end)
        });
        let removed_edge_count = initial_edge_count - self.edges.len();

        // Prune nodes
        let initial_node_count = self.nodes.len();
        self.nodes
            .retain(|&nid, _| retained_node_ids.contains(&nid));
        let removed_node_count = initial_node_count - self.nodes.len();

        // Size after pruning but before shrink
        let edges_size_after_prune = self.edges.len() * size_of::<Edge>();
        let nodes_size_after_prune = self.nodes.len() * size_of::<PoeNode>();

        // Shrink to fit for memory optimization
        self.edges.shrink_to_fit();
        self.nodes.shrink_to_fit();

        // Log the results
        let duration = start_time.elapsed();
        log::debug!(
            "Pruned tree in {:?}. Removed {} edges and {} nodes.",
            duration,
            removed_edge_count,
            removed_node_count
        );
        log::debug!(
            "Memory usage: Edges - before: {}, after prune: {}.",
            format_bytes(edges_size_before),
            format_bytes(edges_size_after_prune),
        );
        log::debug!(
            "Memory usage: Nodes - before: {}, after prune: {}.",
            format_bytes(nodes_size_before),
            format_bytes(nodes_size_after_prune),
        );
    }

    /// The main parser for the POE2_Tree.json we found...
    /// NOTE this panics! intentionally, if there's a problem parsing I want to know and be pointed to the place we need to fix
    pub fn from_value(val: &Value) -> Result<Self, serde_json::Error> {
        //TODO: this is pretty nasty...

        let groups: AHashMap<GroupId, coordinates::Group> = val
            .get("passive_tree")
            .and_then(|tree| tree.get("groups"))
            .and_then(|groups| groups.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(gid, gval)| {
                        let gid = gid.parse::<NodeId>().ok()?;

                        // Value doesn't support deserialising to f32
                        let x = gval.get("x")?.as_f64()?;
                        let y = gval.get("y")?.as_f64()?;
                        Some((
                            gid,
                            coordinates::Group {
                                x: x as f32,
                                y: y as f32,
                            },
                        ))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let passive_skills: AHashMap<String, skills::PassiveSkill> = val
            .get("passive_skills")
            .and_then(|ps| ps.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(skill_id, skill_val)| {
                        if skill_val.get("is_just_icon").is_some() {
                            log::trace!("just an icon!");
                            return None;
                        }
                        let stats_json = skill_val.get("stats")?;
                        match serde_json::from_value::<PassiveSkill>(skill_val.clone()) {
                            Ok(mut skill) => {
                                // NOTE: custom deserialising of the Stat type.
                                if let Some(stats_map) = stats_json.as_object() {
                                    for (k, v) in stats_map {
                                        if let Some(n) = v.as_f64() {
                                            skill.stats.push(Stat::from_key_value(k, n));
                                        }
                                    }
                                }
                                Some((skill_id.clone(), skill))
                            }
                            Err(e) => {
                                log::error!("Failed to parse skill {}: {}", skill_id, e);
                                log::error!("{:#?}", skill_val);
                                None
                            }
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let nodes: AHashMap<NodeId, PoeNode> = val
            .get("passive_tree")
            .and_then(|tree| tree.get("nodes"))
            .and_then(|nodes| nodes.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(node_id, nval)| {
                        let node_id = node_id.parse::<NodeId>().ok()?;
                        let skill_id = nval.get("skill_id")?.as_str()?.to_string();
                        let parent =
                            nval.get("parent").and_then(|v| v.as_u64()).unwrap_or(0) as NodeId;
                        let radius = nval.get("radius").and_then(|v| v.as_u64()).unwrap_or(0) as u8;
                        let position =
                            nval.get("position").and_then(|v| v.as_u64()).unwrap_or(0) as NodeId;

                        // Calculate world position with null safety
                        let (wx, wy) = {
                            let mut wx_wy = (0.0, 0.0);
                            groups.get(&parent).iter().for_each(|group| {
                                let result = calculate_world_position(group, radius, position);
                                wx_wy = result;
                            });
                            wx_wy
                        };

                        if let Some(skill) = passive_skills.get(&skill_id) {
                            let name = skill.name();
                            let is_notable = skill.is_notable();

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
                                    wx,
                                    wy,
                                    active: false,
                                },
                            ))
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let edges: AHashSet<Edge> = match val.get("passive_tree") {
            Some(tree) => match tree.get("nodes") {
                Some(nodes) => match nodes.as_object() {
                    Some(obj) => {
                        obj.iter()
                            .flat_map(|(from_id, node)| {
                                let from_id = match from_id.parse::<NodeId>() {
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
                                                    end: to_id as NodeId,
                                                }),
                                                None => {
                                                    eprintln!(
                                                        "Invalid connection in node `{}`: {:?}",
                                                        from_id, connection
                                                    );
                                                    log::error!("{}, {:#?}", cons, array);
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

    fn node(&self, target: NodeId) -> &PoeNode {
        self.nodes.get(&target).unwrap()
    }

    fn passive_for_node(&self, arg: &PoeNode) -> &PassiveSkill {
        self.passive_skills.get(&arg.skill_id).unwrap()
    }
}

/// Make the world position (wx, wy) for a node.
pub fn calculate_world_position(
    group: &coordinates::Group,
    radius: u8,
    position: NodeId,
) -> (f32, f32) {
    let r = radius as usize;
    let position = position as usize;
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
    }) as f32;

    // Calculate the angle in radians
    //TODO: f16, or f32?

    let angle = match slots as NodeId {
        16 => {
            // Use predefined angles for 16-slot orbits
            const PREDEFINED_16: [f32; 16] = [
                0.0, 30.0, 45.0, 60.0, 90.0, 120.0, 135.0, 150.0, 180.0, 210.0, 225.0, 240.0,
                270.0, 300.0, 315.0, 330.0,
            ];
            PREDEFINED_16[position % 16].to_radians()
        }
        40 => {
            // Use predefined angles for 40-slot orbits
            const PREDEFINED_40: [f32; 40] = [
                0.0, 10.0, 20.0, 30.0, 40.0, 45.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0,
                120.0, 130.0, 135.0, 140.0, 150.0, 160.0, 170.0, 180.0, 190.0, 200.0, 210.0, 220.0,
                225.0, 230.0, 240.0, 250.0, 260.0, 270.0, 280.0, 290.0, 300.0, 310.0, 315.0, 320.0,
                330.0, 340.0, 350.0,
            ];
            PREDEFINED_40[position % 40].to_radians()
        }
        _ => {
            // Uniform angle division for ALL other cases
            (2.0 * std::f32::consts::PI * position as f32 / slots) - (std::f32::consts::PI / 2.0)
        }
    };

    //polar-to-Cartesian
    (
        group.x + radius_value * angle.cos(),
        group.y + radius_value * angle.sin(),
    )
}

/// Make the world position (wx, wy) for a node.
pub fn calculate_world_position_with_negative_y(
    group: &coordinates::Group,
    radius: u8,
    position: NodeId,
) -> (f32, f32) {
    let (wx, mut wy) = calculate_world_position(group, radius, position);
    wy *= -1.0;
    (wx, wy)
}

pub fn get_circle_radius(radius: u8, position: NodeId, group: &GroupId) -> f32 {
    *ORBIT_RADII.get(radius as usize).unwrap_or_else(|| {
        panic!(
            "Failed to retrieve radius for r={} with position={} and group coordinates=({:#?})",
            radius, position, group
        )
    })
}

#[cfg(test)]
mod tests {

    use crate::{edges::Edge, quick_tree};

    #[test]
    fn bidirectional_edges() {
        let tree = quick_tree();
        for edge in &tree.edges {
            let reverse_edge = Edge {
                start: edge.end,
                end: edge.start,
            };
            assert!(
                tree.edges.contains(&reverse_edge),
                "Edge from {} to {} is not bidirectional",
                edge.end,
                edge.start
            );
        }
        println!("All edges are bidirectional.");
    }
}
