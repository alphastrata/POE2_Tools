pub mod character;
pub mod config;
pub mod consts;
pub mod coordinates;
pub mod debug_utils;
pub mod edges;
pub mod nodes;
pub mod pathfinding;
pub mod skills;
pub mod stats;
pub mod type_wrappings;

use consts::{CHAR_START_NODES, ORBIT_RADII, ORBIT_SLOTS};
use debug_utils::format_bytes;
use edges::Edge;
use nodes::PoeNode;
use type_wrappings::{GroupId, NodeId};

use serde_json::Value;
use std::{
    collections::{HashMap, HashSet},
    mem::size_of,
    time::Instant,
};

pub mod prelude {}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::{edges::Edge, stats::Operand, PassiveTree};

    #[test]
    fn path_between_flow_like_water_and_chaos_inoculation() {
        let file = File::open("../../data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

        // Use fuzzy search to find nodes
        let flow_ids = tree.fuzzy_search_nodes("flow like water");
        let chaos_ids = tree.fuzzy_search_nodes("chaos inoculation");

        assert!(!flow_ids.is_empty(), "No node found for 'flow like water'");
        assert!(
            !chaos_ids.is_empty(),
            "No node found for 'chaos inoculation'"
        );

        let start_id = flow_ids[0];
        let target_id = chaos_ids[0];

        // Find shortest path using Dijkstra's Algorithm
        let path = tree.find_shortest_path(start_id, target_id);
        if path.is_empty() {
            println!("No path found between {} and {}", start_id, target_id);
        } else {
            println!("Path found: {:?}", path);
            for node_id in path.iter() {
                let n = tree.nodes.get(node_id).unwrap();
                if !n.name.contains("Attribute") {
                    print!("(ID:{} NAME: {}) ->", node_id, n.name);
                } else {
                    print!("[ID:{}] ->", node_id);
                }
            }
        }
        // Update this value based on expected path length after refactoring
        assert_eq!(path.len(), 15, "Path length mismatch");
        println!("{:#?}", path);
    }

    #[test]
    fn bidirectional_edges() {
        let file = File::open("../../data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

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

    #[test]
    fn path_between_avatar_of_fire_and_over_exposure() {
        let file = File::open("../../data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

        // Use fuzzy search to find nodes
        let avatar_ids = tree.fuzzy_search_nodes("Avatar of Fire");
        let over_exposure_ids = tree.fuzzy_search_nodes("Overexposure");

        assert!(!avatar_ids.is_empty(), "No node found for 'Avatar of Fire'");
        assert!(
            !over_exposure_ids.is_empty(),
            "No node found for 'OverExposure'"
        );

        let start_id = avatar_ids[0];
        let target_id = over_exposure_ids[0];

        // Find shortest path using Dijkstra's Algorithm
        let path = tree.find_shortest_path(start_id, target_id);

        if path.is_empty() {
            panic!(
                "No path found between {} and {}",
                tree.nodes[&start_id].name, tree.nodes[&target_id].name
            );
        } else {
            println!("Path found: {:?}", path);
            for node_id in path.iter() {
                let n = tree.nodes.get(node_id).unwrap();
                println!("(ID:{} NAME: {})", node_id, n.name);
            }
        }
        // Update this value based on expected path length after refactoring
        assert_eq!(path.len(), 27, "Path length mismatch");
    }

    #[test]
    fn collect_life_nodes_from_real_tree() {
        let file = File::open("../../data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();
        let tree: PassiveTree = PassiveTree::from_value(&u).unwrap();

        let mut life_nodes = Vec::new();
        let mut total_life = 0.0;

        tree.nodes.values().for_each(|node| {
            node.stats.iter().for_each(|stat| {
                //NOTE: we should do 'something' to allow any of ["Maximum Life", "max life", 'maximum life', "maximum_life"] to work..
                // maybe a stat.name.supported_patterns_of($needle) -> bool, replacing ' ' with '_' should probs get us 99% of the way there
                if stat.name.contains("maximum_life") && matches!(stat.operand, Operand::Add) {
                    life_nodes.push(node.node_id);
                    total_life += stat.value;
                } else if stat.name.contains("life") {
                    eprintln!("'life' keyword found in {}, maybe we should modify on entry to make nicer..", stat.name);
                }
            });
        });

        println!(
            "Life Nodes Count: {}, Total Life Added: {}",
            life_nodes.len(),
            total_life
        );

        assert!(!life_nodes.is_empty(), "Expected at least one life node");
        assert!(total_life > 0.0, "Total life should be greater than zero");
    }

    #[test]
    fn collect_evasion_percentage_nodes_from_real_tree() {
        let file = File::open("../../data/POE2_Tree.json").unwrap();
        let reader = BufReader::new(file);
        let u = serde_json::from_reader(reader).unwrap();

        let tree = PassiveTree::from_value(&u).unwrap();
        let mut evasion_nodes = Vec::new();
        let mut total_evasion_percent = 0.0;

        tree.nodes.values().for_each(|node| {
            node.stats.iter().for_each(|stat| {
                if stat.name.contains("evasion_rating") && matches!(stat.operand, Operand::Percentage)
                {
                    evasion_nodes.push(node.node_id);
                    total_evasion_percent += stat.value;
                } else if stat.name.contains("evasion") {
                    eprintln!("'evasion' keyword found in {}, maybe we should modify on entry to make nicer..", stat.name);
                }
            });
        });

        println!(
            "Evasion Nodes Count: {}, Total Evasion Percentage: {}",
            evasion_nodes.len(),
            total_evasion_percent
        );

        assert!(
            !evasion_nodes.is_empty(),
            "Expected at least one evasion node"
        );
        assert!(
            total_evasion_percent > 0.0,
            "Total evasion percentage should be greater than zero"
        );
    }
}

#[derive(Debug, Clone, Default)]
pub struct PassiveTree {
    pub groups: HashMap<GroupId, coordinates::Group>,
    pub nodes: HashMap<NodeId, PoeNode>,
    pub edges: HashSet<Edge>,
    pub passive_skills: HashMap<String, skills::PassiveSkill>,
}

impl PassiveTree {
    const CULL_NODES_AFTER_THIS: f64 = 12_400.0;

    /// There's lots of nodes that we don't wish to plot (usually), this removes them.
    pub fn remove_hidden(&mut self) {
        let start_time = Instant::now();

        // Determine retained node IDs
        let retained_node_ids: std::collections::HashSet<_> = self
            .nodes
            .iter()
            .filter_map(|(&nid, node)| {
                let dist = (node.wx.powi(2) + node.wy.powi(2)).sqrt();
                if dist < Self::CULL_NODES_AFTER_THIS || CHAR_START_NODES.contains(&nid) {
                    Some(nid)
                } else {
                    None
                }
            })
            .collect();

        // Size before pruning
        let edges_size_before = self.edges.len() * size_of::<Edge>();
        let nodes_size_before = self.nodes.len() * size_of::<PoeNode>();

        // Count removed edges
        let initial_edge_count = self.edges.len();
        self.edges.retain(|edge| {
            retained_node_ids.contains(&edge.start) && retained_node_ids.contains(&edge.end)
        });
        let removed_edge_count = initial_edge_count - self.edges.len();

        // Count removed nodes
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
                                let result = calculate_world_position(group, radius, position);
                                wx_wy = result;
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
